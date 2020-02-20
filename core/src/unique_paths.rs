use crate::common::Debug;
use crate::internals::Record;
use anyhow::Result;
use num_format::{Locale, ToFormattedString};
use size_format::SizeFormatterSI;
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug)]
pub struct UniquePathsConfig {
    pub paths_file: PathBuf,
    pub dups_file: PathBuf,
    pub target_file: PathBuf,
    pub only_paths: bool,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
}

pub fn unique_paths(config: UniquePathsConfig) -> Result<()> {
    println!("UNIQUE PATHS | config: {:?}", config);
    let now = Instant::now();
    let mut ctx = Context::new(config)?;
    ctx.process()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    println!(
        "Written {} lines {:?}",
        ctx.lines_written.to_formatted_string(&Locale::en),
        ctx.config.target_file
    );
    println!(
        "Paths discarded: {}",
        ctx.paths_discarded.to_formatted_string(&Locale::en)
    );
    println!(
        "Size of all files: {}B",
        SizeFormatterSI::new(ctx.total_size)
    );
    println!("Errors: {} ({:?})", 0, ctx.config.error_log);
    Ok(())
}

struct Context {
    config: UniquePathsConfig,
    lines_written: u64,
    paths_discarded: u64,
    total_size: u64,
}

#[derive(Debug)]
struct DupEntry {
    pub dups: Vec<String>,
    pub size: String,
}

impl Context {
    pub fn new(config: UniquePathsConfig) -> Result<Self> {
        Ok(Context {
            config,
            lines_written: 0,
            paths_discarded: 0,
            total_size: 0,
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let dups: Vec<Vec<String>> = serde_json::from_reader(File::open(&self.config.dups_file)?)?;
        let mut skip_set: HashSet<String> = HashSet::new();
        for dup in dups.into_iter() {
            for path in dup.into_iter().skip(1) {
                skip_set.insert(path);
            }
        }
        self.paths_discarded = skip_set.len() as u64;
        let mut paths = csv::Reader::from_reader(File::open(&self.config.paths_file)?);
        let mut output = csv::Writer::from_writer(File::create(&self.config.target_file)?);
        for record in paths.deserialize() {
            let record: Record = record?;
            let path = &record.path;
            if skip_set.contains(path) {
                continue;
            }
            if self.config.only_paths {
                output.write_field(path)?;
                output.write_record(None::<&[u8]>)?;
            } else {
                output.serialize(&record)?;
            }
            self.total_size += record.size;
            self.lines_written += 1;
        }
        Ok(())
    }
}
