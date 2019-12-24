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
