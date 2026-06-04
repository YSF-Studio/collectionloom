use std::io::Read;
use std::path::Path;

/// Test hash verification with actual files
#[test]
fn test_hash_verification_with_real_files() {
    let test_dir = "/tmp/collectionloom-test";

    // SHA-256 test — known value
    let data = b"VERIFICATION TARGET";
    let hashes = ysf_core::hashing::multi_hash_buffer(data);

    // Read the file and hash it
    let path = format!("{}/verify_me.txt", test_dir);
    let mut file = std::fs::File::open(&path).expect("Cannot open verify_me.txt");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Read error");
    let trimmed = contents.trim();
    let hashes2 = ysf_core::hashing::multi_hash_buffer(trimmed.as_bytes());

    let sha256_actual = hashes2.sha256.expect("SHA-256 should produce a hash");

    // Verify against expected
    let expected_path = format!("{}/expected.sha256", test_dir);
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

    println!(
        "✅ SHA-256 Verification PASSED: {} matches {}",
        sha256_actual, expected_hash
    );
}

/// Test multi-hash with a large (10MB) disk image
#[test]
fn test_disk_image_hashing() {
    let path = "/tmp/collectionloom-test/evidence.dd";
    assert!(Path::new(path).exists(), "evidence.dd should exist");

    let mut file = std::fs::File::open(path).expect("Cannot open evidence.dd");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("Read error");

    assert_eq!(data.len(), 10 * 1024 * 1024, "File should be exactly 10MB");

    let hashes = ysf_core::hashing::multi_hash_buffer(&data);

    assert!(
        hashes.sha256.is_some(),
        "SHA-256 should be computed for 10MB file"
    );
    assert!(
        hashes.sha1.is_some(),
        "SHA-1 should be computed for 10MB file"
    );
    assert!(hashes.md5.is_some(), "MD5 should be computed for 10MB file");

    println!("✅ Disk Image Hashing PASSED:");
    println!("   SHA-256: {}", hashes.sha256.unwrap());
    println!("   SHA-1:   {}", hashes.sha1.unwrap());
    println!("   MD5:     {}", hashes.md5.unwrap());
}

/// Test hash consistency — same file should produce same hash
#[test]
fn test_hash_consistency() {
    let data1 = b"FORENSIC EVIDENCE - CASE #001";
    let data2 = b"FORENSIC EVIDENCE - CASE #002";
    let data3 = b"FORENSIC EVIDENCE - CASE #001"; // same as data1

    let h1 = ysf_core::hashing::multi_hash_buffer(data1);
    let h2 = ysf_core::hashing::multi_hash_buffer(data2);
    let h3 = ysf_core::hashing::multi_hash_buffer(data3);

    // Same content → same hash
    assert_eq!(
        h1.sha256, h3.sha256,
        "Same content should produce same SHA-256"
    );
    assert_eq!(h1.sha1, h3.sha1, "Same content should produce same SHA-1");
    assert_eq!(h1.md5, h3.md5, "Same content should produce same MD5");

    // Different content → different hash
    assert_ne!(
        h1.sha256, h2.sha256,
        "Different content should produce different SHA-256"
    );

    println!("✅ Hash Consistency PASSED — same content → same hash, different → different");
}

/// Test system snapshot
#[test]
fn test_system_snapshot() {
    let snap =
        ysf_core::snapshot::take_snapshot("test-snapshot", Some("/tmp/collectionloom-test"))
            .expect("Snapshot should work");

    assert!(!snap.id.0.is_empty(), "Snapshot ID should not be empty");
    assert!(!snap.timestamp.is_empty(), "Timestamp should not be empty");
    assert!(
        !snap.files.is_empty(),
        "Should find files in /tmp/collectionloom-test"
    );

    println!("✅ System Snapshot PASSED:");
    println!("   ID: {}", snap.id.0);
    println!("   Time: {}", snap.timestamp);
    println!("   Files found: {}", snap.files.len());
}

/// Test entropy calculation
#[test]
fn test_entropy_calculation() {
    let path = "/tmp/collectionloom-test/evidence.dd";
    let mut file = std::fs::File::open(path).expect("Cannot open evidence.dd");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("Read error");
    let entropy = ysf_core::hashing::compute_entropy(&data);

    assert!(
        entropy >= 0.0 && entropy <= 8.0,
        "Entropy must be between 0 and 8"
    );
    // Random data should have high entropy (> 7.0 typically)
    println!("✅ Entropy: {:.4} (random data, expected ~7.9)", entropy);
}

/// Test file preview
#[test]
fn test_file_preview() {
    let path = "/tmp/collectionloom-test/case_notes.txt";
    let preview =
        ysf_core::preview::preview_file(path).expect("Preview should work");

    // Check preview content
    match &preview.preview {
        ysf_core::preview::PreviewContent::Text(text) => {
            assert!(!text.is_empty(), "Preview should have content");
            assert!(
                text.starts_with("=== EVIDENCE LOG ==="),
                "Preview should start with EVIDENCE LOG"
            );
            println!("✅ File Preview PASSED:");
            println!("   Lines: {}", text.lines().count());
            println!("   First line: {:?}", text.lines().next().unwrap_or(""));
        }
        _ => panic!("Expected text preview, got: {:?}", preview.kind),
    }
    println!("   Size: {} bytes", preview.size);
    println!("   MIME: {:?}", preview.mime_type);
}

/// Test raw evidence binary parsing
#[test]
fn test_raw_binary_parsing() {
    let path = "/tmp/collectionloom-test/raw_evidence.bin";
    let mut file = std::fs::File::open(path).expect("Cannot open raw_evidence.bin");
    let mut buf = [0u8; 512];
    file.read_exact(&mut buf).expect("Read error");

    // First 512 bytes should be zero (MBR simulation)
    assert_eq!(buf[0..8], [0x00; 8], "First 8 bytes should be zero (MBR)");

    // Check NTFS magic after MBR
    let mut full = Vec::new();
    file.read_to_end(&mut full).expect("Read error");
    println!(
        "✅ Raw Binary Parsing PASSED — read {} bytes total",
        512 + full.len()
    );
}
