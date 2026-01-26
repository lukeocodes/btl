use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "btl",
    about = "Bottle - Background task manager that auto-replaces processes",
    long_about = "btl (pronounced 'bottle') backgrounds processes and auto-replaces them on re-run"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Command to run in background (if no subcommand provided)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all tracked background processes
    List,

    /// Clean dead process entries
    Clean {
        /// Remove all entries (not just dead ones)
        #[arg(long)]
        all: bool,
    },

    /// Show or tail logs for a process
    Logs {
        /// Hash of the process (omit to see all)
        hash: Option<String>,
    },

    /// Manually kill a background process
    Kill {
        /// Hash of the process to kill
        hash: Option<String>,

        /// Kill all tracked processes
        #[arg(long)]
        all: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
