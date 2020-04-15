use crate::common::{Debug, HashAlgorithm};
use crate::internals::{compute_hash, Record, Reporter};
use anyhow::Result;
use num_format::{Locale, ToFormattedString};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug)]
pub struct HashPathsConfig {
    pub source_file: PathBuf,
    pub target_file: PathBuf,
    pub bytes: u64,
    pub algorithm: HashAlgorithm,
    pub show_progression: bool,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
}

pub fn hash_paths(config: HashPathsConfig) -> Result<()> {
    println!("HASH PATHS | config: {:?}", config);
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
        "Errors: {} ({:?})",
        ctx.reporter.error_count().to_formatted_string(&Locale::en),
        ctx.config.error_log
    );
    Ok(())
}

struct Context {
    config: HashPathsConfig,
    reporter: Reporter,
    lines_written: u64,
}

impl Context {
    pub fn new(config: HashPathsConfig) -> Result<Self> {
        Ok(Context {
            reporter: Reporter::new(config.error_log.clone(), config.debug),
            config,
            lines_written: 0,
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let mut reader = csv::Reader::from_reader(File::open(&self.config.source_file)?);
        let mut total_size = 0;
        for record in reader.deserialize() {
            let record: Record = record?;
            total_size += record.size;
        }
        let mut reader = csv::Reader::from_reader(File::open(&self.config.source_file)?);
        let mut writer = csv::Writer::from_writer(File::create(&self.config.target_file)?);
        let mut current_size: u64 = 0;
        for record in reader.deserialize() {
            let mut record: Record = record?;
            let path = &record.path;
            let size = record.size;

            if self.config.show_progression {
                current_size += size;
                print!(
                    "\r{:.2}%        ",
                    (current_size as f64 / total_size as f64) * 100.0
                );
            }

            record.hash = match compute_hash(
                Path::new(path),
                size,
                self.config.bytes,
                self.config.algorithm,
            ) {
                Ok(hash) => hash,
                Err(e) => {
                    self.reporter.report_error(&path.to_string(), e)?;
                    continue;
                }
            };
            writer.serialize(record)?;
            self.lines_written += 1;
        }
        println!();
        Ok(())
    }
}
