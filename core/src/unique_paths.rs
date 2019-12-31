use crate::common::{Debug};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Result, BufReader, BufWriter};
use std::path::{PathBuf};
use std::time::Instant;

#[derive(Debug)]
pub struct UniquePathsConfig {
    pub paths_file: PathBuf,
    pub dups_file: PathBuf,
    pub target_file: PathBuf,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
}

pub fn unique_paths(config: UniquePathsConfig) -> Result<()> {
    println!("UNIQUE PATHS | config: {:?}", config);
    let now = Instant::now();
    let mut ctx = Context::new(&config)?;
    ctx.process()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    println!("Written {} lines {:?}", ctx.lines_written, config.target_file);
    println!("Paths discarded: {}", ctx.paths_discarded);
    println!("Errors: {} ({:?})", 0, config.error_log);
    Ok(())
}

struct Context {
    input_paths: PathBuf,
    input_dups: PathBuf,
    output: PathBuf,
    lines_written: u64,
    paths_discarded: u64,
}

#[derive(Debug)]
struct DupEntry {
    pub dups: Vec<String>,
    pub size: String,
}

impl Context {
    pub fn new<'a>(config: &'a UniquePathsConfig) -> Result<Self> {
        Ok(Context {
            input_paths: config.paths_file.clone(),
            input_dups: config.dups_file.clone(),
            output: config.target_file.clone(),
            lines_written: 0,
            paths_discarded: 0
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let dups: Vec<Vec<String>> = serde_json::from_reader(BufReader::new(File::open(&self.input_dups)?))?;
        let mut skip_set: HashSet<String> = HashSet::new();
        for dup in dups.into_iter() {
            for path in dup.into_iter().skip(1) {
                skip_set.insert(path);
            }
        }
        self.paths_discarded = skip_set.len() as u64;
        let mut paths = csv::Reader::from_reader(BufReader::new(File::open(&self.input_paths)?));
        let mut output = csv::Writer::from_writer(BufWriter::new(File::create(&self.output)?));
        for record in paths.records() {
            let record = record?;
            let path = &record[0];
            if skip_set.contains(path) {
                continue;
            }
            output.write_record(&record)?;
            self.lines_written += 1;
        }
        Ok(())
    }
}