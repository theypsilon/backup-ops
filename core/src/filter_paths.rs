use crate::common::Debug;
use std::fs::File;
use std::io::Result;
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Debug)]
pub struct FilterPath {
    path: String,
    case_insensitive: bool,
}

impl FilterPath {
    pub fn new(path: &str) -> Self {
        let case_insensitive = path.starts_with(":case-insensitive:!");
        let path = if case_insensitive {
            path[":case-insensitive:!".len()..].to_string().to_lowercase()
        } else {
            path.into()
        };
        FilterPath {
            path,
            case_insensitive,
        }
    }
    pub fn ends_with(&self, other: &str) -> bool {
        if self.case_insensitive {
            other.to_lowercase().ends_with(&self.path)
        } else {
            other.ends_with(&self.path)
        }
    }
    pub fn starts_with(&self, other: &str) -> bool {
        if self.case_insensitive {
            other.to_lowercase().starts_with(&self.path)
        } else {
            other.starts_with(&self.path)
        }
    }
    pub fn contains(&self, other: &str) -> bool {
        if self.case_insensitive {
            other.to_lowercase().contains(&self.path)
        } else {
            other.contains(&self.path)
        }
    }
}

#[derive(Debug)]
pub struct FilterPathsConfig {
    pub source_file: PathBuf,
    pub target_file: PathBuf,
    pub debug: Debug,
    pub error_log: Option<PathBuf>,
    pub size_min: u64,
    pub size_max: u64,
    pub unique_sizes: bool,
    pub unique_hashes: bool,
    pub blacklist_path_starts: Vec<FilterPath>,
    pub blacklist_path_ends: Vec<FilterPath>,
    pub blacklist_path_contents: Vec<FilterPath>,
    pub whitelist_path_ends: Vec<FilterPath>,
    pub whitelist_path_contents: Vec<FilterPath>,
}

impl Default for FilterPathsConfig {
    fn default() -> Self {
        FilterPathsConfig {
            source_file: Default::default(),
            target_file: Default::default(),
            debug: Default::default(),
            error_log: Default::default(),
            unique_sizes: false,
            unique_hashes: false,
            size_min: std::u64::MIN,
            size_max: std::u64::MAX,
            blacklist_path_starts: Default::default(),
            blacklist_path_ends: Default::default(),
            blacklist_path_contents: Default::default(),
            whitelist_path_ends: Default::default(),
            whitelist_path_contents: Default::default(),
        }
    }
}

pub fn filter_paths(config: FilterPathsConfig) -> Result<()> {
    println!("config: {:?}", config);
    let now = Instant::now();
    let mut ctx = Context::new(&config)?;
    ctx.process()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    println!("Written {} lines {:?}", ctx.lines_written, config.target_file);
    println!("Errors: {} ({:?})", 0, config.error_log);
    Ok(())
}

struct Context<'a> {
    config: &'a FilterPathsConfig,
    input: File,
    output: File,
    lines_written: u64,
}

struct MapValue {
    path: Option<String>,
}

impl MapValue {
    pub fn new(path: String) -> Self {
        MapValue {
            path: Some(path),
        }
    }
}

impl<'a> Context<'a> {
    pub fn new(config: &'a FilterPathsConfig) -> Result<Self> {
        Ok(Context {
            input: File::open(&config.source_file)?,
            output: File::create(&config.target_file)?,
            config,
            lines_written: 0,
        })
    }

