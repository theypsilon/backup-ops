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
    let mut ctx = Context::new(config)?;
    ctx.process()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    Ok(())
}

struct Context {
    input: File,
    output: File,
}

#[derive(Debug)]
struct DupEntry {
    pub dups: Vec<String>,
    pub size: String,
}

impl Context {
    pub fn new(config: DetectDupsConfig) -> Result<Self> {
        Ok(Context {
            input: File::open(&config.source_file)?,
            output: File::create(&config.target_file)?,
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
        let result: Vec<_> = dups.into_iter().map(|pair| pair.1).collect();
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
            }
            write!(self.output, "]")?;
        }
        write!(self.output, "\n]\n")?;
        Ok(())
    }
}