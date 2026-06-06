//! Pure-Rust Expert Witness Format (E01) writer — no libewf/ewfacquire dependency.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use flate2::write::ZlibEncoder;
use flate2::Compression;
use md5::{Digest as Md5Digest, Md5};
use sha2::Sha256;

const CHUNK_SIZE: usize = 32 * 1024;
const SECTION_DESC: usize = 76;
const EWF_SIG: &[u8; 8] = b"EVF\t\r\n\xff\x00";

pub struct EwfWriter {
    dest: PathBuf,
}

impl EwfWriter {
    pub fn new(dest: &Path) -> Self {
        Self { dest: dest.to_path_buf() }
    }

    pub fn acquire(
        &self,
        source: &str,
        cancel: &AtomicBool,
    ) -> Result<String, String> {
        let src_size = crate::block_device::device_size(source)?;
        let src = File::open(source).map_err(|e| format!("Cannot open {source}: {e}"))?;
        let has_known_size = src_size > 0;
        let mut reader = std::io::BufReader::with_capacity(super::hashing::HASH_BUFFER_SIZE, src);

        let mut out = File::create(&self.dest)
            .map_err(|e| format!("Cannot create {}: {e}", self.dest.display()))?;

        let mut md5 = Md5::new();
        let mut sha256 = Sha256::new();
        let mut chunks: Vec<Vec<u8>> = Vec::new();
        let mut buf = vec![0u8; CHUNK_SIZE];
        let mut total_read: u64 = 0;

        loop {
            if cancel.load(Ordering::SeqCst) {
                let _ = std::fs::remove_file(&self.dest);
                return Err("CANCELLED".into());
            }
            let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            let slice = &buf[..n];
            md5.update(slice);
            sha2::Digest::update(&mut sha256, slice);
            total_read += n as u64;

            let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
            enc.write_all(slice).map_err(|e| e.to_string())?;
            chunks.push(enc.finish().map_err(|e| e.to_string())?);

            let pct = if has_known_size {
                (total_read as f64 / src_size as f64) * 90.0
            } else {
                0.0
            };
            super::progress::update_progress(
                pct,
                &format!(
                    "E01 imaging: {} / {}",
                    crate::block_device::format_capacity(total_read),
                    if has_known_size {
                        crate::block_device::format_capacity(src_size)
                    } else {
                        "unknown".into()
                    }
                ),
                total_read,
                if has_known_size { src_size } else { 0 },
            );
        }

        let md5_digest = md5.finalize();
        let sha256_digest = sha2::Digest::finalize(sha256);
        write_ewf_file(
            &mut out,
            &chunks,
            total_read,
            src_size,
            md5_digest.as_slice(),
            sha256_digest.as_slice(),
        )?;
        out.flush().map_err(|e| e.to_string())?;

        let hash = format!("{:x}", sha256_digest);
        let done_total = if has_known_size { src_size } else { total_read };
        super::progress::update_progress(100.0, "E01 acquisition complete", done_total, done_total);
        super::progress::finish_progress(Ok(hash.clone()));
        Ok(hash)
    }
}

fn write_section_desc(w: &mut File, kind: &str, next: u64, size: u64) -> Result<(), String> {
    let mut hdr = [0u8; SECTION_DESC];
    let kb = kind.as_bytes();
    hdr[..kb.len().min(16)].copy_from_slice(&kb[..kb.len().min(16)]);
    hdr[16..24].copy_from_slice(&next.to_le_bytes());
    hdr[24..32].copy_from_slice(&size.to_le_bytes());
    let sum: u32 = hdr.iter().map(|&b| b as u32).sum();
    hdr[72..76].copy_from_slice(&sum.to_le_bytes());
    w.write_all(&hdr).map_err(|e| e.to_string())
}

fn write_ewf_file(
    w: &mut File,
    chunks: &[Vec<u8>],
    media_size: u64,
    src_size: u64,
    md5_digest: &[u8],
    sha256_digest: &[u8],
) -> Result<(), String> {
    let header_body = build_header_body(media_size.max(src_size));
    let volume_body = build_volume_body(media_size.max(src_size));
    let table_body = build_table_body(chunks);
    let data_body: Vec<u8> = chunks.iter().flatten().copied().collect();
    let digest_body = build_digest_body(md5_digest, sha256_digest);

    let v_off = (SECTION_DESC + header_body.len()) as u64;
    let t_off = v_off + SECTION_DESC as u64 + volume_body.len() as u64;
    let d_off = t_off + SECTION_DESC as u64 + table_body.len() as u64;
    let g_off = d_off + SECTION_DESC as u64 + data_body.len() as u64;
    let done_off = g_off + SECTION_DESC as u64 + digest_body.len() as u64;

    write_section_desc(w, "header", v_off, header_body.len() as u64)?;
    w.write_all(&header_body).map_err(|e| e.to_string())?;
    write_section_desc(w, "volume", t_off, volume_body.len() as u64)?;
    w.write_all(&volume_body).map_err(|e| e.to_string())?;
    write_section_desc(w, "table", d_off, table_body.len() as u64)?;
    w.write_all(&table_body).map_err(|e| e.to_string())?;
    write_section_desc(w, "data", g_off, data_body.len() as u64)?;
    w.write_all(&data_body).map_err(|e| e.to_string())?;
    write_section_desc(w, "digest", done_off, digest_body.len() as u64)?;
    w.write_all(&digest_body).map_err(|e| e.to_string())?;
    write_section_desc(w, "done", 0, 0)?;
    Ok(())
}

fn utf16le(s: &str) -> Vec<u8> {
    let mut v = Vec::new();
    for c in s.encode_utf16() {
        v.extend_from_slice(&c.to_le_bytes());
    }
    v.extend_from_slice(&[0, 0]);
    v
}

fn build_header_body(media_size: u64) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(EWF_SIG);
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&utf16le("CollectionLoom"));
    b.extend_from_slice(&utf16le("1.0"));
    b.extend_from_slice(&utf16le("Forensic acquisition"));
    b.extend_from_slice(&utf16le("CollectionLoom E01"));
    b.extend_from_slice(&media_size.to_le_bytes());
    b
}

fn build_volume_body(media_size: u64) -> Vec<u8> {
    let sectors = media_size / 512;
    let mut b = Vec::new();
    b.extend_from_slice(&[0x90, 0x01]);
    b.extend_from_slice(&1u32.to_le_bytes());
    b.extend_from_slice(&512u32.to_le_bytes());
    b.extend_from_slice(&(sectors as u64).to_le_bytes());
    b.extend_from_slice(&1u32.to_le_bytes());
    b.extend_from_slice(&[0u8; 20]);
    b.extend_from_slice(&utf16le("Physical"));
    b
}

fn build_table_body(chunks: &[Vec<u8>]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&(chunks.len() as u32).to_le_bytes());
    let mut offset = 0u64;
    for c in chunks {
        b.extend_from_slice(&offset.to_le_bytes());
        b.extend_from_slice(&(c.len() as u32).to_le_bytes());
        offset += c.len() as u64;
    }
    b
}

fn build_digest_body(md5_digest: &[u8], sha256_digest: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(md5_digest);
    b.extend_from_slice(sha256_digest);
    b
}

pub fn e01_path(destination: &Path) -> std::path::PathBuf {
    let s = destination.to_string_lossy();
    if s.ends_with(".E01") || s.ends_with(".e01") {
        destination.to_path_buf()
    } else {
        destination.with_extension("E01")
    }
}
