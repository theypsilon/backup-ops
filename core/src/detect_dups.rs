use crate::common::{DateMode, Debug, Hashing, Sizes, TraverseMode};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::{read_dir, DirEntry, File, Metadata};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Read, Result, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

#[derive(Debug)]
pub struct DetectDupsConfig {
    pub source_file: PathBuf,
    pub target_file: PathBuf,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
    pub size_min: u64,
    pub exclude_path_starts: Vec<String>,
    pub exclude_path_contents: Vec<String>,
}

pub fn detect_dups(config: DetectDupsConfig) -> Result<()> {
    let now = Instant::now();
    let mut ctx = Context::new(config)?;
    ctx.process()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    Ok(())
}

struct Context {
    config: DetectDupsConfig,
    input: File,
    output: File,
}

impl Context {
    pub fn new(config: DetectDupsConfig) -> Result<Self> {
        Ok(Context {
            input: File::open(&config.source_file)?,
            output: File::create(&config.target_file)?,
            config,
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let mut reader = csv::Reader::from_reader(&self.input);
        let mut set: HashMap<String, String> = HashMap::new();
        let mut dups: HashMap<String, Vec<String>> = HashMap::new();
        for record in reader.records() {
            let record = record?;
            assert_eq!(record.len(), 3);
            let key = format!("{}|{}", &record[1], &record[2]);
            if let Some(other_file) = set.get(&key) {
                if let Some(v) = dups.get_mut(&key) {
                    v.push(other_file.clone());
                } else {
                    dups.insert(key, vec![record[0].into(), other_file.clone()]);
                }
            } else if record[2].parse::<u64>().unwrap() >= self.config.size_min {
                if !self
                    .config
                    .exclude_path_starts
                    .iter()
                    .any(|p| record[0].starts_with(p))
                    && !self
                        .config
                        .exclude_path_contents
                        .iter()
                        .any(|p| record[0].contains(p))
                {
                    set.insert(key, record[0].into());
                }
            }
        }
        let mut count = 0;
        let entries = dups.len();
        for pair in dups.into_iter() {
            count += pair.1.len();
            write!(self.output, "{:?}\n", pair)?;
        }
        println!("dups: {:?} in {} entries.", count, entries);
        Ok(())
    }
}
