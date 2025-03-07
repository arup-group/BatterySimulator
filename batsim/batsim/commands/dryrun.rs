use anyhow::{Context, Result};
use clap::Parser;
use indicatif::HumanCount;
use std::{fs::File, io::BufReader, path::PathBuf};

use configuration::{config::Config, handler::AgentConfig, sampler};
use tracer::Population;

use crate::utils;

// Entry point for `optmimise` CLI command.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct DryrunCommand {
    /// Config path
    #[clap(short, long)]
    config: Option<PathBuf>,
    /// Path to traces file
    #[clap(short, long, default_value = "traces.trc")]
    trace_path: PathBuf,
    /// Output file path
    #[clap(short, long, default_value = "config.csv")]
    output: PathBuf,
    /// Read traces from human readable json format
    #[arg(short, long)]
    json: bool,
}

impl DryrunCommand {
    pub fn run(&self) -> Result<()> {
        // Load config
        let config = match &self.config {
            Some(path) => Config::load(path),
            None => Ok(Config::default()),
        }?;
        config.valid()?;

        // Prepare input files
        let traces_file = File::open(&self.trace_path).context(format!(
            "unable to open file '{}'",
            self.trace_path.display()
        ))?;
        let traces_reader = BufReader::new(traces_file);

        // Prepare output paths
        let out_file = File::create(&self.output).expect("Unable to create out file");

        // Prepare output files
        let mut specs_writer = csv::Writer::from_writer(out_file);

        //Rng
        let mut rng = sampler::new(config.seed);

        // Load population
        let spinner = utils::default_spinner();
        spinner.set_message(format!(
            "[1/2] Loading traces from {}...",
            &self.trace_path.display()
        ));
        let population: Population = Population::deserialise(traces_reader, self.json)?;
        spinner.finish_with_message(format!(
            "[1/2] Completed loading traces ({} persons/plans)",
            HumanCount(population.len() as u64)
        ));

        // Write Agent Configurations
        let progress_bar = utils::default_progress_bar(population.len() as u64);
        progress_bar.set_message(format!(
            "[2/2] Writing configurations to '{}'...",
            &self.output.display()
        ));
        for (pid, person) in population.into_iter() {
            progress_bar.inc(1);
            let agent_config = AgentConfig::build(&config, pid, person, &mut rng);
            specs_writer
                .serialize(agent_config.to_record())
                .context(format!("failed to write specs for pid: '{}'", pid))?;
            agent_config.validate()?;
        }
        specs_writer.flush()?;
        progress_bar.finish_with_message(format!(
            "[2/2] Completed writing results to '{}'",
            self.output.display()
        ));
        Ok(())
    }
}
