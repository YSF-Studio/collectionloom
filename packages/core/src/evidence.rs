use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceId {
    pub case_initials: String,
    pub media_type: String,
    pub sequence: u16,
}

impl EvidenceId {
    /// Create evidence ID: `[CASE-INITIALS]-[MEDIA-TYPE]-[SEQUENCE]` e.g. `BR2026-DSK-001`.
    pub fn new(case_initials: &str, media_type: &str) -> Self {
        let counter_path = evidence_counter_path(case_initials, media_type);
        let sequence = match std::fs::read_to_string(&counter_path) {
            Ok(s) => {
                let current: u16 = s.trim().parse().unwrap_or(0);
                let next = current + 1;
                let _ = std::fs::create_dir_all(counter_path.parent().unwrap());
                let _ = std::fs::write(&counter_path, next.to_string());
                next
            }
            Err(_) => {
                let _ = std::fs::create_dir_all(counter_path.parent().unwrap());
                let _ = std::fs::write(&counter_path, "1");
                1
            }
        };
        Self {
            case_initials: case_initials.to_string(),
            media_type: media_type.to_uppercase(),
            sequence,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}-{}-{:03}",
            self.case_initials, self.media_type, self.sequence
        )
    }
}

fn evidence_counter_path(case_initials: &str, media_type: &str) -> PathBuf {
    dirs_next()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ysf")
        .join(format!(
            "evidence_{}_{}.counter",
            case_initials.to_uppercase(),
            media_type.to_uppercase()
        ))
}

fn derive_case_initials(case_name: &str) -> String {
    let year = Utc::now().format("%Y").to_string();
    let letters: String = case_name
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    if letters.len() >= 2 {
        format!("{letters}{year}")
    } else {
        format!("CL{year}")
    }
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
}

/// Single action in the chain of custody log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLog {
    pub timestamp: String,
    pub operator: String,
    pub action: String,
    pub details: String,
    pub hash: Option<String>,
}

/// Complete chain of custody for an evidence collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfCustody {
    pub evidence_id: String,
    pub case_name: String,
    pub operator: String,
    pub source_device: String,
    pub source_size_bytes: u64,
    pub actions: Vec<ActionLog>,
    pub final_hashes: Option<super::hashing::HashSet>,
    pub signature: Option<Vec<u8>>,
}

impl ChainOfCustody {
    pub fn new(case_name: &str, operator: &str, source_device: &str, source_size: u64) -> Self {
        Self::with_media_type(case_name, operator, source_device, source_size, "DSK")
    }

    pub fn with_media_type(
        case_name: &str,
        operator: &str,
        source_device: &str,
        source_size: u64,
        media_type: &str,
    ) -> Self {
        let initials = derive_case_initials(case_name);
        let eid = EvidenceId::new(&initials, media_type);
        Self {
            evidence_id: eid.to_string(),
            case_name: case_name.to_string(),
            operator: operator.to_string(),
            source_device: source_device.to_string(),
            source_size_bytes: source_size,
            actions: vec![],
            final_hashes: None,
            signature: None,
        }
    }

    pub fn add_action(&mut self, action: &str, details: &str, hash: Option<&str>) {
        self.actions.push(ActionLog {
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            operator: self.operator.clone(),
            action: action.to_string(),
            details: details.to_string(),
            hash: hash.map(|h| h.to_string()),
        });
    }

    pub fn set_final_hashes(&mut self, hashes: super::hashing::HashSet) {
        self.final_hashes = Some(hashes);
    }

    pub fn sign(&mut self, private_key: &[u8]) -> Result<(), String> {
        let data = serde_json::to_string(&self).map_err(|e| e.to_string())?;
        let sig = super::crypto::sign_data(private_key, data.as_bytes())?;
        self.signature = Some(sig);
        Ok(())
    }
}

/// Generate QR code PNG for evidence label (scannable, ISO 27037 §7.1).
pub fn generate_qr_label(
    evidence_id: &str,
    device: &str,
    case: &str,
    operator: Option<&str>,
    acquired_at: Option<&str>,
    hash_sha256: Option<&str>,
) -> Vec<u8> {
    use image::{ImageBuffer, Rgb, RgbImage};
    use qrcode::QrCode;

    let mut text = format!("EID:{evidence_id}\nDEV:{device}\nCASE:{case}");
    if let Some(op) = operator.filter(|s| !s.is_empty()) {
        text.push_str(&format!("\nOP:{op}"));
    }
    if let Some(at) = acquired_at.filter(|s| !s.is_empty()) {
        text.push_str(&format!("\nAT:{at}"));
    }
    if let Some(h) = hash_sha256.filter(|s| !s.is_empty()) {
        text.push_str(&format!("\nSHA256:{h}"));
    }
    let code = QrCode::new(text.as_bytes()).expect("valid QR payload");
    let modules = code.width() as u32;
    let scale = 8u32;
    let quiet = 4u32;
    let size = modules * scale + quiet * 2;
    let mut img: RgbImage = ImageBuffer::new(size, size);

    for y in 0..modules {
        for x in 0..modules {
            let dark = code[(x as usize, y as usize)] == qrcode::Color::Dark;
            let px = if dark {
                Rgb([0u8, 0u8, 0u8])
            } else {
                Rgb([255u8, 255u8, 255u8])
            };
            for dy in 0..scale {
                for dx in 0..scale {
                    img.put_pixel(quiet + x * scale + dx, quiet + y * scale + dy, px);
                }
            }
        }
    }

    let mut buf = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buf),
        image::ImageFormat::Png,
    )
    .expect("PNG encode");
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_id_format_example() {
        let unique = format!("XY{}", std::process::id());
        let eid = EvidenceId::new(&unique, "DSK");
        let s = eid.to_string();
        assert!(s.starts_with(&format!("{unique}-DSK-")), "Got: {s}");
        assert!(s.ends_with("-001"), "Expected first sequence 001, got: {s}");
    }
}
