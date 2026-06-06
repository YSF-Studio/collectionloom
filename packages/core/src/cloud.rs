use serde::Serialize;

use crate::aws_sigv4;

fn validate_aws_region(region: &str) -> Result<(), String> {
    let region = region.trim();
    if region.is_empty() || region.len() > 32 {
        return Err("Invalid AWS region".into());
    }
    if !region
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err("Invalid AWS region characters".into());
    }
    let parts: Vec<&str> = region.split('-').collect();
    if parts.len() < 3 || parts[0].len() != 2 {
        return Err("Invalid AWS region format".into());
    }
    Ok(())
}

fn validate_aws_credentials(access_key: &str, secret_key: &str) -> Result<(), String> {
    if access_key.is_empty() || access_key.len() > 128 || access_key.chars().any(|c| !c.is_ascii()) {
        return Err("Invalid AWS access key".into());
    }
    if secret_key.is_empty() || secret_key.len() > 128 || secret_key.chars().any(|c| !c.is_ascii()) {
        return Err("Invalid AWS secret key".into());
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudSnapshot {
    pub provider: String,
    pub region: String,
    pub volume_id: String,
    pub snapshot_id: Option<String>,
}

/// Create AWS EBS snapshot via EC2 Query API with Signature Version 4.
pub async fn aws_create_snapshot(
    region: &str,
    volume_id: &str,
    access_key: &str,
    secret_key: &str,
) -> Result<String, String> {
    validate_aws_region(region)?;
    validate_aws_credentials(access_key, secret_key)?;
    if !volume_id.starts_with("vol-") || volume_id.len() > 32 {
        return Err("Invalid AWS volume ID".into());
    }

    let host = format!("ec2.{region}.amazonaws.com");
    let params = [
        ("Action", "CreateSnapshot"),
        ("Description", "CollectionLoom forensic snapshot"),
        ("Version", "2016-11-15"),
        ("VolumeId", volume_id),
    ];
    let (authorization, amz_date, body) =
        aws_sigv4::sign_post_form(&host, region, "ec2", access_key, secret_key, &params)?;

    let endpoint = format!("https://{host}/");
    let client = reqwest::Client::new();
    let resp = client
        .post(&endpoint)
        .header("Authorization", authorization)
        .header("Content-Type", "application/x-www-form-urlencoded; charset=utf-8")
        .header("Host", &host)
        .header("x-amz-date", amz_date)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("AWS request failed: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("AWS EC2 error (HTTP {}): {}", status.as_u16(), text));
    }
    Ok(text)
}

/// Create Azure disk snapshot via REST API
pub async fn azure_create_snapshot(
    subscription: &str,
    resource_group: &str,
    disk_name: &str,
    snapshot_name: &str,
    token: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/snapshots/{}?api-version=2024-03-01",
        subscription, resource_group, snapshot_name
    );

    let body = serde_json::json!({
        "location": "eastus",
        "properties": {
            "creationData": {
                "createOption": "Copy",
                "sourceResourceId": format!("/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Compute/disks/{}", subscription, resource_group, disk_name)
            }
        }
    });

    let resp = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Azure request failed: {}", e))?;

    Ok(resp.text().await.map_err(|e| e.to_string())?)
}

/// Create GCP disk snapshot via REST API (no SDK)
pub async fn gcp_create_snapshot(
    project: &str,
    zone: &str,
    disk: &str,
    snapshot_name: &str,
    token: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/disks/{}/createSnapshot",
        project, zone, disk
    );

    let body = serde_json::json!({
        "name": snapshot_name,
        "description": "CollectionLoom forensic snapshot"
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("GCP request failed: {}", e))?;

    Ok(resp.text().await.map_err(|e| e.to_string())?)
}
