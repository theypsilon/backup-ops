extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use anyhow::Result;
use core::common::Debug;
use core::single_hash::{single_hash, SingleHashConfig};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "single-hash", about = "Calculate hash of a file")]
struct CliOpts {
    #[structopt(short = "i", long = "input", help = "Input file")]
    source_file: String,

    #[structopt(short = "d", long = "debug", help = "Activates debug mode.")]
    debug: bool,

    #[structopt(
        short = "b",
        long = "bytes",
        help = "Determine how many bytes are readed to calculate the hash. Zero means all bytes. Default value is 0."
    )]
    bytes: Option<u64>,

    #[structopt(short = "e", long = "error-log", help = "Error log file.")]
    error_log: Option<String>,
}

impl CliOpts {
    fn into_config(self) -> SingleHashConfig {
        SingleHashConfig {
            source_file: PathBuf::from(&self.source_file),
            debug: if self.debug { Debug::On } else { Debug::Off },
            bytes: if let Some(bytes) = self.bytes {
                bytes
            } else {
                0
            },
            error_log: self.error_log.as_ref().map(|path| PathBuf::from(&path)),
        }
    }
}

fn main() -> Result<()> {
    single_hash(CliOpts::from_args().into_config())
}
