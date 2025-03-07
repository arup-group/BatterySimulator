use anyhow::{Context, Result};
use clap::Parser;
use indicatif::HumanCount;
use std::{
    fs::{create_dir_all, File},
    io::BufReader,
    path::PathBuf,
};

use crate::utils;
use configuration::{config::Config, handler::AgentConfig, sampler};
use optimise::handler::OptimiseHandler;
use simulate::{
    record::{AgentSimulationRecord, EventsRecord},
    results::SummaryHandler,
};
use tracer::Population;

// Entry point for `optmimise` CLI command.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct OptimiseCommand {
    /// Config path
    #[clap(short, long)]
    config: Option<PathBuf>,
    /// Path to traces file
    #[clap(short, long, default_value = "traces.trc")]
    trace_path: PathBuf,
    /// Output directory path
    #[clap(short, long, default_value = "outputs")]
    outpath: PathBuf,
    /// Read traces from human readable json format
    #[arg(short, long)]
    json: bool,
}
impl OptimiseCommand {
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
        create_dir_all(&self.outpath)?;
        let mut specs_path = self.outpath.clone();
        specs_path.push("specs.csv");
        let mut report_path = self.outpath.clone();
        report_path.push("report.csv");
        let mut charge_events_path = self.outpath.clone();
        charge_events_path.push("events.csv");

        // Prepare output files
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

        // Load traces
        let spinner = utils::default_spinner();
        spinner.set_message(format!(
            "[1/3] Loading traces from {}...",
            &self.trace_path.display()
        ));
        let population: Population = Population::deserialise(traces_reader, self.json)?;
        spinner.finish_with_message(format!(
            "[1/3] Completed loading traces ({} persons/plans)",
            HumanCount(population.len() as u64)
        ));

        // Optimisation
        let optimiser: OptimiseHandler = OptimiseHandler::new(&config);
        let progress_bar = utils::default_progress_bar(population.len() as u64);
        progress_bar.set_message("[2/3] Optimising agent charging...");

        let sim_records: Vec<AgentSimulationRecord> = population
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
            "[2/3] Completed {} optimised battery simulations",
            sim_records.len()
        ));

        // Write Results
        let progress_bar = utils::default_progress_bar(sim_records.len() as u64);
        progress_bar.set_message(format!(
            "[3/3] Writing results to '{}'...",
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
            "[3/3] Completed writing results to '{}'",
            self.outpath.display()
        ));
        println!("{}", summary);

        Ok(())
    }
}
