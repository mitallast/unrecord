#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Duration {
    Hours(u64),
    Minutes(u64),
    Seconds(u64),
    Millis(u64),
}

impl Duration {
    pub fn to_millis(self) -> u64 {
        match self {
            Duration::Hours(hours) => hours * 3_600_000,
            Duration::Minutes(minutes) => minutes * 60_000,
            Duration::Seconds(seconds) => seconds * 1_000,
            Duration::Millis(millis) => millis,
        }
    }
}
