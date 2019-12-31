use sha1::{Digest, Sha1};
use std::fmt::Write as _;
use std::fs::File;
use std::io::{Read, BufReader, Result};
use std::path::Path;

pub(crate) fn compute_hash(path: &Path, size: usize) -> Result<String> {
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

/*
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