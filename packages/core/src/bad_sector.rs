//! Forensic-resilient block reads: skip bad sectors, zero-fill, and log.

use serde::Serialize;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub const DEFAULT_SECTOR_SIZE: u64 = 512;
const MAX_LOGGED_SECTORS: usize = 10_000;

#[derive(Debug, Clone, Default, Serialize)]
pub struct BadSectorEntry {
    pub sector: u64,
    pub byte_offset: u64,
    pub error: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BadSectorLog {
    pub error_sectors: u64,
    pub entries: Vec<BadSectorEntry>,
}

impl BadSectorLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, byte_offset: u64, sector_size: u64, err: &std::io::Error) {
        self.error_sectors += 1;
        if self.entries.len() < MAX_LOGGED_SECTORS {
            self.entries.push(BadSectorEntry {
                sector: byte_offset / sector_size.max(1),
                byte_offset,
                error: err.to_string(),
            });
        }
    }

    /// Append JSON-lines log beside the image destination.
    pub fn write_log_file(&self, image_path: &Path) -> Result<Option<std::path::PathBuf>, String> {
        if self.error_sectors == 0 {
            return Ok(None);
        }
        let log_path = image_path.with_extension("bad_sectors.log");
        let mut lines = String::new();
        for e in &self.entries {
            let line = serde_json::json!({
                "sector": e.sector,
                "byte_offset": e.byte_offset,
                "error": e.error,
            });
            lines.push_str(&line.to_string());
            lines.push('\n');
        }
        let header = format!(
            "# CollectionLoom bad sector log — {} sectors\n",
            self.error_sectors
        );
        std::fs::write(&log_path, format!("{header}{lines}")).map_err(|e| e.to_string())?;
        Ok(Some(log_path))
    }
}

/// Read up to `buf.len()` bytes at `byte_offset`, zero-filling unreadable sectors.
pub fn read_resilient(
    file: &mut File,
    buf: &mut [u8],
    byte_offset: u64,
    sector_size: u64,
    log: &mut BadSectorLog,
) -> Result<usize, String> {
    if buf.is_empty() {
        return Ok(0);
    }

    let ss = sector_size.max(1) as usize;

    match file.read(buf) {
        Ok(0) => return Ok(0),
        Ok(n) => return Ok(n),
        Err(first_err) => {
            file.seek(SeekFrom::Start(byte_offset))
                .map_err(|e| format!("Seek failed at {byte_offset}: {e}"))?;

            let mut pos = 0usize;
            while pos < buf.len() {
                let end = (pos + ss).min(buf.len());
                match file.read(&mut buf[pos..end]) {
                    Ok(0) => break,
                    Ok(n) => {
                        if n < end - pos {
                            buf[pos + n..end].fill(0);
                            log.record(byte_offset + pos as u64 + n as u64, sector_size, &first_err);
                        }
                        pos = end;
                    }
                    Err(e) => {
                        log.record(byte_offset + pos as u64, sector_size, &e);
                        buf[pos..end].fill(0);
                        pos = end;
                    }
                }
            }

            if pos == 0 {
                log.record(byte_offset, sector_size, &first_err);
                let fill = ss.min(buf.len());
                buf[..fill].fill(0);
                Ok(fill)
            } else {
                Ok(pos)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Error;

    #[test]
    fn bad_sector_log_counts_entries() {
        let mut log = BadSectorLog::new();
        let err = Error::new(std::io::ErrorKind::Other, "I/O error");
        log.record(1024, 512, &err);
        log.record(1536, 512, &err);
        assert_eq!(log.error_sectors, 2);
    }

    #[test]
    fn resilient_read_on_file_roundtrip() {
        let dir = std::env::temp_dir().join(format!("cl_bad_sector_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("disk.bin");
        std::fs::write(&path, vec![0xABu8; 2048]).unwrap();

        let mut file = File::open(&path).unwrap();
        let mut buf = [0u8; 1024];
        let mut log = BadSectorLog::new();
        let n = read_resilient(&mut file, &mut buf, 0, 512, &mut log).unwrap();
        assert_eq!(n, 1024);
        assert_eq!(log.error_sectors, 0);
        assert!(buf.iter().all(|&b| b == 0xAB));
    }
}
