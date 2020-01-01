extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use core::common::{Debug};
use core::detect_dups::{detect_dups, DetectDupsConfig};
use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "detect-dups", about = "Detects dups from a set of paths.")]
struct CliOpts {
    #[structopt(short="i", long="input", help = "Input file")]
    source_file: String,

    #[structopt(short="o", long="output", help = "Output file")]
    target_file: String,

    #[structopt(short = "d", long = "debug", help = "Activates debug mode.")]
    debug: bool,

    #[structopt(short = "e", long = "error-log", help = "Error log file.")]
    error_log: Option<String>,
}

impl CliOpts {
    fn into_config(self) -> DetectDupsConfig {
        DetectDupsConfig {
            source_file: PathBuf::from(&self.source_file),
            target_file: PathBuf::from(&self.target_file),
            debug: if self.debug { Debug::On } else { Debug::Off },
            error_log: self.error_log.as_ref().map(|path| PathBuf::from(&path)),
        }
    }
}

fn main() -> Result<()> {
    detect_dups(CliOpts::from_args().into_config())
}
