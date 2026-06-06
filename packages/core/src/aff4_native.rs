//! Pure-Rust AFF4-L container writer (ZIP + RDF turtle metadata).

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const STREAM_PATH: &str = "default/data";

pub struct Aff4Writer {
    dest: PathBuf,
}

impl Aff4Writer {
    pub fn new(dest: &Path) -> Self {
        Self {
            dest: dest.to_path_buf(),
        }
    }

    pub fn acquire(
        &self,
        source: &str,
        split_size: Option<u64>,
        cancel: &AtomicBool,
    ) -> Result<String, String> {
        if let Some(sz) = split_size {
            return self.acquire_split(source, sz, cancel);
        }
        let (hash, _) = self.acquire_single(&self.dest, source, cancel, None, None)?;
        Ok(hash)
    }

    fn acquire_split(
        &self,
        source: &str,
        split_size: u64,
        cancel: &AtomicBool,
    ) -> Result<String, String> {
        let src_size = crate::block_device::device_size(source)?;
        let mut src = File::open(source).map_err(|e| format!("Cannot open {source}: {e}"))?;
        let has_known_size = src_size > 0;

        let stem = self
            .dest
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dir = self.dest.parent().unwrap_or(Path::new("."));

        let mut overall = Sha256::new();
        let mut total: u64 = 0;
        let mut part_num: u32 = 0;

        loop {
            if cancel.load(Ordering::SeqCst) {
                return Err("CANCELLED".into());
            }
            part_num += 1;
            let part_path = dir.join(format!("{stem}.{part_num:05}.aff4"));

            let part_bytes = self.write_part(
                &part_path,
                source,
                &mut src,
                split_size,
                cancel,
                Some(&mut overall),
            )?;

            total += part_bytes;

            let pct = if has_known_size && src_size > 0 {
                (total as f64 / src_size as f64) * 100.0
            } else {
                0.0
            };
            crate::progress::update_progress(
                pct,
                &format!(
                    "AFF4 part {part_num}: {} / {}",
                    crate::block_device::format_capacity(total),
                    if has_known_size {
                        crate::block_device::format_capacity(src_size)
                    } else {
                        "unknown".into()
                    }
                ),
                total,
                if has_known_size { src_size } else { 0 },
            );

            if part_bytes < split_size {
                break;
            }
            if has_known_size && total >= src_size {
                break;
            }
        }

        let hash = format!("{:x}", overall.finalize());
        crate::progress::finish_progress(Ok(hash.clone()));
        Ok(hash)
    }

    fn write_part(
        &self,
        dest: &Path,
        source: &str,
        reader: &mut File,
        max_bytes: u64,
        cancel: &AtomicBool,
        overall: Option<&mut Sha256>,
    ) -> Result<u64, String> {
        let (_, written) =
            self.acquire_single(dest, source, cancel, Some((reader, max_bytes)), overall)?;
        Ok(written)
    }

    /// Write one AFF4 container. If `limited` is Some, read at most `max` bytes from the shared reader.
    #[allow(unused_assignments)]
    fn acquire_single(
        &self,
        dest: &Path,
        source: &str,
        cancel: &AtomicBool,
        limited: Option<(&mut File, u64)>,
        mut overall: Option<&mut Sha256>,
    ) -> Result<(String, u64), String> {
        let using_shared = limited.is_some();
        let mut owned_file: Option<File>;
        let (reader, max_read) = if let Some((f, max)) = limited {
            owned_file = None;
            (f, max)
        } else {
            owned_file = Some(
                File::open(source).map_err(|e| format!("Cannot open {source}: {e}"))?,
            );
            (owned_file.as_mut().unwrap(), u64::MAX)
        };

        let src_size = if using_shared {
            max_read
        } else {
            crate::block_device::device_size(source)?
        };
        let has_known_size = src_size > 0 && src_size != u64::MAX;

        let file =
            File::create(dest).map_err(|e| format!("Cannot create {}: {e}", dest.display()))?;
        let mut zip = ZipWriter::new(file);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        let turtle = format!(
            "@prefix af: <http://aff4.org/aff4#> .\n\
             @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
             <> af:tool \"CollectionLoom\" .\n\
             <> af:stream <aff4:stream1> .\n\
             <aff4:stream1> af:path \"{STREAM_PATH}\" ;\n\
                 af:conventionalPath \"{source}\" .\n"
        );
        zip.start_file("information.turtle", opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(turtle.as_bytes()).map_err(|e| e.to_string())?;

        zip.start_file(STREAM_PATH, opts)
            .map_err(|e| e.to_string())?;

        let mut sha256 = Sha256::new();
        let mut buf = vec![0u8; crate::hashing::HASH_BUFFER_SIZE];
        let mut part_written: u64 = 0;

        loop {
            if cancel.load(Ordering::SeqCst) {
                let _ = std::fs::remove_file(dest);
                return Err("CANCELLED".into());
            }
            if part_written >= max_read {
                break;
            }
            let to_read = buf.len().min((max_read - part_written) as usize);
            let n = reader
                .read(&mut buf[..to_read])
                .map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            sha256.update(&buf[..n]);
            if let Some(overall) = overall.as_deref_mut() {
                overall.update(&buf[..n]);
            }
            zip.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            part_written += n as u64;

            if !using_shared {
                let pct = if has_known_size {
                    (part_written as f64 / src_size as f64) * 100.0
                } else {
                    0.0
                };
                crate::progress::update_progress(
                    pct,
                    &format!(
                        "AFF4 imaging: {} / {}",
                        crate::block_device::format_capacity(part_written),
                        if has_known_size {
                            crate::block_device::format_capacity(src_size)
                        } else {
                            "unknown".into()
                        }
                    ),
                    part_written,
                    if has_known_size { src_size } else { 0 },
                );
            }
        }

        let hash = format!("{:x}", sha256.finalize());
        let meta = format!(
            "format: aff4-l\nsha256: {hash}\nsize_bytes: {part_written}\nsource: {source}\n"
        );
        zip.start_file("container.metadata", opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(meta.as_bytes()).map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;

        if !using_shared {
            crate::progress::finish_progress(Ok(hash.clone()));
        }
        Ok((hash, part_written))
    }
}

pub fn aff4_path(destination: &Path) -> std::path::PathBuf {
    let s = destination.to_string_lossy();
    if s.ends_with(".aff4") {
        destination.to_path_buf()
    } else {
        destination.with_extension("aff4")
    }
}

/// Hash concatenated stream data from split AFF4 parts (not the ZIP containers).
pub fn hash_split_aff4_stream(base: &Path, part_count: u32) -> Result<String, String> {
    use std::io::Read;
    use zip::ZipArchive;

    let stem = base
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let dir = base.parent().unwrap_or(Path::new("."));
    let mut hasher = Sha256::new();
    for part in 1..=part_count {
        let path = dir.join(format!("{stem}.{part:05}.aff4"));
        let file = File::open(&path).map_err(|e| format!("Open {}: {e}", path.display()))?;
        let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
        let mut entry = archive
            .by_name(STREAM_PATH)
            .map_err(|e| format!("Missing stream in {}: {e}", path.display()))?;
        let mut buf = vec![0u8; crate::hashing::HASH_BUFFER_SIZE];
        loop {
            let n = entry.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Count split AFF4 parts on disk.
pub fn count_split_aff4_parts(base: &Path) -> u32 {
    let stem = base
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let dir = base.parent().unwrap_or(Path::new("."));
    let mut count = 0u32;
    loop {
        count += 1;
        let path = dir.join(format!("{stem}.{count:05}.aff4"));
        if !path.exists() {
            return count - 1;
        }
    }
}
