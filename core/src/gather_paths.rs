use crate::common::{Debug, TraverseMode};
use std::fs::{read_dir, DirEntry, File, Metadata};
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant};

#[derive(Debug)]
pub struct GatherPathsConfig {
    pub source_paths: Vec<PathBuf>,
    pub target_file: PathBuf,
    pub traverse_mode: TraverseMode,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
}

pub fn gather_paths(config: GatherPathsConfig) -> Result<()> {
    println!("config: {:?}", config);
    let now = Instant::now();
    let mut ctx = Context::new(&config)?;
    for path in config.source_paths.iter() {
        process_path(&mut ctx, path)?;
    }
    ctx.end_writing()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    println!("Written {} lines {:?}", ctx.lines_written, config.target_file);
    println!("Errors: {} ({:?})", ctx.errors_reported, config.error_log);
    Ok(())
}

struct Context<'a> {
    config: &'a GatherPathsConfig,
    error_file: Option<File>,
    errors_reported: u64,
    lines_written: u64,
    csv_out: csv::Writer<File>,
}

impl Context<'_> {
    pub fn new<'a>(config: &'a GatherPathsConfig) -> Result<Context<'a>> {
        Ok(Context {
            config,
            error_file: match config.error_log {
                Some(ref name) => Some(File::create(&name)?),
                None => None,
            },
            errors_reported: 0,
            lines_written: 0,
            csv_out: csv::Writer::from_writer(File::create(&config.target_file)?),
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
    fn report_error<T: std::error::Error + std::fmt::Display>(
        &mut self,
        entry: &DirEntry,
        error: T,
    ) -> Result<()> {
        self.errors_reported += 1;
        if let Some(error_file) = &mut self.error_file {
            error_file
                .write_all(&format!("entry: {:?}, error: {:?}\n", entry, error).as_bytes())?;
        }
        let debugging = if let Debug::On = self.config.debug {
            true
        } else {
            false
        };
        if debugging || self.error_file.is_none() {
            println!("entry: {:?}, error: {:?}", entry, error);
        }
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
        Err(e) => ctx.report_error(entry, e)?,
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
    let size = format_size(metadata.len());
    if let Debug::On = ctx.config.debug {
        print!(", size: {}", size);
    }
    ctx.write_field(&size)?;
    ctx.write_field("NULL")?;
    ctx.write_eol()?;
    if let Debug::On = ctx.config.debug {
        println!();
    }
    Ok(())
}

fn format_size(len: u64) -> String {
    format!("{}", len)
}

fn process_dir_1(ctx: &mut Context, entry: &DirEntry) -> Result<()> {
    match process_dir_2(ctx, entry) {
        Ok(()) => {}
        Err(e) => ctx.report_error(entry, e)?,
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
