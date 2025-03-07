use anyhow::{Context, Result};
use clap::Parser;
use indicatif::HumanCount;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};

use crate::utils;
use configuration::{config::Config, handler::AgentConfig, sampler};
use optimise::handler::OptimiseHandler;
use simulate::{record::EventsRecord, results::SummaryHandler};
use tracer::{self, Network, Population};
use xml;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct RunCommand {
    /// Config path
    #[clap(short, long)]
    config: Option<PathBuf>,
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
    /// Path to traces file
    #[clap(short, long, default_value = "traces.trc")]
    trace_path: PathBuf,
    /// Output directory path
    #[clap(short, long, default_value = "outputs")]
    outpath: PathBuf,
    /// Write traces to human readable json format
    #[arg(short, long)]
    json: bool,
}

impl RunCommand {
    pub fn run(&self) -> Result<()> {
        // Load config
        let config = match &self.config {
            Some(path) => Config::load(path),
            None => Ok(Config::default()),
        }?;
        config.valid()?;

        // Prepare input paths
        let network_path = self.dir.join(&self.network);
        let population_path = self.dir.join(&self.population);
        let events_path = self.dir.join(&self.events);

        // Prepare input files
        let mut network_reader = xml::reader(&network_path)?;
        let mut population_reader = xml::reader(&population_path)?;
        let events_reader = xml::reader(&events_path)?;

        // Prepare output paths
        create_dir_all(&self.outpath)?;
        let traces_path = self.outpath.join(Path::new("traces.trc"));
        let mut specs_path = self.outpath.clone();
        specs_path.push("specs.csv");
        let mut report_path = self.outpath.clone();
        report_path.push("report.csv");
        let mut charge_events_path = self.outpath.clone();
        charge_events_path.push("events.csv");

        // Prepare output files
        let traces_file = File::create(&traces_path)?;

        let specs_file = File::create(&specs_path).context(format!(
            // filter records
            "unable to create out file for filters '{}'",
            specs_path.display()
        ))?;
        let mut specs_wtr = csv::Writer::from_writer(specs_file);

        let report_file = File::create(&report_path).context(format!(
            // agent records
            "unable to create out file '{}'",
            report_path.display()
        ))?;
        let mut record_wtr = csv::Writer::from_writer(report_file);

        let events_file = File::create(&charge_events_path).context(format!(
            // events
            "unable to create out file '{}'",
            charge_events_path.display()
        ))?;
        let mut events_wtr = csv::Writer::from_writer(events_file);

        //Rng
        let mut rng = sampler::new(config.seed);

        // Load network
        let spinner = utils::default_spinner();
        spinner.set_message("[1/6] Loading MATSim network...");
        let network = Network::from_xml(&mut network_reader).context("failed to load network")?;
        spinner.finish_with_message(format!(
            "[1/6] Completed loading network ({} links)",
            HumanCount(network.links.len() as u64)
        ));

        // Load Population
        let spinner = utils::default_spinner();
        spinner.set_message("[2/6] Loading Population...");
        let mut population =
            Population::from_xml(&mut population_reader).context("failed to load population")?;
        spinner.finish_with_message(format!(
            "[2/6] Completed loading population ({} persons/plans)",
            HumanCount(population.len() as u64)
        ));

        // Build Traces
        let progress = utils::default_spinner();
        progress.set_message("[3/6] Building traces...");
        let mut tracer = tracer::TraceHandler::new();
        let mut events = tracer::MATSimEventsReader::from_xml(events_reader);
        tracer.add_network(&network);
        tracer.add_traces(&mut population, &mut events)?;
        progress.finish_with_message("[3/6] Completed building all traces for population");

        // Write Traces
        let spinner = utils::default_spinner();
        spinner.set_message(format!(
            "[4/6] Writing traces to {}...",
            &traces_path.display()
        ));
        population.serialise(traces_file, self.json)?;
        spinner.finish_with_message(format!(
            "[4/6] Completed writing traces to {}",
            &traces_path.display()
        ));

        // Optimisation
        let optimiser: OptimiseHandler = OptimiseHandler::new(&config);
        let progress_bar = utils::default_progress_bar(population.len() as u64);
        progress_bar.set_message("[5/6] Optimising agent charging...");

        let sim_records = population
            .into_iter()
            .map(|(pid, person)| {
                progress_bar.inc(1);
                let agent_config = AgentConfig::build(&config, pid, person, &mut rng);
                specs_wtr
                    .serialize(agent_config.to_record())
                    .context(format!("failed to write specs for pid: '{}'", pid))?;
                optimiser
                    .optimise(&config, pid, person, agent_config)
                    .context(format!("optimiser failed at '{pid}'"))
            })
            .collect::<Result<Vec<_>>>()?;

        specs_wtr.flush()?;

        progress_bar.set_length(0);
        progress_bar.tick();
        progress_bar.finish_with_message(format!(
            "[5/6] Completed {} optimised battery simulations",
            sim_records.len()
        ));

        // Write Results
        let progress_bar = utils::default_progress_bar(sim_records.len() as u64);
        progress_bar.set_message(format!(
            "[6/6] Writing results to '{}'...",
            &self.outpath.display()
        ));

        let mut summary = SummaryHandler::new(&config);

        for sim in sim_records.iter() {
            progress_bar.inc(1);

            let record = sim.to_record();
            record_wtr
                .serialize(&record)
                .context(format!("failed to write record pid '{}'", record.pid))?;
            summary.add_leak(record.leak.unwrap());
            for day in sim.slice() {
                for event in day {
                    summary.add(event);
                    events_wtr
                        .serialize(event)
                        .context(format!("failed to write event for pid '{}'", record.pid))?;
                }
            }
        }
        record_wtr.flush()?;
        events_wtr.flush()?;
        summary.finalise();
        progress_bar.finish_with_message(format!(
            "[6/6] Completed writing results to '{}'",
            self.outpath.display()
        ));
        println!("{}", summary);

        Ok(())
    }
}
