use chrono::{TimeZone, Utc};
use sha1::{Digest, Sha1};
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::{read_dir, DirEntry, File, Metadata};
use std::io::{Read, Result, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Instant};

#[derive(Debug)]
pub struct Options {
    pub source_path: PathBuf,
    pub target_file: PathBuf,
    pub traverse_mode: TraverseMode,
    pub debug: Debug,
    pub date_mode: DateMode,
    pub sizes: Sizes,
    pub hashing: Hashing,
    pub error_log: Option<PathBuf>,
}

struct Context<'a> {
    options: &'a Options,
    buf: String,
    file: File,
    new_line: bool,
    error_file: Option<File>,
}

impl Context<'_> {
    pub fn new<'a>(options: &'a Options) -> Result<Context<'a>> {
        Ok(Context {
            options,
            buf: String::with_capacity(200_000_000),
            file: File::create(&options.target_file)?,
            new_line: true,
            error_file: match options.error_log {
                Some(ref name) => Some(File::create(&name)?),
                None => None,
            }
        })
    }
    fn write_field(&mut self, data: &str) -> Result<()> {
        if self.new_line {
            if self.buf.len() > 100_000_000 {
                print!(".");
                self.file.write_all(self.buf.as_bytes())?;
                self.buf.clear();
            }
        } else {
            self.buf.push(',');
        }
        self.buf.push_str(data);
        self.new_line = false;
        Ok(())
    }
    fn write_eol(&mut self) -> Result<()> {
        self.buf.push('\n');
        self.new_line = true;
        Ok(())
    }
    fn end_writing(&mut self) -> Result<()> {
        self.file.write_all(self.buf.as_bytes())
    }
    fn report_error<T: std::error::Error + std::fmt::Display>(&mut self, entry: &DirEntry, error: T) -> Result<()> {
        if let Some(error_file) = &mut self.error_file {
            error_file.write_all(&format!("entry: {:?}, error: {:?}\n", entry, error).as_bytes())?;
        }
        let debugging = if let Debug::On = self.options.debug { true } else { false };
        if debugging || self.error_file.is_none() {
            println!("entry: {:?}, error: {:?}", entry, error);
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TraverseMode {
    Recursive,
    NonRecursive,
}

#[derive(Copy, Clone, Debug)]
pub enum Debug {
    On,
    Off,
}

#[derive(Copy, Clone, Debug)]
pub enum DateMode {
    Yes,
    No,
}

#[derive(Copy, Clone, Debug)]
pub enum Sizes {
    Yes,
    No,
}

#[derive(Copy, Clone, Debug)]
pub enum Hashing {
    Yes,
    No,
}

pub fn process(options: &Options) -> Result<()> {
    let now = Instant::now();
    let mut ctx = Context::new(options)?;
    process_path(&mut ctx, &options.source_path)?;
    ctx.end_writing()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    Ok(())
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
    if let Debug::On = ctx.options.debug {
        print!("path: {:?}, file_name: {:?}", path, file_name);
    }
    ctx.write_field(path.to_str().unwrap())?;
    ctx.write_field(file_name.to_str().unwrap())?;
    if let Sizes::Yes = ctx.options.sizes {
        let size = format_size(metadata.len());
        if let Debug::On = ctx.options.debug {
            print!(", size: {}", size);
        }
        ctx.write_field(&size)?;
    }
    if let Hashing::Yes = ctx.options.hashing {
        let hash = compute_hash(path);
        if let Debug::On = ctx.options.debug {
            print!(", hash: {}", hash);
        }
        ctx.write_field(&hash)?;
    }
    if let DateMode::Yes = ctx.options.date_mode {
        let created = format_date(metadata.created());
        let modified = format_date(metadata.modified());
        let accessed = format_date(metadata.accessed());
        if let Debug::On = ctx.options.debug {
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
    if let Debug::On = ctx.options.debug {
        println!();
    }
    Ok(())
}

const BUFFER_SIZE: usize = 1024;

fn compute_hash(path: &Path) -> String {
    if let Ok(mut file) = File::open(&path) {
        let mut sh = Sha1::default();
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            let n = match file.read(&mut buffer) {
                Ok(n) => n,
                Err(_) => return "READ_ERROR".into(),
            };
            sh.input(&buffer[..n]);
            if n == 0 || n < BUFFER_SIZE {
                break;
            }
        }
        let result = sh.result();
        let mut hash = String::with_capacity(result.len());
        for byte in result {
            write!(&mut hash, "{:02x}", byte).unwrap();
        }
        hash
    } else {
        "OPEN_ERROR".into()
    }
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
    if let TraverseMode::Recursive = ctx.options.traverse_mode {
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
