use crate::common::{Debug};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};
use std::path::{PathBuf};
use std::time::Instant;

#[derive(Debug)]
pub struct DetectDupsConfig {
    pub source_file: PathBuf,
    pub target_file: PathBuf,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
}

pub fn detect_dups(config: DetectDupsConfig) -> Result<()> {
    let now = Instant::now();
    let mut ctx = Context::new(&config)?;
    ctx.process()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    println!("Written {} lines {:?}", ctx.lines_written, config.target_file);
    println!("Paths included: {}", ctx.paths_included);
    println!("Errors: {} ({:?})", 0, config.error_log);
    Ok(())
}

struct Context {
    input: File,
    output: File,
    lines_written: u64,
    paths_included: u64,
}

#[derive(Debug)]
struct DupEntry {
    pub dups: Vec<String>,
    pub size: String,
}

impl Context {
    pub fn new<'a>(config: &'a DetectDupsConfig) -> Result<Self> {
        Ok(Context {
            input: File::open(&config.source_file)?,
            output: File::create(&config.target_file)?,
            lines_written: 0,
            paths_included: 0
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let mut reader = csv::Reader::from_reader(&self.input);
        let mut set: HashMap<String, (String, String)> = HashMap::new();
        let mut dups: HashMap<String, DupEntry> = HashMap::new();
        for record in reader.records() {
            let record = record?;
            assert_eq!(record.len(), 3);
            let key = record[2].to_string();
            if let Some((other_file, other_size)) = set.get(&key) {
                if other_size != &record[1] {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Collision detected between: '{}' and '{}'", &record[0], other_file)));
                }
                if let Some(v) = dups.get_mut(&key) {
                    v.dups.push(record[0].into());
                } else {
                    dups.insert(key, DupEntry { dups: vec![other_file.clone(), record[0].into()], size: other_size.into() });
                }
            } else {
                set.insert(key, (record[0].into(), record[1].into()));
            }
        }
        let mut result: Vec<_> = dups.into_iter().map(|pair| pair.1).collect();
        result.sort_by(|a, b| std::cmp::Ord::cmp(&a.dups[0], &b.dups[0]));
        write!(self.output, "[\n")?;
        let mut first_line = true;
        for v in result.into_iter() {
            if first_line {
                first_line = false;
            } else {
                write!(self.output, ",\n")?;
            }
            let mut first_dup = true;
            for p in v.dups {
                if first_dup {
                    first_dup = false;
                    write!(self.output, "\t[")?;
                } else {
                    write!(self.output, ", ")?;
                }
                write!(self.output, "\"{}\"", p)?;
                self.paths_included += 1;
            }
            write!(self.output, "]")?;
            self.lines_written += 1;
        }
        write!(self.output, "\n]\n")?;
        Ok(())
    }
}