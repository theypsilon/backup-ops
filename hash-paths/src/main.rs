extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use anyhow::Result;
use core::common::{Debug, HashAlgorithm};
use core::hash_paths::{hash_paths, HashPathsConfig};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "hash-paths", about = "Adds hash to a list of paths.")]
struct CliOpts {
    #[structopt(short = "i", long = "input", help = "Input file")]
    source_file: String,

    #[structopt(short = "o", long = "output", help = "Output file")]
    target_file: String,

    #[structopt(
        short = "b",
        long = "bytes",
        help = "Determine how many bytes are readed to calculate the hash. Zero means all bytes. Default value is 0."
    )]
    bytes: Option<u64>,

    #[structopt(
        short = "a",
        long = "algorithm",
        help = "Choose hash algorithm. Default algorithm is Sha1."
    )]
    algorithm: Option<HashAlgorithm>,

    #[structopt(
        short = "p",
        long = "show-progression",
        help = "Show progression information."
    )]
    progression: bool,

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
            bytes: self.bytes.unwrap_or(0),
            algorithm: self.algorithm.unwrap_or(HashAlgorithm::Md5),
            show_progression: self.progression,
            debug: if self.debug { Debug::On } else { Debug::Off },
            error_log: self.error_log.as_ref().map(|path| PathBuf::from(&path)),
        }
    }
}

fn main() -> Result<()> {
    hash_paths(CliOpts::from_args().into_config())
}
