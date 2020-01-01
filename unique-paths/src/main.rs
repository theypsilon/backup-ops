extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use core::common::{Debug};
use core::unique_paths::{unique_paths, UniquePathsConfig};
use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "unique-paths", about = "Generates a list from files that are unique.")]
struct CliOpts {
    #[structopt(long="input-paths", help = "Input paths")]
    paths_file: String,

    #[structopt(long="input-dups", help = "Input duplicated files")]
    dups_file: String,

    #[structopt(short="o", long="output", help = "Output file")]
    target_file: String,

    #[structopt(short="p", long="only-paths", help = "Only output paths, and not the full record.")]
    only_paths: bool,
    
    #[structopt(short = "d", long = "debug", help = "Activates debug mode.")]
    debug: bool,

    #[structopt(short = "e", long = "error-log", help = "Error log file.")]
    error_log: Option<String>,
}

impl CliOpts {
    fn into_config(self) -> UniquePathsConfig {
        UniquePathsConfig {
            paths_file: PathBuf::from(&self.paths_file),
            dups_file: PathBuf::from(&self.dups_file),
            target_file: PathBuf::from(&self.target_file),
            only_paths: self.only_paths,
            debug: if self.debug { Debug::On } else { Debug::Off },
            error_log: self.error_log.as_ref().map(|path| PathBuf::from(&path)),
        }
    }
}

fn main() -> Result<()> {
    unique_paths(CliOpts::from_args().into_config())
}
