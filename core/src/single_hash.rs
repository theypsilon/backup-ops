use crate::common::Debug;
use crate::internals::compute_hash;
use anyhow::{anyhow, Result};
use size_format::SizeFormatterSI;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug)]
pub struct SingleHashConfig {
    pub source_file: PathBuf,
    pub bytes: u64,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
}

pub fn single_hash(config: SingleHashConfig) -> Result<()> {
    println!("SINGLE HASH | config: {:?}", config);
    let now = Instant::now();
    let mut ctx = Context::new(config)?;
    let hash = ctx.process()?;
    println!("Calculated hash is: {}", hash);
    println!("Duration: {:#?}", (Instant::now() - now));
    println!(
        "File size: {}",
        SizeFormatterSI::new(ctx.file_size)
    );
    println!("Errors: {} ({:?})", 0, ctx.config.error_log);
    Ok(())
}

struct Context {
    config: SingleHashConfig,
    file_size: u64,
}

impl Context {
    pub fn new(config: SingleHashConfig) -> Result<Self> {
        Ok(Context {
            config,
            file_size: 0
        })
    }

    pub fn process(&mut self) -> Result<String> {
        let file = File::open(&self.config.source_file)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.is_dir() {
            return Err(anyhow!(
                "The path '{:?}' is not a normal file.", self.config.source_file
            ));
        }
        let size = metadata.len();
        self.file_size = size;
        compute_hash(
            &self.config.source_file,
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
        )
    }
}