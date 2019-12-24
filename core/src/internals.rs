use sha1::{Digest, Sha1};
use std::fmt::Write as _;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const BUFFER_SIZE: usize = 1024;

pub(crate) fn compute_hash(path: &Path) -> String {
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
