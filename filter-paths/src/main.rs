extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use core::common::Debug;
use core::filter_paths::{filter_paths, FilterPath, FilterPathsConfig};
use std::io::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "filter-paths", about = "Filter paths according to given options.")]
struct CliOpts {
    #[structopt(short = "i", long = "input", help = "Input file")]
    source_file: String,

    #[structopt(short = "o", long = "output", help = "Output file")]
    target_file: String,

    #[structopt(long = "size-min", help = "Minimum size to consider (Default 0).")]
    size_min: Option<u64>,

    #[structopt(short = "d", long = "debug", help = "Activates debug mode.")]
    debug: bool,

    #[structopt(short = "e", long = "error-log", help = "Error log file.")]
    error_log: Option<String>,

    #[structopt(
        long = "size-max",
        help = "Maximum size to consider (Default u64 MAX)."
    )]
    size_max: Option<u64>,

    #[structopt(long = "exclude-unique-sizes", help = "Exclude files that don't have the same size as other files in the set.")]
    unique_sizes: bool,

    #[structopt(long = "exclude-unique-hashes", help = "Exclude files that don't have the same hash as other files in the set.")]
    unique_hashes: bool,

    #[structopt(
        long = "blacklist-path-starts",
        help = "Excluding paths starting in this way. Prepend ':case-insensitive:!' if you don't wanna have a case sensitive match."
    )]
    blacklist_path_starts: Vec<String>,

    #[structopt(
        long = "blacklist-path-ends",
        help = "Excluding paths ending in this way. Prepend ':case-insensitive:!' if you don't wanna have a case sensitive match."
    )]
    blacklist_path_ends: Vec<String>,

    #[structopt(
        long = "blacklist-path-containing",
        help = "Excluding paths containing these substrings. Prepend ':case-insensitive:!' if you don't wanna have a case sensitive match."
    )]
    blacklist_path_contents: Vec<String>,

    #[structopt(
        long = "whitelist-path-ends",
        help = "Include only paths ending in this way. Prepend ':case-insensitive:!' if you don't wanna have a case sensitive match."
    )]
    whitelist_path_ends: Vec<String>,

    #[structopt(
        long = "whitelist-path-containing",
        help = "Include only paths containing these substrings. Prepend ':case-insensitive:!' if you don't wanna have a case sensitive match."
    )]
    whitelist_path_contents: Vec<String>,
}

impl CliOpts {
    fn into_config(self) -> FilterPathsConfig {
        FilterPathsConfig {
            source_file: PathBuf::from(&self.source_file),
            target_file: PathBuf::from(&self.target_file),
            debug: if self.debug { Debug::On } else { Debug::Off },
            error_log: self.error_log.as_ref().map(|path| PathBuf::from(&path)),
            size_min: self.size_min.unwrap_or(0),
            size_max: self.size_max.unwrap_or(std::u64::MAX),
            unique_sizes: self.unique_sizes,
            unique_hashes: self.unique_hashes,
            blacklist_path_starts: self
                .blacklist_path_starts
                .iter()
                .map(|p| FilterPath::new(p))
                .collect(),
            blacklist_path_ends: self
                .blacklist_path_ends
                .iter()
                .map(|p| FilterPath::new(p))
                .collect(),
            blacklist_path_contents: self
                .blacklist_path_contents
                .iter()
                .map(|p| FilterPath::new(p))
                .collect(),
            whitelist_path_ends: self
                .whitelist_path_ends
                .iter()
                .map(|p| FilterPath::new(p))
                .collect(),
            whitelist_path_contents: self
                .whitelist_path_contents
                .iter()
                .map(|p| FilterPath::new(p))
                .collect(),
        }
    }
}

fn main() -> Result<()> {
    filter_paths(CliOpts::from_args().into_config())
}
