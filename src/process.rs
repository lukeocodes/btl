use anyhow::{Context, Result};
use nix::sys::signal;
use nix::unistd::Pid;

#[cfg(unix)]
pub fn get_current_uid() -> u32 {
    unsafe { libc::getuid() }
}

#[cfg(unix)]
pub fn is_process_alive(pid: u32) -> bool {
    // Signal 0 checks if process exists without killing it
    signal::kill(Pid::from_raw(pid as i32), None).is_ok()
}

#[cfg(unix)]
pub fn get_process_uid(pid: u32) -> Result<u32> {
    use std::fs;

    let status_path = format!("/proc/{}/status", pid);
    let content = fs::read_to_string(&status_path)
        .context("Failed to read process status")?;

    for line in content.lines() {
        if line.starts_with("Uid:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                return parts[1].parse()
                    .context("Failed to parse UID");
            }
        }
    }

    Err(anyhow::anyhow!("Could not find UID in process status"))
}

pub fn validate_pid(pid: u32, expected_uid: u32) -> bool {
    if !is_process_alive(pid) {
        return false;
    }

    // On Linux, verify UID matches
    #[cfg(target_os = "linux")]
    {
        match get_process_uid(pid) {
            Ok(uid) => uid == expected_uid,
            Err(_) => false,
        }
    }

    // On macOS, just trust the alive check (harder to get UID)
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}
