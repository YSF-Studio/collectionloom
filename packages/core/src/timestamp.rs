//! Signed timestamp tokens for chain-of-custody (local Ed25519; optional RFC 3161 TSA).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::crypto::{sign_data, verify_signature, KeypairStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampToken {
    pub signed_at: String,
    pub content_digest: String,
    pub signature_hex: String,
    pub public_key_hex: String,
    /// `local-ed25519` or `rfc3161-tsa`
    pub method: String,
    pub tsa_url: Option<String>,
}

fn digest_payload(payload: &[u8], signed_at: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hasher.update(signed_at.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Create a locally signed timestamp binding payload bytes to UTC time (offline-capable).
pub fn create_local_timestamp(
    payload: &[u8],
    private_key: &[u8],
    public_key: &[u8],
) -> Result<TimestampToken, String> {
    let signed_at = Utc::now().to_rfc3339();
    let content_digest = digest_payload(payload, &signed_at);
    let sign_input = format!("{content_digest}|{signed_at}");
    let signature = sign_data(private_key, sign_input.as_bytes())?;
    Ok(TimestampToken {
        signed_at,
        content_digest,
        signature_hex: hex_encode(&signature),
        public_key_hex: hex_encode(public_key),
        method: "local-ed25519".into(),
        tsa_url: None,
    })
}

pub fn verify_local_timestamp(token: &TimestampToken, payload: &[u8]) -> Result<bool, String> {
    let expected = digest_payload(payload, &token.signed_at);
    if expected != token.content_digest {
        return Ok(false);
    }
    let pk = hex_decode(&token.public_key_hex)?;
    let sig = hex_decode(&token.signature_hex)?;
    let sign_input = format!("{}|{}", token.content_digest, token.signed_at);
    verify_signature(&pk, sign_input.as_bytes(), &sig)
}

/// Optional RFC 3161 timestamp via HTTP TSA (requires network). Falls back to local on error.
pub async fn create_timestamp_with_optional_tsa(
    payload: &[u8],
    keypair: &KeypairStore,
    tsa_url: Option<&str>,
) -> Result<TimestampToken, String> {
    if let Some(url) = tsa_url.filter(|u| !u.is_empty()) {
        if let Ok(mut token) = request_rfc3161_tsa(url, payload).await {
            token.public_key_hex = hex_encode(&keypair.public_key);
            return Ok(token);
        }
    }
    create_local_timestamp(payload, &keypair.private_key, &keypair.public_key)
}

async fn request_rfc3161_tsa(url: &str, payload: &[u8]) -> Result<TimestampToken, String> {
    let digest = Sha256::digest(payload);
    let body = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, digest);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(url)
        .header("Content-Type", "application/timestamp-query")
        .body(body.into_bytes())
        .send()
        .await
        .map_err(|e| format!("TSA request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("TSA HTTP {}", resp.status()));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    Ok(TimestampToken {
        signed_at: Utc::now().to_rfc3339(),
        content_digest: format!("{:x}", digest),
        signature_hex: hex_encode(bytes.as_ref()),
        public_key_hex: String::new(),
        method: "rfc3161-tsa".into(),
        tsa_url: Some(url.to_string()),
    })
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("invalid hex".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| "invalid hex".into()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_timestamp_roundtrip() {
        let kp = KeypairStore::generate();
        let payload = b"evidence-123";
        let token = create_local_timestamp(payload, &kp.private_key, &kp.public_key).unwrap();
        assert!(verify_local_timestamp(&token, payload).unwrap());
    }
}
