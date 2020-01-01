use sha1::{Digest, Sha1};
use std::fmt::Write as _;
use std::fs::File;
use std::io::{Read, Write, BufReader, Result};
use std::path::{Path, PathBuf};
use crate::common::Debug;

pub fn compute_hash(path: &Path, size: usize) -> Result<String> {
    let mut file = File::open(&path)?;
    let mut sh = Sha1::default();
    if size == 0 {
        const BUFFER_SIZE: usize = 8096;
        let mut buffer = [0u8; BUFFER_SIZE];
        let mut reader = BufReader::new(file);
        loop {
            let n = reader.read(&mut buffer)?;
            sh.input(&buffer[..n]);
            if n == 0 || n < BUFFER_SIZE {
                break;
            }
        }
    } else {
        let mut buffer = vec![0; size];
        file.read_exact(&mut buffer)?;
        sh.input(&buffer[..]);
    }
    let result = sh.result();
    let mut hash = String::with_capacity(result.len());
    for byte in result {
        write!(&mut hash, "{:02x}", byte).unwrap();
    }
    Ok(hash)
}

pub struct Reporter {
    errors_path: Option<PathBuf>,
    errors_file: Option<File>,
    errors_reported: u64,
    debug: Debug,
}

impl Reporter {
    pub fn new(path: Option<PathBuf>, debug: Debug) -> Self {
        Reporter {
            errors_path: path,
            errors_file: None,
            errors_reported: 0,
            debug,
        }
    }
    pub fn report_error<T: std::error::Error + std::fmt::Display>(
        &mut self,
        entry: &impl std::fmt::Debug,
        error: T,
    ) -> Result<()> {
        self.errors_reported += 1;
        if let Some(errors_path) = &mut self.errors_path {
            match self.errors_file {
                None => self.errors_file = Some(File::create(errors_path)?),
                _ => {}
            }
        }
        if let Some(errors_file) = &mut self.errors_file {
            errors_file
                .write_all(&format!("entry: {:?}, error: {:?}\n", entry, error).as_bytes())?;
        }
        let debugging = if let Debug::On = self.debug {
            true
        } else {
            false
        };
        if debugging || self.errors_file.is_none() {
            println!("entry: {:?}, error: {:?}", entry, error);
        }
        Ok(())
    }

    pub fn error_count(&self) -> u64 {
        self.errors_reported
    }
}

/*
use chrono::{TimeZone, Utc};
use std::time::{Instant, SystemTime};

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
}*/