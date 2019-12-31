extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use core::common::{Debug, TraverseMode};
use core::gather_paths::{gather_paths, GatherPathsConfig};
use std::io::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "gather-paths", about = "Gather paths from a given path.")]
struct CliOpts {
    #[structopt(short = "i", long = "input", help = "Input paths", required = true)]
    source_paths: Vec<String>,

    #[structopt(short = "o", long = "output", help = "Output file")]
    target_file: String,

    #[structopt(
        short = "r",
        long = "recursive",
        help = "Recursive navigation within child folders."
    )]
    recursive: bool,

    #[structopt(short = "d", long = "debug", help = "Activates debug mode.")]
    debug: bool,

    #[structopt(short = "e", long = "error-log", help = "Error log file.")]
    error_log: Option<String>,
}

impl CliOpts {
    fn to_config(&self) -> GatherPathsConfig {
        GatherPathsConfig {
            source_paths: self.source_paths.iter().map(PathBuf::from).collect(),
            target_file: PathBuf::from(&self.target_file),
            traverse_mode: if self.recursive {
                TraverseMode::Recursive
            } else {
                TraverseMode::NonRecursive
            },
            debug: if self.debug { Debug::On } else { Debug::Off },
            error_log: self.error_log.as_ref().map(|path| PathBuf::from(&path)),
        }
    }
}

fn main() -> Result<()> {
    gather_paths(CliOpts::from_args().to_config())
}
