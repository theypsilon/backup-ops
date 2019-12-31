use crate::common::Debug;
use std::fs::File;
use std::io::Result;
use std::path::{PathBuf, Path};
use std::time::Instant;
use crate::internals::compute_hash;

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
    let mut ctx = Context::new(&config)?;
    ctx.process()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    println!("Written {} lines {:?}", ctx.lines_written, config.target_file);
    println!("Errors: {} ({:?})", 0, config.error_log);
    Ok(())
}

struct Context<'a> {
    input: File,
    output: File,
    config: &'a HashPathsConfig,
    lines_written: u64,
}

impl<'a> Context<'a> {
    pub fn new(config: &'a HashPathsConfig) -> Result<Self> {
        Ok(Context {
            input: File::open(&config.source_file)?,
            output: File::create(&config.target_file)?,
            config,
            lines_written: 0,
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let mut reader = csv::Reader::from_reader(&self.input);
        let start_pos = reader.position().clone();
        let mut total_size = 0;
        for record in reader.records() {
            let record = record?;
            let size = record[1].parse::<u64>().unwrap();
            total_size += size;
        }
        reader.seek(start_pos)?;
        let mut writer = csv::Writer::from_writer(&self.output);
        let mut current_size: u64 = 0;
        for record in reader.records() {
            let record = record?;
            let path = &record[0];
            let size = record[1].parse::<u64>().unwrap();
            let hash = compute_hash(Path::new(path), if self.config.bytes == 0 {
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
            })?;
            writer.write_field(&record[0])?;
            writer.write_field(&record[1])?;
            writer.write_field(&hash.to_string())?;
            writer.write_record(None::<&[u8]>)?;
            
            self.lines_written += 1;

            current_size += size;
            print!("\r{:.2}%", (current_size as f64 / total_size as f64) * 100.0);
        }
        println!();
        Ok(())
    }
}