    pub fn process(&mut self) -> Result<()> {
        let mut reader = csv::Reader::from_reader(&self.input);
        let mut writer = csv::Writer::from_writer(&self.output);

        let mut sizes: HashMap<u64, MapValue> = HashMap::with_capacity(100_000);
        let mut hashes: HashMap<String, MapValue> = HashMap::with_capacity(100_000);

        let mut dups: HashSet<String> = HashSet::with_capacity(100_000);
        let mut records: Vec<csv::StringRecord> = Vec::with_capacity(100_000);
        for record in reader.records() {
            let record = record?;
            let path = &record[0];
            let size = record[1].parse::<u64>().unwrap();
            if is_filtered(&self.config, path, size) {
                continue;
            }
            records.push(record.clone());
            if self.config.unique_sizes {
                if let Some(other) = sizes.get_mut(&size) {
                    dups.insert(path.into());
                    if let Some(other_path) = other.path.take() {
                        dups.insert(other_path);
                    }
                } else {
                    sizes.insert(size, MapValue::new(path.into()));
                }
            }
            if self.config.unique_hashes {
                let hash: String = (&record[2]).into();
                if let Some(other) = hashes.get_mut(&hash) {
                    dups.insert(path.into());
                    if let Some(other_path) = other.path.take() {
                        dups.insert(other_path);
                    }
                } else {
                    hashes.insert(hash, MapValue::new(path.into()));
                }
            }
        }
        for record in records {
            if (self.config.unique_hashes || self.config.unique_sizes) && !dups.contains(&record[0]) {
                continue;
            }
            writer.write_record(&record)?;
            self.lines_written += 1;
        }
        Ok(())
    }
}

fn is_filtered(config: &FilterPathsConfig, path: &str, size: u64) -> bool {
    if config.blacklist_path_starts.len() > 0
        && config
            .blacklist_path_starts
            .iter()
            .any(|p| p.starts_with(path))
    {
        return true;
    }
    if config.blacklist_path_ends.len() > 0
        && config.blacklist_path_ends.iter().any(|p| p.ends_with(path))
    {
        return true;
    }
    if config.blacklist_path_contents.len() > 0
        && config
            .blacklist_path_contents
            .iter()
            .any(|p| p.contains(path))
    {
        return true;
    }
    if config.whitelist_path_ends.len() > 0
        && !config.whitelist_path_ends.iter().any(|p| p.ends_with(path))
    {
        return true;
    }
    if config.whitelist_path_contents.len() > 0
        && !config
            .whitelist_path_contents
            .iter()
            .any(|p| p.contains(path))
    {
        return true;
    }

    if size < config.size_min || size > config.size_max {
        return true;
    }
    false
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_default_is_not_filtered() {
        let config = FilterPathsConfig::default();
        let actual = is_filtered(
            &config,
            "/mnt/c/Users/Jose/Documents/Old_CDs 1/DVD 2/mIRC/DCC/01 - Oihu.mp3",
            1,
        );
        assert_eq!(actual, false);
    }

    #[test]
    fn test_whitelist_end_ok_is_not_filtered() {
        let mut config = FilterPathsConfig::default();
        config.whitelist_path_ends.push(FilterPath::new(".mp3"));
        let actual = is_filtered(
            &config,
            "/mnt/c/Users/Jose/Documents/Old_CDs 1/DVD 2/mIRC/DCC/01 - Oihu.mp3",
            1,
        );
        assert_eq!(actual, false);
    }

    #[test]
    fn test_whitelist_end_not_ok_is_filtered() {
        let mut config = FilterPathsConfig::default();
        config.whitelist_path_ends.push(FilterPath::new(".png"));
        let actual = is_filtered(
            &config,
            "/mnt/c/Users/Jose/Documents/Old_CDs 1/DVD 2/mIRC/DCC/01 - Oihu.mp3",
            1,
        );
        assert_eq!(actual, true);
    }

    #[test]
    fn test_whitelist_end_not_case_sensitive_ok_is_filtered() {
        let mut config = FilterPathsConfig::default();
        config.whitelist_path_ends.push(FilterPath::new(".MP3"));
        let actual = is_filtered(
            &config,
            "/mnt/c/Users/Jose/Documents/Old_CDs 1/DVD 2/mIRC/DCC/01 - Oihu.mp3",
            1,
        );
        assert_eq!(actual, true);
    }


    #[test]
    fn test_whitelist_end_case_insensitive_ok_is_not_filtered() {
        let mut config = FilterPathsConfig::default();
        config.whitelist_path_ends.push(FilterPath::new(":case-insensitive:!.MP3"));
        let actual = is_filtered(
            &config,
            "/mnt/c/Users/Jose/Documents/Old_CDs 1/DVD 2/mIRC/DCC/01 - Oihu.mp3",
            1,
        );
        assert_eq!(actual, false);
    }
}
