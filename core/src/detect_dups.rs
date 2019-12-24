use crate::common::{Debug, Hashing};
use crate::internals::compute_hash;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug)]
pub struct DetectDupsConfig {
    pub source_file: PathBuf,
    pub target_file: PathBuf,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
    pub size_min: u64,
    pub exclude_path_starts: Vec<String>,
    pub exclude_path_contents: Vec<String>,
    pub hashing: Hashing,
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
            if self
                .config
                .exclude_path_starts
                .iter()
                .any(|p| record[0].starts_with(p))
                || self
                    .config
                    .exclude_path_contents
                    .iter()
                    .any(|p| record[0].contains(p))
                || record[2].parse::<u64>().unwrap() < self.config.size_min
            {
                continue;
            }
            if let Some(other_file) = set.get(&key) {
                if let Some(v) = dups.get_mut(&key) {
                    v.push(record[0].into());
                } else {
                    dups.insert(key, vec![other_file.clone(), record[0].into()]);
                }
            } else {
                set.insert(key, record[0].into());
            }
        }
        if let Hashing::Yes = self.config.hashing {
            println!(
                "Without hashing:\ndups: {:?}.\n\nWith hashing:",
                count_dups(&dups)
            );

            let mut new_dups: HashMap<String, Vec<String>> = HashMap::new();
            let mut set: HashMap<String, String> = HashMap::new();
            for (_, pack) in dups.into_iter() {
                for path in pack.into_iter() {
                    let hash = compute_hash(&Path::new(&path));
                    let key = hash.to_string();
                    if let Some(other_file) = set.get(&key) {
                        if let Some(v) = new_dups.get_mut(&key) {
                            v.push(path.into());
                        } else {
                            new_dups.insert(key, vec![other_file.clone(), path.into()]);
                        }
                    } else {
                        set.insert(key, path.into());
                    }
                }
            }
            dups = new_dups;
        }

        println!("dups: {:?}.", count_dups(&dups));

        let mut result: Vec<_> = dups.into_iter().map(|pair| pair.1).collect();
        result.sort();
        for v in result.into_iter() {
            write!(self.output, "{:?}\n", v)?;
        }
        Ok(())
    }
}

fn count_dups(map: &HashMap<String, Vec<String>>) -> (usize, usize) {
    let mut files = 0;
    for pair in map {
        files += pair.1.len();
    }
    (map.len(), files)
}
