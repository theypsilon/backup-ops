use crate::common::Debug;
use crate::internals::compute_hash;
use crate::internals::Reporter;
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
        let start_pos = reader.position().clone();
        let mut total_size = 0;
        for record in reader.records() {
            let record = record?;
            let size = record[1].parse::<u64>()?;
            total_size += size;
        }
        reader.seek(start_pos)?;
        let mut writer = csv::Writer::from_writer(File::create(&self.config.target_file)?);
        let mut current_size: u64 = 0;
        for record in reader.records() {
            let record = record?;
            let path = &record[0];
            let size = record[1].parse::<u64>()?;

            current_size += size;
            print!(
                "\r{:.2}%        ",
                (current_size as f64 / total_size as f64) * 100.0
            );

            let hash = match compute_hash(
                Path::new(path),
                if self.config.bytes == 0 {
                    if size > 100_000_000 {
                        0
                    } else {
                        size as usize
                    }
                } else {
                    if size > self.config.bytes {
                        self.config.bytes as usize
                    } else {
                        size as usize
                    }
                },
            ) {
                Ok(hash) => hash,
                Err(e) => {
                    self.reporter.report_error(&path.to_string(), e)?;
                    continue;
                }
            };
            writer.write_field(&record[0])?;
            writer.write_field(&record[1])?;
            writer.write_field(&hash.to_string())?;
            writer.write_record(None::<&[u8]>)?;

            self.lines_written += 1;
        }
        println!();
        Ok(())
    }
}
