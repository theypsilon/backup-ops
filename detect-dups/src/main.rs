extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use core::common::{DateMode, Debug, Hashing, Sizes, TraverseMode};
use core::detect_dups::{detect_dups, DetectDupsConfig};
use std::io::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "detect-dups", about = "Detects dups from a set of paths.")]
struct CliOpts {
    #[structopt(help = "Input file")]
    source_file: String,

    #[structopt(help = "Output file")]
    target_file: String,

    #[structopt(
        short = "m",
        long = "size-min",
        help = "Minimum size to consider (Default 0)."
    )]
    size_min: Option<u64>,

    #[structopt(short = "d", long = "debug", help = "Activates debug mode.")]
    debug: bool,

    #[structopt(short = "e", long = "error-log", help = "Error log file.")]
    error_log: Option<String>,

    #[structopt(
        long = "exclude-path-starts",
        help = "Excluding paths starting in this way."
    )]
    exclude_path_starts: Vec<String>,

    #[structopt(
        long = "exclude-path-contents",
        help = "Excluding paths containing these substrings."
    )]
    exclude_path_contents: Vec<String>,
}

impl CliOpts {
    fn into_config(self) -> DetectDupsConfig {
        DetectDupsConfig {
            source_file: PathBuf::from(&self.source_file),
            target_file: PathBuf::from(&self.target_file),
            debug: if self.debug { Debug::On } else { Debug::Off },
            error_log: self.error_log.as_ref().map(|path| PathBuf::from(&path)),
            size_min: self.size_min.unwrap_or(0),
            exclude_path_starts: self.exclude_path_starts,
            exclude_path_contents: self.exclude_path_contents,
        }
    }
}

fn main() -> Result<()> {
    let config = CliOpts::from_args().into_config();
    println!("config: {:?}", config);
    detect_dups(config)
}
