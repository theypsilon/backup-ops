use crate::common::Debug;
use crate::internals::Reporter;
use anyhow::{anyhow, Result};
use num_format::{Locale, ToFormattedString};
use regex::Regex;
use size_format::SizeFormatterSI;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Default)]
pub struct CopyFilesConfig {
    pub source_file: PathBuf,
    pub target_folder: PathBuf,
    pub debug: Debug,
    pub flatten_output: bool,
    pub error_log: Option<PathBuf>,
}

pub fn copy_files(config: CopyFilesConfig) -> Result<()> {
    println!("COPY FILES | config: {:?}", config);
    let now = Instant::now();
    let mut ctx = Context::new(config)?;
    ctx.process()?;
    println!("Duration: {:#?}", (Instant::now() - now));
    println!(
        "Copied {} files {:?}",
        ctx.lines_written.to_formatted_string(&Locale::en),
        ctx.config.target_folder
    );
    println!(
        "Disk space taken: {}B",
        SizeFormatterSI::new(ctx.copied_size)
    );
    println!(
        "Errors: {} ({:?})",
        ctx.reporter.error_count().to_formatted_string(&Locale::en),
        ctx.config.error_log
    );
    Ok(())
}

struct Context {
    config: CopyFilesConfig,
    reporter: Reporter,
    lines_written: u64,
    copied_size: u64,
}

impl Context {
    pub fn new(config: CopyFilesConfig) -> Result<Self> {
        Ok(Context {
            reporter: Reporter::new(config.error_log.clone(), config.debug),
            config,
            lines_written: 0,
            copied_size: 0,
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
        std::fs::create_dir(&Path::new(&self.config.target_folder))?;
        let mut target_path_generator =
            TargetPathGenerator::new(self.config.flatten_output, &self.config.target_folder);
        reader.seek(start_pos)?;
        let mut current_size: u64 = 0;
        for record in reader.records() {
            let record = record?;

            if total_size > 0 {
                let size = record[1].parse::<u64>()?;

                current_size += size;
                print!(
                    "\r{:.2}%        ",
                    (current_size as f64 / total_size as f64) * 100.0
                );
            }

            let source_path = Path::new(&record[0]);
            let target_path = target_path_generator.get_target_path(source_path)?;
            if !self.config.flatten_output {
                std::fs::create_dir_all(&target_path.parent().ok_or(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Parent should be a dir",
                ))?)?;
            }
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

struct TargetPathGenerator {
    flatten: bool,
    target_folder: PathBuf,
    paths: HashSet<OsString>,
    regex: Regex,
}

impl TargetPathGenerator {
    pub fn new(flatten: bool, target_folder: &PathBuf) -> Self {
        TargetPathGenerator {
            flatten,
            target_folder: target_folder.clone(),
            paths: HashSet::new(),
            regex: Regex::new(r".* - Copy \((?P<times>\d+)\)$").unwrap(),
        }
    }
}

impl TargetPathGenerator {
    fn get_target_path(&mut self, source_path: &Path) -> Result<PathBuf> {
        if self.flatten {
            let mut file_name =
                OsString::from(source_path.file_name().ok_or(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Can't get filename from source path",
                ))?);
            while self.paths.contains(&file_name.to_os_string()) {
                let mut file_stem = Path::new(&file_name).file_stem().unwrap().to_owned();
                let mut changed = false;
                if let Some(stem_str) = file_stem.to_str() {
                    if let Some(caps) = self.regex.captures(&stem_str) {
                        let times = &caps["times"];
                        if let Ok(count) = times.parse::<u64>() {
                            let limit_to_common_part =
                                file_stem.len() - (times.len() + " - Copy ()".len());
                            file_stem = match file_stem.into_string() {
                                Ok(string) => OsString::from(&string[0..limit_to_common_part]),
                                Err(string) => {
                                    return Err(anyhow!(
                                        "Wrong unicode for this one: {:?}",
                                        string
                                    ));
                                }
                            };
                            file_stem.push(&format!(" - Copy ({})", count + 1));
                            changed = true;
                        }
                    }
                }
                if !changed {
                    file_stem.push(" - Copy (1)");
                }
                if let Some(extension) = source_path.extension() {
                    file_stem.push(".");
                    file_stem.push(extension);
                }
                file_name = file_stem;
            }
            let target_path = self.target_folder.join(file_name);
            Ok(target_path)
        } else {
            let target_path = self.target_folder.join(if source_path.has_root() {
                source_path.components().skip(1).collect::<PathBuf>()
            } else {
                source_path.to_owned()
            });
            Ok(target_path)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn get_target_path(flatten: bool, source: &str, target: &str) -> String {
        let mut gen = TargetPathGenerator::new(flatten, &PathBuf::from(target));
        format!("{:?}", gen.get_target_path(Path::new(source)).unwrap())
    }

    fn get_target_path_substitutes(
        flatten: bool,
        source: &str,
        target: &str,
        already: &[&str],
    ) -> String {
        let mut gen = TargetPathGenerator::new(flatten, &PathBuf::from(target));
        already
            .iter()
            .for_each(|path| assert_eq!(true, gen.paths.insert(std::ffi::OsString::from(path))));
        format!("{:?}", gen.get_target_path(Path::new(source)).unwrap())
    }

    macro_rules! eq_tests {
        ( $( $name:ident: $input:expr => $expected:expr;)* ) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!($input, $expected);
                }
            )*
        };
    }

    eq_tests! {
        get_not_flatten_target_path_without_slash_ending: get_target_path(false, "/la/le/li.mp3", "/out") => "\"/out/la/le/li.mp3\"";
        get_not_flatten_target_path_with_slash_ending: get_target_path(false, "/la/le/li.mp3", "/out/") => "\"/out/la/le/li.mp3\"";
        get_flatten_target_path_without_slash_ending: get_target_path(true, "/la/le/li.mp3", "/out") => "\"/out/li.mp3\"";
        get_flatten_target_path_with_slash_ending: get_target_path(true, "/la/le/li.mp3", "/out/") => "\"/out/li.mp3\"";
        get_flatten_target_path_with_first_substitute_ending: get_target_path_substitutes(true, "/la/le/li.mp3", "/out/", &["li.mp3"]) => "\"/out/li - Copy (1).mp3\"";
        get_flatten_target_path_with_second_substitute_ending: get_target_path_substitutes(
                    true,
                    "/la/le/li.mp3",
                    "/out/",
                    &["li.mp3", "li - Copy (1).mp3"]
                ) => "\"/out/li - Copy (2).mp3\"";
        get_flatten_target_path_with_10th_substitute_ending: get_target_path_substitutes(
                    true,
                    "/la/le/li - Copy (9).mp3",
                    "/out/",
                    &["li - Copy (9).mp3"]
                ) => "\"/out/li - Copy (10).mp3\"";
        get_flatten_target_path_with_11th_substitute_ending: get_target_path_substitutes(
                    true,
                    "/la/le/li - Copy (10).mp3",
                    "/out/",
                    &["li - Copy (10).mp3"]
                ) => "\"/out/li - Copy (11).mp3\"";
        get_flatten_target_path_with_substitute_ending_without_extension: get_target_path_substitutes(
                    true,
                    "/la/le/li - Copy (10)",
                    "/out/",
                    &["li - Copy (10)"]
                ) => "\"/out/li - Copy (11)\"";
        get_flatten_target_misleading_path_uses_first_substitute: get_target_path_substitutes(true, "/la/le/li - Copy (x)", "/out/", &["li - Copy (x)"]) => "\"/out/li - Copy (x) - Copy (1)\"";
    }
}
