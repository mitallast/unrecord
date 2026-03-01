use crate::time::duration::Duration;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeCode {
    hour: u8,
    minute: u8,
    second: u8,
    milli: u16,
}

// base
#[allow(dead_code)]
impl TimeCode {
    pub fn new(hour: u8, minute: u8, second: u8, milli: u16) -> Self {
        Self {
            hour,
            minute,
            second,
            milli,
        }
    }

    pub fn to_string(&self) -> String {
        let Self {
            hour: h,
            minute: m,
            second: s,
            milli: l,
        } = self;

        match (h, l) {
            (0, 0) => format!("{}:{:02}", m, s),
            (0, f) => format!("{}:{:02}.{:03}", m, s, f),
            (h, 0) => format!("{}:{:02}:{:02}", h, m, s),
            (h, f) => format!("{}:{:02}:{:02}.{:03}", h, m, s, f),
        }
    }
}

// getters
#[allow(dead_code)]
impl TimeCode {
    pub fn hour(&self) -> u8 {
        self.hour
    }

    pub fn minute(&self) -> u8 {
        self.minute
    }

    pub fn second(&self) -> u8 {
        self.second
    }

    pub fn milli(&self) -> u16 {
        self.milli
    }
}

// setters
#[allow(dead_code)]
impl TimeCode {
    pub fn at_hour(mut self, hour: u8) -> Self {
        self.hour = hour;
        self
    }

    pub fn at_minute(mut self, minute: u8) -> Self {
        self.minute = minute;
        self
    }

    pub fn at_second(mut self, second: u8) -> Self {
        self.second = second;
        self
    }

    pub fn at_milli(mut self, milli: u16) -> Self {
        self.milli = milli;
        self
    }
}

// truncate
#[allow(dead_code)]
impl TimeCode {
    pub fn truncate(&self, duration: Duration) -> Self {
        let duration = duration.to_millis();
        let truncated = self.to_millis() / duration;
        let millis = truncated * duration;
        Self::from_millis(millis)
    }
}

// convert
#[allow(dead_code)]
impl TimeCode {
    pub fn to_millis(self) -> u64 {
        let mut total: u64 = self.milli as u64;
        total += self.second as u64 * 1_000;
        total += self.minute as u64 * 60_000;
        total += self.hour as u64 * 3_600_000;
        total
    }

    pub fn from_millis(mut total: u64) -> TimeCode {
        let milli = (total % 1_000) as u16;
        total = total / 1_000;
        let second = (total % 60) as u8;
        total = total / 60;
        let minute = (total % 60) as u8;
        total = total / 60;
        let hour = total as u8;

        Self {
            hour,
            minute,
            second,
            milli,
        }
    }

    pub fn from_seconds(mut total: u64) -> TimeCode {
        let second = (total % 60) as u8;
        total = total / 60;
        let minute = (total % 60) as u8;
        total = total / 60;
        let hour = total as u8;

        Self {
            hour,
            minute,
            second,
            milli: 0,
        }
    }

    pub fn from_minutes(mut total: u64) -> TimeCode {
        let minute = (total % 60) as u8;
        total = total / 60;
        let hour = total as u8;

        Self {
            hour,
            minute,
            second: 0,
            milli: 0,
        }
    }

    pub fn from_hours(hour: u8) -> TimeCode {
        Self {
            hour,
            minute: 0,
            second: 0,
            milli: 0,
        }
    }
}

impl Add<Duration> for TimeCode {
    type Output = TimeCode;

    fn add(self, rhs: Duration) -> Self::Output {
        TimeCode::from_millis(self.to_millis() + rhs.to_millis())
    }
}

impl AddAssign<Duration> for TimeCode {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub<Duration> for TimeCode {
    type Output = TimeCode;

    fn sub(self, rhs: Duration) -> Self::Output {
        TimeCode::from_millis(self.to_millis() - rhs.to_millis())
    }
}

impl SubAssign<Duration> for TimeCode {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl Sub<TimeCode> for TimeCode {
    type Output = Duration;

    fn sub(self, rhs: TimeCode) -> Self::Output {
        Duration::Millis(self.to_millis().saturating_sub(rhs.to_millis()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_getters() {
        let time = TimeCode::new(1, 2, 3, 4);
        assert_eq!(time.hour, 1);
        assert_eq!(time.minute, 2);
        assert_eq!(time.second, 3);
        assert_eq!(time.milli, 4);
    }

    #[test]
    fn test_setters() {
        let time = TimeCode::new(0, 0, 0, 0)
            .at_hour(1)
            .at_minute(2)
            .at_second(3)
            .at_milli(4);

        assert_eq!(time.hour, 1);
        assert_eq!(time.minute, 2);
        assert_eq!(time.second, 3);
        assert_eq!(time.milli, 4);
    }

    #[test]
    fn test_truncate_hard_seconds() {
        let actual = TimeCode::new(1, 2, 3, 999).truncate(Duration::Seconds(1));
        let expected = TimeCode::new(1, 2, 3, 0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_truncate_hard_minutes() {
        let actual = TimeCode::new(1, 2, 3, 4).truncate(Duration::Minutes(1));
        let expected = TimeCode::new(1, 2, 0, 0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_truncate_hard_hours() {
        let actual = TimeCode::new(1, 2, 3, 4).truncate(Duration::Hours(1));
        let expected = TimeCode::new(1, 0, 0, 0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_truncate_soft_millis() {
        let actual = TimeCode::new(1, 2, 3, 999).truncate(Duration::Millis(10));
        let expected = TimeCode::new(1, 2, 3, 990);
        assert_eq!(actual, expected);

        let actual = TimeCode::new(1, 2, 3, 999).truncate(Duration::Millis(100));
        let expected = TimeCode::new(1, 2, 3, 900);
        assert_eq!(actual, expected);

        let actual = TimeCode::new(1, 2, 3, 999).truncate(Duration::Millis(500));
        let expected = TimeCode::new(1, 2, 3, 500);
        assert_eq!(actual, expected);

        let actual = TimeCode::new(1, 2, 3, 999).truncate(Duration::Millis(200));
        let expected = TimeCode::new(1, 2, 3, 800);
        assert_eq!(actual, expected);
    }
}
