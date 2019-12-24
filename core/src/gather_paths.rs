use crate::common::{DateMode, Debug, Hashing, Sizes, TraverseMode};
use crate::internals::compute_hash;
use chrono::{TimeZone, Utc};
use std::ffi::OsString;
use std::fs::{read_dir, DirEntry, File, Metadata};
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

#[derive(Debug)]
pub struct GatherPathsConfig {
    pub source_path: PathBuf,
    pub target_file: PathBuf,
    pub traverse_mode: TraverseMode,
    pub debug: Debug,
    pub date_mode: DateMode,
    pub sizes: Sizes,
    pub hashing: Hashing,
    pub error_log: Option<PathBuf>,
}

pub fn gather_paths(config: &GatherPathsConfig) -> Result<()> {
    let now = Instant::now();
    let mut ctx = Context::new(config)?;
    process_path(&mut ctx, &config.source_path)?;
    ctx.end_writing()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    Ok(())
}

struct Context<'a> {
    config: &'a GatherPathsConfig,
    error_file: Option<File>,
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
            csv_out: csv::Writer::from_writer(File::create(&config.target_file)?),
        })
    }
    fn write_field(&mut self, data: &str) -> Result<()> {
        self.csv_out.write_field(data)?;
        Ok(())
    }
    fn write_eol(&mut self) -> Result<()> {
        self.csv_out.write_record(None::<&[u8]>)?;
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
    process_file_3(ctx, &entry.path(), entry.file_name(), entry.metadata()?)
}

fn process_file_3(
    ctx: &mut Context,
    path: &Path,
    file_name: OsString,
    metadata: Metadata,
) -> Result<()> {
    if let Debug::On = ctx.config.debug {
        print!("path: {:?}, file_name: {:?}", path, file_name);
    }
    ctx.write_field(path.to_str().unwrap())?;
    ctx.write_field(file_name.to_str().unwrap())?;
    if let Sizes::Yes = ctx.config.sizes {
        let size = format_size(metadata.len());
        if let Debug::On = ctx.config.debug {
            print!(", size: {}", size);
        }
        ctx.write_field(&size)?;
    }
    if let Hashing::Yes = ctx.config.hashing {
        let hash = compute_hash(path);
        if let Debug::On = ctx.config.debug {
            print!(", hash: {}", hash);
        }
        ctx.write_field(&hash)?;
    }
    if let DateMode::Yes = ctx.config.date_mode {
        let created = format_date(metadata.created());
        let modified = format_date(metadata.modified());
        let accessed = format_date(metadata.accessed());
        if let Debug::On = ctx.config.debug {
            print!(
                ", created: {}, modified: {}, accessed: {}",
                created, modified, accessed
            );
        }
        ctx.write_field(&created)?;
        ctx.write_field(&modified)?;
        ctx.write_field(&accessed)?;
    }
    ctx.write_eol()?;
    if let Debug::On = ctx.config.debug {
        println!();
    }
    Ok(())
}

fn format_size(len: u64) -> String {
    format!("{}", len)
}

fn format_date(date: Result<SystemTime>) -> String {
    match date {
        Ok(time) => {
            let epoch = match time.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(time) => time.as_secs() as i64,
                Err(_) => return "EPOCH_ERROR".into(),
            };
            Utc.timestamp(epoch, 0)
                .format("%Y-%m-%d (%H:%M:%S)")
                .to_string()
        }
        Err(_) => "NO_SYSTIME".into(),
    }
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
