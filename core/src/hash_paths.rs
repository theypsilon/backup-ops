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
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
}

pub fn hash_paths(config: HashPathsConfig) -> Result<()> {
    let now = Instant::now();
    let mut ctx = Context::new(config)?;
    ctx.process()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    Ok(())
}

struct Context {
    input: File,
    output: File,
}

impl Context {
    pub fn new(config: HashPathsConfig) -> Result<Self> {
        Ok(Context {
            input: File::open(&config.source_file)?,
            output: File::create(&config.target_file)?,
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let mut reader = csv::Reader::from_reader(&self.input);
        let mut writer = csv::Writer::from_writer(&self.output);
        for record in reader.records() {
            let record = record?;
            assert_eq!(record.len(), 2);
            let path = &record[0];
            let hash = compute_hash(Path::new(path));
            writer.write_field(&record[0])?;
            writer.write_field(&record[1])?;
            writer.write_field(&hash.to_string())?;
            writer.write_record(None::<&[u8]>)?;
        }
        Ok(())
    }
}