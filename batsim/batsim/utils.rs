use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
const SPINNER: &[&str] = &[
    "▰▱▱▱▱▱▱▱▱▱▱▱",
    "▰▰▱▱▱▱▱▱▱▱▱▱",
    "▰▰▰▱▱▱▱▱▱▱▱▱",
    "▰▰▰▰▱▱▱▱▱▱▱▱",
    "▰▰▰▰▰▱▱▱▱▱▱▱",
    "▰▰▰▰▰▰▱▱▱▱▱▱",
    "▰▰▰▰▰▰▰▱▱▱▱▱",
    "▰▰▰▰▰▰▰▰▱▱▱▱",
    "▰▰▰▰▰▰▰▰▰▱▱▱",
    "▰▰▰▰▰▰▰▰▰▰▱▱",
    "▰▰▰▰▰▰▰▰▰▰▰▱",
    "▰▰▰▰▰▰▰▰▰▰▰▰",
    "▰▰▰▰▰▰▰▰▰▰▰▰",
];
const BAR: &str = "▰▰▱";

pub fn default_spinner() -> ProgressBar {
    let sp = ProgressBar::new_spinner();
    sp.enable_steady_tick(Duration::from_millis(100));
    sp.set_style(
        ProgressStyle::with_template(" {spinner:.green/yellow}  {msg} [{elapsed_precise}]")
            .unwrap()
            .tick_strings(SPINNER),
    );
    sp
}

pub fn default_progress_bar(count: u64) -> ProgressBar {
    // Provide a custom bar style
    let pb = ProgressBar::new(count);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(
        ProgressStyle::with_template("[{bar:12.green/yellow}] {msg} [{elapsed_precise}] ")
            .unwrap()
            .progress_chars(BAR)
            .tick_strings(SPINNER),
    );
    pb
}
