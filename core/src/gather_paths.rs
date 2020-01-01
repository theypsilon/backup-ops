use crate::common::{Debug, TraverseMode};
use crate::internals::Reporter;
use std::fs::{read_dir, DirEntry, File, Metadata};
use std::io::{Result};
use std::path::{Path, PathBuf};
use std::time::{Instant};
use size_format::{SizeFormatterSI};
use num_format::{Locale, ToFormattedString};

#[derive(Debug)]
pub struct GatherPathsConfig {
    pub source_paths: Vec<PathBuf>,
    pub target_file: PathBuf,
    pub traverse_mode: TraverseMode,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
}

pub fn gather_paths(config: GatherPathsConfig) -> Result<()> {
    println!("GATHER PATHS | config: {:?}", config);
    let now = Instant::now();
    let mut ctx = Context::new(config)?;
    for path in ctx.config.source_paths.clone().iter() {
        process_path(&mut ctx, path)?;
    }
    ctx.end_writing()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    println!("Written {} lines {:?}", ctx.lines_written.to_formatted_string(&Locale::en), ctx.config.target_file);
    println!("Size of all files: {}B", SizeFormatterSI::new(ctx.total_size));
    println!("Errors: {} ({:?})", ctx.reporter.error_count().to_formatted_string(&Locale::en), ctx.config.error_log);
    Ok(())
}

struct Context {
    config: GatherPathsConfig,
    reporter: Reporter,
    lines_written: u64,
    csv_out: csv::Writer<File>,
    total_size: u64,
}

impl Context {
    pub fn new(config: GatherPathsConfig) -> Result<Context> {
        Ok(Context {
            csv_out: csv::Writer::from_writer(File::create(&config.target_file)?),
            reporter: Reporter::new(config.error_log.clone(), config.debug),
            config,
            lines_written: 0,
            total_size: 0,
        })
    }
    fn write_field(&mut self, data: &str) -> Result<()> {
        self.csv_out.write_field(data)?;
        Ok(())
    }
    fn write_eol(&mut self) -> Result<()> {
        self.csv_out.write_record(None::<&[u8]>)?;
        self.lines_written += 1;
        Ok(())
    }
    fn end_writing(&mut self) -> Result<()> {
        Ok(())
    }
}

fn process_path(ctx: &mut Context, path: &Path) -> Result<()> {
    for entry in read_dir(&path)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            process_dir_1(ctx, &entry)?;
        }
        if ty.is_file() {
            process_file_1(ctx, &entry)?;
        }
        if ty.is_symlink() {
            continue;
        }
    }
    Ok(())
}

fn process_file_1(ctx: &mut Context, entry: &DirEntry) -> Result<()> {
    match process_file_2(ctx, entry) {
        Ok(()) => {}
        Err(e) => ctx.reporter.report_error(entry, e)?,
    };
    Ok(())
}

fn process_file_2(ctx: &mut Context, entry: &DirEntry) -> Result<()> {
    process_file_3(ctx, &entry.path(), entry.metadata()?)
}

fn process_file_3(ctx: &mut Context, path: &Path, metadata: Metadata) -> Result<()> {
    if let Debug::On = ctx.config.debug {
        print!("path: {:?}", path);
    }
    ctx.write_field(path.to_str().unwrap())?;
    let size = metadata.len();
    if let Debug::On = ctx.config.debug {
        print!(", size: {}", size);
    }
    ctx.total_size += size;
    ctx.write_field(&size.to_string())?;
    ctx.write_field("NULL")?;
    ctx.write_eol()?;
    if let Debug::On = ctx.config.debug {
        println!();
    }
    Ok(())
}

fn process_dir_1(ctx: &mut Context, entry: &DirEntry) -> Result<()> {
    match process_dir_2(ctx, entry) {
        Ok(()) => {}
        Err(e) => ctx.reporter.report_error(entry, e)?,
    };
    Ok(())
}

fn process_dir_2(ctx: &mut Context, entry: &DirEntry) -> Result<()> {
    if let TraverseMode::Recursive = ctx.config.traverse_mode {
        process_path(ctx, &entry.path())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
