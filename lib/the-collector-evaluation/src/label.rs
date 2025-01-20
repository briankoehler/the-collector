use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Hash)]
#[non_exhaustive]
pub enum IntLevel {
    Not,
    Insignificant,
    Normal,
    Big,
    Turbo,
}

/// TODO: Use these
#[derive(Debug)]
#[non_exhaustive]
pub enum IntLabel {
    /// Inner value is the deaths per minute
    FrequentDeaths(f32),
    /// Inner value is the percentage of time spent dead
    LongTimeDead(f32),
}
