mod state;
mod hash;
mod process;
mod cli;

use anyhow::Result;
use chrono::Utc;

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
    println!("Subcommand not yet implemented");
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
