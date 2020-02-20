use crate::common::{Debug, HashAlgorithm};
use anyhow::Result;
use digest::Digest;
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;
use sha2::Sha512;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub path: String,
    pub size: u64,
    pub hash: String,
}

pub fn compute_hash(
    path: &Path,
    file_size: u64,
    batch_size: u64,
    algo: HashAlgorithm,
) -> Result<String> {
    let size = if batch_size == 0 {
        if file_size > 100_000_000 {
            0
        } else {
            file_size as usize
        }
    } else {
        if file_size > batch_size {
            batch_size as usize
        } else {
            file_size as usize
        }
    };
    match algo {
        HashAlgorithm::Sha1 => compute_hash_internal(path, size, Sha1::default()),
        HashAlgorithm::Md5 => compute_hash_internal(path, size, Md5::default()),
        HashAlgorithm::Sha256 => compute_hash_internal(path, size, Sha256::default()),
        HashAlgorithm::Sha512 => compute_hash_internal(path, size, Sha512::default()),
    }
}

fn compute_hash_internal(path: &Path, size: usize, mut sh: impl Digest) -> Result<String> {
    let mut file = File::open(&path)?;
    let len = file.metadata()?.len();
    if size == 0 {
        const BUFFER_SIZE: usize = 64768;
        let mut readed: u64 = 0;
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            let n = file.read(&mut buffer)?;
            readed += n as u64;
            sh.input(&buffer[..n]);
            if n == 0 || n < BUFFER_SIZE {
                if readed != len {
                    panic!("Readed and len not matching ({} != {})", readed, len);
                }
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
        write!(&mut hash, "{:02x}", byte)?;
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
    pub fn report_error(
        &mut self,
        entry: &impl std::fmt::Debug,
        error: impl std::fmt::Debug,
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
