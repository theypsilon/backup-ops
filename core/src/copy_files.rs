use crate::common::Debug;
use crate::internals::Reporter;
use std::fs::{File};
use std::path::{PathBuf, Path};
use std::time::Instant;
use size_format::{SizeFormatterSI};
use num_format::{Locale, ToFormattedString};
use anyhow::Result;

#[derive(Debug)]
pub struct CopyFilesConfig {
    pub source_file: PathBuf,
    pub target_folder: PathBuf,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
}

pub fn copy_files(config: CopyFilesConfig) -> Result<()> {
    println!("COPY FILES | config: {:?}", config);
    let now = Instant::now();
    let mut ctx = Context::new(config)?;
    ctx.process()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    println!("Copied {} files {:?}", ctx.lines_written.to_formatted_string(&Locale::en), ctx.config.target_folder);
    println!("Disk space taken: {}B", SizeFormatterSI::new(ctx.copied_size));
    println!("Errors: {} ({:?})", ctx.reporter.error_count().to_formatted_string(&Locale::en), ctx.config.error_log);
    Ok(())
}

struct Context {
    config: CopyFilesConfig,
    reporter: Reporter,
    lines_written: u64,
    copied_size: u64
}

impl Context {
    pub fn new(config: CopyFilesConfig) -> Result<Self> {
        Ok(Context {
            reporter: Reporter::new(config.error_log.clone(), config.debug),
            config,
            lines_written: 0,
            copied_size: 0
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let mut reader = csv::Reader::from_reader(File::open(&self.config.source_file)?);
        let start_pos = reader.position().clone();
        let mut total_size = 0;
        for record in reader.records() {
            let record = record?;
            if record.len() > 1 {
                let size = record[1].parse::<u64>()?;
                total_size += size;
            } else {
                total_size = 0;
                break;
            }
        }
        reader.seek(start_pos)?;
        let target_folder = Path::new(&self.config.target_folder);
        let mut current_size: u64 = 0;
        for record in reader.records() {
            let record = record?;
            
            if total_size > 0 {
                let size = record[1].parse::<u64>()?;

                current_size += size;
                print!("\r{:.2}%        ", (current_size as f64 / total_size as f64) * 100.0);
            }

            let source_path = Path::new(&record[0]);
            let target_path = target_folder.join(if source_path.has_root() {
                source_path.components().skip(1).collect::<PathBuf>()
            } else {
                source_path.to_owned()
            });
            std::fs::create_dir_all(&target_path.parent().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Parent should be a dir"))?)?;
            if let Debug::On = self.config.debug {
                print!("Copying {:?} to {:?}", source_path, target_path);
            }
            match std::fs::copy(&source_path, target_path) {
                Ok(size) => self.copied_size += size,
                Err(e) => {
                    self.reporter.report_error(&source_path, e)?;
                    continue;
                }
            }

            self.lines_written += 1;
        }
        if total_size > 0 {
            println!();
        }
        Ok(())
    }
}