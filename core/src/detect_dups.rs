use crate::common::Debug;
use crate::internals::Record;
use anyhow::{anyhow, Result};
use num_format::{Locale, ToFormattedString};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug)]
pub struct DetectDupsConfig {
    pub source_file: PathBuf,
    pub target_file: PathBuf,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
}

pub fn detect_dups(config: DetectDupsConfig) -> Result<()> {
    println!("DETECT DUPS | config: {:?}", config);
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
        "Paths included: {}",
        ctx.paths_included.to_formatted_string(&Locale::en)
    );
    println!("Errors: {} ({:?})", 0, ctx.config.error_log);
    Ok(())
}

struct Context {
    config: DetectDupsConfig,
    lines_written: u64,
    paths_included: u64,
}

type DupEntry = Vec<String>;

impl Context {
    pub fn new<'a>(config: DetectDupsConfig) -> Result<Self> {
        Ok(Context {
            config,
            lines_written: 0,
            paths_included: 0,
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let input = File::open(&self.config.source_file)?;
        let mut reader = csv::Reader::from_reader(input);
        let mut set: HashMap<String, (String, u64)> = HashMap::new();
        let mut dup_map: HashMap<String, DupEntry> = HashMap::new();
        for record in reader.deserialize() {
            let record: Record = record?;
            let key = record.hash.clone();
            if let Some((other_file, other_size)) = set.get(&key) {
                if *other_size != record.size {
                    return Err(anyhow!(
                        "Collision detected between: '{}' and '{}'",
                        &record.path,
                        other_file
                    ));
                }
                if let Some(v) = dup_map.get_mut(&key) {
                    v.push(record.path.into());
                } else {
                    dup_map.insert(key, vec![other_file.clone(), record.path.into()]);
                }
            } else {
                set.insert(key, (record.path.into(), record.size.into()));
            }
        }
        let mut dup_entries: Vec<_> = dup_map.into_iter().map(|pair| pair.1).collect();
        dup_entries.iter_mut().for_each(|v| v.sort_by(std::cmp::Ord::cmp));
        dup_entries.sort_by(|a, b| std::cmp::Ord::cmp(&a[0], &b[0]));

        let mut output = File::create(&self.config.target_file)?;
        write!(output, "[\n")?;
        let mut first_line = true;
        for dup_entry in dup_entries.into_iter() {
            if first_line {
                first_line = false;
            } else {
                write!(output, ",\n")?;
            }
            let mut first_dup = true;
            for p in dup_entry {
                if first_dup {
                    first_dup = false;
                    write!(output, "\t[")?;
                } else {
                    write!(output, ", ")?;
                }
                write!(output, "\"{}\"", p)?;
                self.paths_included += 1;
            }
            write!(output, "]")?;
            self.lines_written += 1;
        }
        write!(output, "\n]\n")?;
        Ok(())
    }
}
