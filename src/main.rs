mod state;
mod hash;
mod process;
mod cli;

use anyhow::Result;
use chrono::{Utc, Local};
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::Command as StdCommand;

fn main() -> Result<()> {
    // Refuse to run as root
    if process::get_current_uid() == 0 {
        eprintln!("Error: btl refuses to run as root for safety");
        eprintln!("Run as a regular user instead");
        std::process::exit(1);
    }

    let cli = cli::parse();

    match cli.command {
        Some(cmd) => handle_subcommand(cmd)?,
        None => handle_background_command(&cli.args)?,
    }

    Ok(())
}

fn handle_subcommand(cmd: cli::Commands) -> Result<()> {
    match cmd {
        cli::Commands::List => handle_list()?,
        cli::Commands::Clean { all } => handle_clean(all)?,
        cli::Commands::Kill { hash, all } => handle_kill(hash, all)?,
        cli::Commands::Logs { hash } => handle_logs(hash)?,
    }
    Ok(())
}

fn handle_background_command(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!("Error: No command provided");
        std::process::exit(1);
    }

    // Ensure directories exist
    state::ensure_dirs()?;

    // Generate hash for this command
    let hash = hash::generate_hash(args)?;
    let log_file = hash::get_log_path(&hash)?;

    // Load state
    let state_path = state::get_state_path()?;
    let mut btl_state = state::load_state(&state_path)?;

    // Clean stale entries
    state::validate_and_clean_state(&mut btl_state)?;

    // Check if process already exists
    if let Some(existing) = btl_state.processes.get(&hash) {
        if process::validate_pid(existing.pid, existing.user_id) {
            println!("[btl] Killing previous process (PID {})", existing.pid);
            if let Err(e) = process::kill_process_gracefully(existing.pid) {
                eprintln!("[btl] Warning: Failed to kill process: {}", e);
            }
        }
    }

    // Spawn new process
    let command = &args[0];
    let cmd_args = &args[1..];

    println!("[btl] Starting: {}", args.join(" "));
    let pid = process::spawn_background_process(command, cmd_args, &log_file)?;

    // Update state
    let proc_state = state::ProcessState {
        pid,
        user_id: process::get_current_uid(),
        command: args.join(" "),
        cwd: std::env::current_dir()?,
        started_at: Utc::now(),
        log_file: log_file.clone(),
    };

    btl_state.processes.insert(hash.clone(), proc_state);
    state::save_state(&state_path, &btl_state)?;

    println!("[btl] Process backgrounded (PID {})", pid);
    println!("[btl] Hash: {}", hash);
    println!("[btl] Logs: {}", log_file.display());

    Ok(())
}

fn handle_list() -> Result<()> {
    state::ensure_dirs()?;

    let state_path = state::get_state_path()?;
    let mut btl_state = state::load_state(&state_path)?;

    // Clean stale entries
    state::validate_and_clean_state(&mut btl_state)?;
    state::save_state(&state_path, &btl_state)?;

    if btl_state.processes.is_empty() {
        println!("No background processes tracked");
        return Ok(());
    }

    println!("{:<16} {:<8} {:<20} {:<30} {}",
        "HASH", "PID", "STARTED", "COMMAND", "CWD");

    for (hash, proc) in &btl_state.processes {
        let duration = Local::now().signed_duration_since(proc.started_at);
        let started = if duration.num_hours() > 0 {
            format!("{}h ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{}m ago", duration.num_minutes())
        } else {
            format!("{}s ago", duration.num_seconds())
        };

        let command_display = if proc.command.len() > 28 {
            format!("{}...", &proc.command[..28])
        } else {
            proc.command.clone()
        };

        println!("{:<16} {:<8} {:<20} {:<30} {}",
            hash, proc.pid, started, command_display, proc.cwd.display());
    }

    Ok(())
}

fn handle_clean(all: bool) -> Result<()> {
    state::ensure_dirs()?;

    let state_path = state::get_state_path()?;
    let mut btl_state = state::load_state(&state_path)?;

    if all {
        let count = btl_state.processes.len();
        btl_state.processes.clear();
        println!("[btl] Removed {} entries", count);
    } else {
        let before = btl_state.processes.len();
        state::validate_and_clean_state(&mut btl_state)?;
        let after = btl_state.processes.len();
        let removed = before - after;

        if removed > 0 {
            println!("[btl] Removed {} dead process entries", removed);
        } else {
            println!("[btl] No dead processes found");
        }
    }

    state::save_state(&state_path, &btl_state)?;
    Ok(())
}

fn handle_kill(hash: Option<String>, all: bool) -> Result<()> {
    state::ensure_dirs()?;

    let state_path = state::get_state_path()?;
    let mut btl_state = state::load_state(&state_path)?;

    if all {
        let count = btl_state.processes.len();
        for (h, proc) in &btl_state.processes {
            println!("[btl] Killing process {} (PID {})", h, proc.pid);
            if let Err(e) = process::kill_process_gracefully(proc.pid) {
                eprintln!("[btl] Warning: Failed to kill {}: {}", h, e);
            }
        }
        btl_state.processes.clear();
        println!("[btl] Killed {} processes", count);
    } else if let Some(h) = hash {
        if let Some(proc) = btl_state.processes.remove(&h) {
            println!("[btl] Killing process {} (PID {})", h, proc.pid);
            process::kill_process_gracefully(proc.pid)?;
            println!("[btl] Process killed");
        } else {
            eprintln!("Error: Process {} not found", h);
            std::process::exit(1);
        }
    } else {
        eprintln!("Error: Provide a hash or --all flag");
        std::process::exit(1);
    }

    state::save_state(&state_path, &btl_state)?;
    Ok(())
}

fn handle_logs(hash: Option<String>) -> Result<()> {
    state::ensure_dirs()?;

    let state_path = state::get_state_path()?;
    let btl_state = state::load_state(&state_path)?;

    if let Some(h) = hash {
        if let Some(proc) = btl_state.processes.get(&h) {
            println!("[btl] Tailing {}", proc.log_file.display());
            println!("---");

            // Use tail -f if available, otherwise read file
            let tail_result = StdCommand::new("tail")
                .arg("-f")
                .arg(&proc.log_file)
                .status();

            if tail_result.is_err() {
                // Fallback: just read the file
                let file = fs::File::open(&proc.log_file)?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    println!("{}", line?);
                }
            }
        } else {
            eprintln!("Error: Process {} not found", h);
            std::process::exit(1);
        }
    } else {
        // Show all available logs
        println!("Available logs:");
        for (h, proc) in &btl_state.processes {
            println!("  {} -> {}", h, proc.log_file.display());
        }
    }

    Ok(())
}
