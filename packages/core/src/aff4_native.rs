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
        Self { dest: dest.to_path_buf() }
    }

    pub fn acquire(
        &self,
        source: &str,
        cancel: &AtomicBool,
    ) -> Result<String, String> {
        let src_size = crate::block_device::device_size(source)?;
        let mut src = File::open(source).map_err(|e| format!("Cannot open {source}: {e}"))?;
        let has_known_size = src_size > 0;

        let file = File::create(&self.dest)
            .map_err(|e| format!("Cannot create {}: {e}", self.dest.display()))?;
        let mut zip = ZipWriter::new(file);
        let opts = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

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
        let mut total: u64 = 0;

        loop {
            if cancel.load(Ordering::SeqCst) {
                let _ = std::fs::remove_file(&self.dest);
                return Err("CANCELLED".into());
            }
            let n = src.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            sha256.update(&buf[..n]);
            zip.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            total += n as u64;
            let pct = if has_known_size {
                (total as f64 / src_size as f64) * 100.0
            } else {
                0.0
            };
            crate::progress::update_progress(
                pct,
                &format!(
                    "AFF4 imaging: {} / {}",
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
        }

        let hash = format!("{:x}", sha256.finalize());
        let meta = format!(
            "format: aff4-l\nsha256: {hash}\nsize_bytes: {total}\nsource: {source}\n"
        );
        zip.start_file("container.metadata", opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(meta.as_bytes()).map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;
        crate::progress::finish_progress(Ok(hash.clone()));
        Ok(hash)
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
