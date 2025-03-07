use std::path::PathBuf;

use crate::utils;
use anyhow::{Context, Result};
use clap::Parser;
use peek::attributes::peek_attributes;
use xml;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct PeekCommand {
    /// Path to MATSim xml plans to peek
    #[clap(short, long, default_value = "output_plans.xml")]
    plans: PathBuf,
    /// Max number of attribute values to show
    #[clap(short, long, default_value = "10")]
    max: usize,
}

impl PeekCommand {
    pub fn run(&self) -> Result<()> {
        let mut reader = xml::reader(&self.plans)?;
        let spinner = utils::default_spinner();
        spinner.set_message("[1/1] Reading...");
        let attributes =
            peek_attributes(&mut reader, self.max).context("failed to load attributes")?;
        spinner.finish_with_message("[1/1] Completed");

        println!("\n\nFound {} population attributes:", attributes.len());
        for (k, v) in attributes.into_iter() {
            print!("- {}: ", k);
            println!("{}", v);
        }
        Ok(())
    }
}
