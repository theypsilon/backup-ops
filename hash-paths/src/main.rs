extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use core::common::Debug;
use core::hash_paths::{hash_paths, HashPathsConfig};
use std::io::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "hash-paths", about = "Adds hash to a list of paths.")]
struct CliOpts {
    #[structopt(short = "i", long = "input", help = "Input file")]
    source_file: String,

    #[structopt(short = "o", long = "output", help = "Output file")]
    target_file: String,

    #[structopt(short = "d", long = "debug", help = "Activates debug mode.")]
    debug: bool,

    #[structopt(short = "e", long = "error-log", help = "Error log file.")]
    error_log: Option<String>,
}

impl CliOpts {
    fn into_config(self) -> HashPathsConfig {
        HashPathsConfig {
            source_file: PathBuf::from(&self.source_file),
            target_file: PathBuf::from(&self.target_file),
            debug: if self.debug { Debug::On } else { Debug::Off },
            error_log: self.error_log.as_ref().map(|path| PathBuf::from(&path)),
        }
    }
}

fn main() -> Result<()> {
    let config = CliOpts::from_args().into_config();
    println!("config: {:?}", config);
    hash_paths(config)
}
