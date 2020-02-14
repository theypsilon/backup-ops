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

impl Default for Debug {
    fn default() -> Self {
        Self::Off
    }
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

#[derive(Copy, Clone, Debug)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl std::str::FromStr for HashAlgorithm {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "md5" => Ok(Self::Md5),
            "sha1" => Ok(Self::Sha1),
            "sha256" => Ok(Self::Sha256),
            "sha512" => Ok(Self::Sha512),
            _ => Err(format!(
                "No algorithm named '{}', try these instead: md5, sha1, sha256, sha512.",
                s
            )),
        }
    }
}
