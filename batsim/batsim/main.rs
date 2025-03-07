mod commands;
mod utils;
pub use commands::dryrun::DryrunCommand;
pub use commands::optimise::OptimiseCommand;
pub use commands::peek::PeekCommand;
pub use commands::run::RunCommand;
pub use commands::trace::TracerCommand;

use clap::{Parser, Subcommand};

fn main() {
    let cli = Cli::parse();
    if let Err(err) = match &cli.command {
        Commands::Run(run_command) => run_command.run(),
        Commands::Tracer(tracer_command) => tracer_command.run(),
        Commands::Optimise(simulation_command) => simulation_command.run(),
        Commands::Dryrun(config_command) => config_command.run(),
        Commands::Attributes(peek_command) => peek_command.run(),
    } {
        eprintln!("Error: {:?}", err);
        std::process::exit(1);
    };
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full batsim pipeline
    Run(commands::RunCommand),
    /// Pre-process MATSim outputs into traces
    Tracer(commands::TracerCommand),
    /// Calculate optimal charge events from given traces
    Optimise(commands::OptimiseCommand),
    /// Dry run agent configurations
    Dryrun(commands::DryrunCommand),
    /// Peek attributes in a plans file
    Attributes(commands::PeekCommand),
}
