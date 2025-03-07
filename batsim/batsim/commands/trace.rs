use anyhow::{Context, Result};
use clap::Parser;
use indicatif::HumanCount;
use std::{fs::File, path::PathBuf};

use crate::utils;
use tracer::{self, Network, Population};
use xml;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct TracerCommand {
    /// MATSim output directory
    #[clap(short, long, default_value = "tests/data")]
    dir: PathBuf,
    /// Name of network file
    #[clap(short, long, default_value = "output_network.xml")]
    network: PathBuf,
    /// Name of plans file
    #[clap(short, long, default_value = "output_plans.xml")]
    population: PathBuf,
    /// Name of events file
    #[clap(short, long, default_value = "output_events.xml")]
    events: PathBuf,
    /// Output file path
    #[clap(short, long, default_value = "traces.trc")]
    output: PathBuf,
    /// Write to human readable json format
    #[arg(short, long)]
    json: bool,
}

impl TracerCommand {
    pub fn run(&self) -> Result<()> {
        // Prepare input paths
        let network_path = self.dir.join(&self.network);
        let population_path = self.dir.join(&self.population);
        let events_path = self.dir.join(&self.events);

        // Prepare input files
        let mut network_reader = xml::reader(&network_path)?;
        let mut population_reader = xml::reader(&population_path)?;
        let events_reader = xml::reader(&events_path)?;

        // Prepare output files
        let traces_file = File::create(&self.output)?;

        // Load network
        let spinner = utils::default_spinner();
        spinner.set_message("[1/4] Loading MATSim network...");
        let network = Network::from_xml(&mut network_reader).context("failed to load network")?;
        spinner.finish_with_message(format!(
            "[1/4] Completed loading network ({} links)",
            HumanCount(network.links.len() as u64)
        ));

        // Load Population
        let spinner = utils::default_spinner();
        spinner.set_message("[2/4] Loading Population...");
        let mut population =
            Population::from_xml(&mut population_reader).context("failed to load population")?;
        spinner.finish_with_message(format!(
            "[2/4] Completed loading population ({} persons/plans)",
            HumanCount(population.len() as u64)
        ));

        // Build Traces
        let progress = utils::default_spinner();
        progress.set_message("[3/4] Building traces...");
        let mut tracer = tracer::TraceHandler::new();
        let mut events = tracer::MATSimEventsReader::from_xml(events_reader);
        tracer.add_network(&network);
        tracer.add_traces(&mut population, &mut events)?;
        progress.finish_with_message("[3/4] Completed building all traces for population");

        // Write Traces
        let spinner = utils::default_spinner();
        spinner.set_message(format!(
            "[4/4] Writing traces to {}...",
            self.output.display()
        ));
        population.serialise(traces_file, self.json)?;
        spinner.finish_with_message(format!(
            "[4/4] Completed writing traces to {}",
            self.output.display()
        ));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs::File, io::BufReader};
    use test_dir::{DirBuilder, TestDir};

    pub fn read_json(path: &PathBuf) -> Result<serde_json::Value> {
        let file = File::open(path).context(format!("unable to open file '{}'", path.display()))?;
        let reader = BufReader::new(file);
        let json = serde_json::from_reader(reader).context("unable to read traces")?;
        Ok(json)
    }

    #[test]
    fn traces_build_from_files() {
        let temp_dir = TestDir::temp();
        let tested_dir = temp_dir.root();
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/data");
        let expected_traces = read_json(&path.join("expected_traces.json")).unwrap();

        let _ = TracerCommand::run(&TracerCommand {
            dir: path.clone(),
            network: path.join("output_network.xml"),
            population: path.join("output_plans.xml"),
            events: path.join("output_events.xml"),
            output: tested_dir.join("traces.json"),
            json: true,
        });

        let output_traces = read_json(&tested_dir.join("traces.json")).unwrap();
        assert_eq!(expected_traces, output_traces)
    }
}
