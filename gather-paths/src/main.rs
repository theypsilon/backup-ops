extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use core::process::{process, DateMode, Debug, Hashing, Options, Sizes, TraverseMode};
use std::io::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "backup-ops", about = "Tools for personal backup processing.")]
struct Opt {
    #[structopt(help = "Input path")]
    source_path: String,

    #[structopt(help = "Output file")]
    target_file: String,

    #[structopt(
        short = "r",
        long = "recursive",
        help = "Recursive navigation within child folders."
    )]
    recursive: bool,

    #[structopt(short = "d", long = "debug", help = "Activates debug mode.")]
    debug: bool,

    #[structopt(short = "t", long = "time", help = "Saves systime metadata.")]
    dates: bool,

    #[structopt(short = "l", long = "length", help = "Saves file sizes.")]
    lengths: bool,

    #[structopt(short = "s", long = "sha1", help = "Saves file computed hash codes.")]
    sha1: bool,

    #[structopt(short = "e", long = "error-log", help = "Error log file.")]
    error_log: Option<String>,
}

impl Opt {
    fn to_options(&self) -> Options {
        Options {
            source_path: PathBuf::from(&self.source_path),
            target_file: PathBuf::from(&self.target_file),
            traverse_mode: if self.recursive {
                TraverseMode::Recursive
            } else {
                TraverseMode::NonRecursive
            },
            debug: if self.debug { Debug::On } else { Debug::Off },
            date_mode: if self.dates {
                DateMode::Yes
            } else {
                DateMode::No
            },
            sizes: if self.lengths { Sizes::Yes } else { Sizes::No },
            hashing: if self.sha1 { Hashing::Yes } else { Hashing::No },
            error_log: self.error_log.as_ref().map(|path| PathBuf::from(&path)),
        }
    }
}

fn main() -> Result<()> {
    let options = Opt::from_args().to_options();
    if let Debug::On = options.debug {
        println!("{:?}", options);
    }
    process(&options)
}
