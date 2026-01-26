mod state;
mod hash;
mod process;
mod cli;

fn main() {
    let cli = cli::parse();

    match cli.command {
        Some(cmd) => {
            println!("Subcommand: {:?}", cmd);
        }
        None => {
            if cli.args.is_empty() {
                eprintln!("Error: No command provided");
                std::process::exit(1);
            }
            println!("Running: {:?}", cli.args);
        }
    }
}
