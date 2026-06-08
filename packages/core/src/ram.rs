use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct RamCaptureConfig {
    pub tool: RamCaptureTool,
    pub output_file: String,
    pub compress: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum RamCaptureTool {
    LiME,
    Avml,
    WinPmem,
}

/// Detect available RAM capture tools (./tools/ first, then PATH).
pub fn detect_tools() -> Vec<RamCaptureTool> {
    let tools = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if crate::portable::tool_available("avml") {
            tools.push(RamCaptureTool::Avml);
        }
        let lime_in_kit = crate::portable::tools_dir()
            .map(|t| t.join("lime").is_dir())
            .unwrap_or(false);
        if lime_in_kit || std::path::Path::new("/usr/lib/lime/lime.ko").exists() {
            tools.push(RamCaptureTool::LiME);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if crate::portable::tool_available("winpmem.exe")
            || crate::portable::tool_available("winpmem")
            || crate::portable::tool_available("WinPmem")
            || crate::portable::tool_available("winpmem_v4")
            || crate::portable::tool_available("winpmem_v4.exe")
        {
            tools.push(RamCaptureTool::WinPmem);
        }
        if crate::portable::tool_available("avml") {
            tools.push(RamCaptureTool::Avml);
        }
    }

    tools
}

/// Capture RAM using Avml (preferred — no kernel module needed)
pub fn capture_avml(output: &str, compress: bool) -> Result<String, String> {
    let mut cmd = crate::portable::command("avml")?;
    if compress { cmd.arg("-c"); }
    cmd.arg(output);

    let result = cmd.output().map_err(|e| format!("avml failed: {}. Is avml installed?", e))?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("avml capture failed: {}", stderr));
    }

    // Compute hash of captured file
    let path = std::path::Path::new(output);
    if path.exists() {
        super::hashing::multi_hash(path, &std::sync::atomic::AtomicBool::new(false))
            .map(|h| h.sha256.unwrap_or_default())
            .map_err(|e| e)
    } else {
        Err("Output file not found after capture".into())
    }
}

fn resolve_winpmem_command() -> Result<std::process::Command, String> {
    for name in ["winpmem.exe", "winpmem", "WinPmem", "winpmem_v4", "winpmem_v4.exe"] {
        if let Ok(cmd) = crate::portable::command(name) {
            return Ok(cmd);
        }
    }
    Err("WinPmem not found in ./tools/ or PATH".into())
}

/// Capture RAM using WinPmem (Windows)
pub fn capture_winpmem(output: &str) -> Result<String, String> {
    let status = resolve_winpmem_command()?
        .args(["-o", output])
        .status()
        .map_err(|e| format!("WinPmem failed: {}", e))?;

    if !status.success() {
        return Err("WinPmem capture failed".into());
    }
    Ok(format!("Captured to {}", output))
}

/// Get total system RAM size
pub fn get_ram_size() -> Result<u64, String> {
    #[cfg(target_os = "linux")]
    {
        let meminfo = std::fs::read_to_string("/proc/meminfo")
            .map_err(|e| e.to_string())?;
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                let kb: u64 = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                return Ok(kb * 1024);
            }
        }
        return Err("MemTotal not found in /proc/meminfo".into());
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl").args(["-n", "hw.memsize"]).output()
            .map_err(|e| e.to_string())?;
        let bytes: u64 = String::from_utf8_lossy(&output.stdout).trim().parse().unwrap_or(0);
        return Ok(bytes);
    }
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("wmic").args(["computersystem", "get", "totalphysicalmemory"]).output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let bytes: u64 = stdout.lines().nth(1).unwrap_or("0").trim().parse().unwrap_or(0);
        return Ok(bytes);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err("Cannot determine RAM size".into())
}
