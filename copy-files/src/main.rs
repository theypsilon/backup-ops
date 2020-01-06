extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use anyhow::Result;
use core::common::Debug;
use core::copy_files::{copy_files, CopyFilesConfig};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "hash-paths", about = "Adds hash to a list of paths.")]
struct CliOpts {
    #[structopt(short = "i", long = "input", help = "Input file")]
    source_file: String,

    #[structopt(short = "o", long = "output", help = "Output folder")]
    target_folder: String,

    #[structopt(
        short = "f",
        long = "flatten-output",
        help = "Outputs all the files in a single folder instead of recreating the whole filesystem tree."
    )]
    flatten_output: bool,

    #[structopt(short = "d", long = "debug", help = "Activates debug mode.")]
    debug: bool,

    #[structopt(short = "e", long = "error-log", help = "Error log file.")]
    error_log: Option<String>,
}

impl CliOpts {
    fn into_config(self) -> CopyFilesConfig {
        CopyFilesConfig {
            source_file: PathBuf::from(&self.source_file),
            target_folder: PathBuf::from(&self.target_folder),
            flatten_output: self.flatten_output,
            debug: if self.debug { Debug::On } else { Debug::Off },
            error_log: self.error_log.as_ref().map(|path| PathBuf::from(&path)),
        }
    }
}

fn main() -> Result<()> {
    copy_files(CliOpts::from_args().into_config())
}
