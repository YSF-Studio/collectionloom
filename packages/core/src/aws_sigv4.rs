//! AWS Signature Version 4 for EC2 Query API (POST, form-urlencoded body).

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

const ALGORITHM: &str = "AWS4-HMAC-SHA256";

type HmacSha256 = Hmac<Sha256>;

fn sha256_hex(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

fn hmac_sha256(key: &[u8], data: &str) -> Result<Vec<u8>, String> {
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| "invalid HMAC key".to_string())?;
    mac.update(data.as_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

fn signing_key(secret_key: &str, date_stamp: &str, region: &str, service: &str) -> Result<Vec<u8>, String> {
    let k_date = hmac_sha256(format!("AWS4{secret_key}").as_bytes(), date_stamp)?;
    let k_region = hmac_sha256(&k_date, region)?;
    let k_service = hmac_sha256(&k_region, service)?;
    hmac_sha256(&k_service, "aws4_request")
}

fn aws_uri_encode(input: &str, encode_slash: bool) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b'/' if !encode_slash => out.push('/'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn form_body(params: &[(&str, &str)]) -> String {
    let mut sorted: Vec<_> = params.to_vec();
    sorted.sort_by_key(|(k, _)| *k);
    sorted
        .iter()
        .map(|(k, v)| format!("{}={}", aws_uri_encode(k, true), aws_uri_encode(v, true)))
        .collect::<Vec<_>>()
        .join("&")
}

/// Sign a POST request with `application/x-www-form-urlencoded` body.
pub fn sign_post_form(
    host: &str,
    region: &str,
    service: &str,
    access_key: &str,
    secret_key: &str,
    params: &[(&str, &str)],
) -> Result<(String, String, String), String> {
    let amz_date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = &amz_date[..8];
    let body = form_body(params);
    let payload_hash = sha256_hex(body.as_bytes());

    let canonical_headers = format!(
        "content-type:application/x-www-form-urlencoded; charset=utf-8\nhost:{host}\nx-amz-date:{amz_date}\n"
    );
    let signed_headers = "content-type;host;x-amz-date";
    let canonical_request = format!(
        "POST\n/\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
    );
    let credential_scope = format!("{date_stamp}/{region}/{service}/aws4_request");
    let string_to_sign = format!(
        "{ALGORITHM}\n{amz_date}\n{credential_scope}\n{}",
        sha256_hex(canonical_request.as_bytes())
    );

    let key = signing_key(secret_key, date_stamp, region, service)?;
    let signature = hmac_sha256(&key, &string_to_sign)?
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let authorization = format!(
        "{ALGORITHM} Credential={access_key}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}"
    );

    Ok((authorization, amz_date, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_body_sorts_params() {
        let body = form_body(&[("VolumeId", "vol-abc"), ("Action", "CreateSnapshot")]);
        assert!(body.starts_with("Action=CreateSnapshot"));
        assert!(body.contains("VolumeId=vol-abc"));
    }
}
