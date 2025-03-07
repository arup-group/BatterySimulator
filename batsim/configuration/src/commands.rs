use anyhow::{Context, Result};
use clap::Parser;
use serde_json;
use std::{
    fs::{File, OpenOptions},
    io::BufReader,
    path::PathBuf,
    time::Instant,
};

use crate::{
    config::{charge_plan::ChargePlanner, config::Config, handler::ConfigHandler, sampler, groups::activity},
    tracer::Population,
};

// Entry point for `optmimise` CLI command.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct ConfigCommand {
    /// Config path
    #[clap(short, long)]
    config: Option<PathBuf>,
    /// Path to traces file
    #[clap(short, long, default_value = "traces.json")]
    trace_path: PathBuf,
    /// Output file path
    #[clap(short, long, default_value = "config.csv")]
    output: PathBuf,
}

impl ConfigCommand {
    pub fn run(&self) -> Result<()> {
        // Load config
        let config = match &self.config {
            Some(path) => Config::load(path),
            None => {
                warn!("Using default configuration");
                Ok(Config::default())
            }
        }?;

        let handler = ConfigHandler::new(&config);

        // Load samplers
        let mut rng = sampler::new(config.seed);

        // Load traces
        let parsing_started = Instant::now();
        let file = File::open(&self.trace_path).context(format!(
            "unable to open file '{}'",
            self.trace_path.display()
        ))?;
        let reader = BufReader::new(file);
        let population: Population =
            serde_json::from_reader(reader).context("unable to read traces")?;
        info!(
            "{} traces loaded in ==> {} ms",
            population.len(),
            parsing_started.elapsed().as_millis()
        );

        let out_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.output)
            .expect("Unable to create out file");
        let mut writer = csv::Writer::from_writer(out_file);
        writer
            .write_record(&["pid", "battery", "en_route", "activities"])
            .unwrap();

        for (pid, person) in population.into_iter() {
            let (battery_spec, en_route_spec, activity_specs) =
                handler.get(pid, person, &mut rng)?;
            let battery_spec_name: &str = battery_spec.name.as_deref().unwrap_or("None");
            let en_route_name: &str = en_route_spec.name.as_deref().unwrap_or("None");
            let activity_names =
                &ChargePlanner::new(activity_specs).name();

            writer
                .write_record(&[
                    pid,
                    battery_spec_name,
                    en_route_name,
                    activity_names,
                ])
                .context(format!("failed to write for pid: '{}'", pid))?;
        }
        writer.flush().unwrap();
        Ok(())
    }
}
