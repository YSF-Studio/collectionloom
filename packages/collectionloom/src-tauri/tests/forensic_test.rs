use std::io::Read;
use std::path::{Path, PathBuf};

fn test_dir() -> PathBuf {
    std::env::var("COLLECTIONLOOM_TEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../test-fixtures")
        })
}

fn ensure_binary_fixtures(dir: &Path) {
    let evidence = dir.join("evidence.dd");
    if !evidence.exists() {
        let mut data = vec![0u8; 10 * 1024 * 1024];
        for (i, b) in data.iter_mut().enumerate() {
            *b = ((i * 7919 + 104729) & 0xff) as u8;
        }
        std::fs::write(&evidence, &data).expect("write evidence.dd");
    }
    let raw = dir.join("raw_evidence.bin");
    if !raw.exists() {
        std::fs::write(&raw, vec![0u8; 4096]).expect("write raw_evidence.bin");
    }
}

#[test]
fn test_hash_verification_with_real_files() {
    let test_dir = test_dir();
    ensure_binary_fixtures(&test_dir);

    let path = test_dir.join("verify_me.txt");
    let mut file = std::fs::File::open(&path).expect("Cannot open verify_me.txt");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Read error");
    let trimmed = contents.trim();
    let hashes2 = ysf_core::hashing::multi_hash_buffer(trimmed.as_bytes());

    let sha256_actual = hashes2.sha256.expect("SHA-256 should produce a hash");

    let expected_path = test_dir.join("expected.sha256");
    let mut expected_file =
        std::fs::File::open(&expected_path).expect("Cannot open expected.sha256");
    let mut expected_content = String::new();
    expected_file
        .read_to_string(&mut expected_content)
        .expect("Read error");
    let expected_hash = expected_content
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();

    assert_eq!(
        sha256_actual.to_lowercase(),
        expected_hash,
        "SHA-256 hash should match expected value. Got: {}, Expected: {}",
        sha256_actual,
        expected_hash
    );
}

#[test]
fn test_disk_image_hashing() {
    let test_dir = test_dir();
    ensure_binary_fixtures(&test_dir);
    let path = test_dir.join("evidence.dd");
    assert!(path.exists(), "evidence.dd should exist");

    let mut file = std::fs::File::open(&path).expect("Cannot open evidence.dd");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("Read error");

    assert_eq!(data.len(), 10 * 1024 * 1024, "File should be exactly 10MB");

    let hashes = ysf_core::hashing::multi_hash_buffer(&data);

    assert!(hashes.sha256.is_some());
    assert!(hashes.sha1.is_some());
    assert!(hashes.md5.is_some());
}

#[test]
fn test_hash_consistency() {
    let data1 = b"FORENSIC EVIDENCE - CASE #001";
    let data2 = b"FORENSIC EVIDENCE - CASE #002";
    let data3 = b"FORENSIC EVIDENCE - CASE #001";

    let h1 = ysf_core::hashing::multi_hash_buffer(data1);
    let h2 = ysf_core::hashing::multi_hash_buffer(data2);
    let h3 = ysf_core::hashing::multi_hash_buffer(data3);

    assert_eq!(h1.sha256, h3.sha256);
    assert_eq!(h1.sha1, h3.sha1);
    assert_eq!(h1.md5, h3.md5);
    assert_ne!(h1.sha256, h2.sha256);
}

#[test]
fn test_system_snapshot() {
    let test_dir = test_dir();
    ensure_binary_fixtures(&test_dir);
    let dir_str = test_dir.to_string_lossy().to_string();

    let snap = ysf_core::snapshot::take_snapshot("test-snapshot", Some(&dir_str))
        .expect("Snapshot should work");

    assert!(!snap.id.0.is_empty());
    assert!(!snap.timestamp.is_empty());
    assert!(!snap.files.is_empty());
}

#[test]
fn test_entropy_calculation() {
    let test_dir = test_dir();
    ensure_binary_fixtures(&test_dir);
    let path = test_dir.join("evidence.dd");
    let mut file = std::fs::File::open(&path).expect("Cannot open evidence.dd");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("Read error");
    let entropy = ysf_core::hashing::compute_entropy(&data);

    assert!(entropy >= 0.0 && entropy <= 8.0);
}

#[test]
fn test_file_preview() {
    let test_dir = test_dir();
    let path = test_dir.join("case_notes.txt");
    let preview = ysf_core::preview::preview_file(path.to_str().unwrap()).expect("Preview should work");

    match &preview.preview {
        ysf_core::preview::PreviewContent::Text(text) => {
            assert!(!text.is_empty());
            assert!(text.starts_with("=== EVIDENCE LOG ==="));
        }
        _ => panic!("Expected text preview"),
    }
}

#[test]
fn test_raw_binary_parsing() {
    let test_dir = test_dir();
    ensure_binary_fixtures(&test_dir);
    let path = test_dir.join("raw_evidence.bin");
    let mut file = std::fs::File::open(&path).expect("Cannot open raw_evidence.bin");
    let mut buf = [0u8; 512];
    file.read_exact(&mut buf).expect("Read error");

    assert_eq!(buf[0..8], [0x00; 8], "First 8 bytes should be zero (MBR)");
}
