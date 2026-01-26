use anyhow::{Context, Result};
use nix::sys::signal;
use nix::unistd::Pid;
use std::fs::{File, OpenOptions};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

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

pub fn kill_process_gracefully(pid: u32) -> Result<()> {
    let pid = Pid::from_raw(pid as i32);

    // Try SIGTERM first
    if let Err(e) = signal::kill(pid, signal::Signal::SIGTERM) {
        return Err(anyhow::anyhow!("Failed to send SIGTERM: {}", e));
    }

    // Wait up to 3 seconds for graceful shutdown
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(100));
        if signal::kill(pid, None).is_err() {
            return Ok(());
        }
    }

    // Still alive, force kill with SIGKILL
    eprintln!("[btl] Process {} did not respond to SIGTERM, sending SIGKILL", pid);
    signal::kill(pid, signal::Signal::SIGKILL)
        .context("Failed to send SIGKILL")?;

    // Wait up to 1 second for final cleanup
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(100));
        if signal::kill(pid, None).is_err() {
            return Ok(());
        }
    }

    eprintln!("[btl] Warning: Process {} may still be running", pid);
    Ok(())
}

pub fn spawn_background_process(
    command: &str,
    args: &[String],
    log_file: &Path,
) -> Result<u32> {
    let mut cmd = Command::new(command);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            File::create(log_file)
                .context("Failed to create log file")?
        ))
        .stderr(Stdio::from(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .context("Failed to open log file for stderr")?
        ));

    // Detach from parent process group
    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| {
            nix::unistd::setsid()
                .map(|_| ())
                .map_err(|e| std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("setsid failed: {}", e)
                ))
        });
    }

    let child = cmd.spawn()
        .context("Failed to spawn process")?;

    Ok(child.id())
}
