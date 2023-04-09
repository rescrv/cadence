use std::cmp::Ordering;
use std::fmt::Display;

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use chrono::naive::{MAX_DATE, MIN_DATE};
use chrono::offset::TimeZone;
use chrono_tz::Tz;

pub const DEFAULT_TIMEZONE: &str = "America/Los_Angeles";

///////////////////////////////////////////// TimeOfDay ////////////////////////////////////////////

/// TimeOfDay bucketizes the times of the day.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq)]
pub enum TimeOfDay {
    NoPreference,
    Morning,
    Afternoon,
    Evening,
}

impl TimeOfDay {
    fn parse(s: &str) -> Option<TimeOfDay> {
        match s {
            "nopref" => Some(TimeOfDay::NoPreference),
            "morning" => Some(TimeOfDay::Morning),
            "afternoon" => Some(TimeOfDay::Afternoon),
            "evening" => Some(TimeOfDay::Evening),
            _ => None,
        }
    }
}

impl Default for TimeOfDay {
    fn default() -> TimeOfDay {
        TimeOfDay::NoPreference
    }
}

impl PartialOrd for TimeOfDay {
    fn partial_cmp(&self, rhs: &TimeOfDay) -> Option<Ordering> {
        let lhs = match self {
            TimeOfDay::NoPreference => 0,
            TimeOfDay::Morning => 1,
            TimeOfDay::Afternoon => 2,
            TimeOfDay::Evening => 3,
        };
        let rhs = match rhs {
            TimeOfDay::NoPreference => 0,
            TimeOfDay::Morning => 1,
            TimeOfDay::Afternoon => 2,
            TimeOfDay::Evening => 3,
        };
        Some(lhs.cmp(&rhs))
    }
}

impl Display for TimeOfDay {
    fn fmt(&self, fmter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TimeOfDay::NoPreference => {
                write!(fmter, "NP")
            }
            TimeOfDay::Morning => {
                write!(fmter, "M")
            }
            TimeOfDay::Afternoon => {
                write!(fmter, "A")
            }
            TimeOfDay::Evening => {
                write!(fmter, "E")
            }
        }
    }
}

/////////////////////////////////////////// DateTimeOfDay //////////////////////////////////////////

/// DateTime represents the morning, afternoon, and evening on a particular date.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct DateTimeOfDay {
    pub date: NaiveDate,
    pub when: TimeOfDay,
}

impl DateTimeOfDay {
    pub const BOTTOM: DateTimeOfDay = DateTimeOfDay {
        date: MIN_DATE,
        when: TimeOfDay::NoPreference,
    };

    pub const TOP: DateTimeOfDay = DateTimeOfDay {
        date: MAX_DATE,
        when: TimeOfDay::NoPreference,
    };

    pub fn from_ymd(year: i32, month: u32, day: u32, tod: TimeOfDay) -> DateTimeOfDay {
        let date = NaiveDate::from_ymd(year, month, day);
        DateTimeOfDay {
            date: date,
            when: tod,
        }
    }

    pub fn from_naive_date_time(date_time: NaiveDateTime) -> DateTimeOfDay {
        let date = date_time.date();
        // TODO(rescrv) Document somewhere that this is a choice.
        let when = if date_time.time() < NaiveTime::from_hms(12, 0, 0) {
            TimeOfDay::Morning
        } else if date_time.time() < NaiveTime::from_hms(17, 0, 0) {
            TimeOfDay::Afternoon
        } else {
            TimeOfDay::Evening
        };
        DateTimeOfDay { date, when }
    }

    pub fn parse(from: &str) -> Result<DateTimeOfDay, String> {
        let (date, when) = from.split_at(11);
        let date = match NaiveDate::parse_from_str(date, "%Y-%m-%d:") {
            Ok(date) => date,
            Err(err) => {
                return Err(err.to_string());
            }
        };
        let when = match TimeOfDay::parse(when) {
            Some(x) => x,
            None => TimeOfDay::NoPreference,
        };
        Ok(DateTimeOfDay { date, when })
    }

    pub fn day(&self) -> u32 {
        self.date.day()
    }

    pub fn with_different_time_of_day(&self, when: TimeOfDay) -> DateTimeOfDay {
        DateTimeOfDay {
            date: self.date,
            when,
        }
    }

    pub fn with_different_date(&self, date: chrono::NaiveDate) -> DateTimeOfDay {
        DateTimeOfDay {
            date,
            when: self.when,
        }
    }

    pub fn days_apart(&self, other: Self) -> u64 {
        i64::abs((self.date - other.date).num_days()) as u64
    }

    pub fn prev_date(&self) -> Self {
        DateTimeOfDay {
            date: self.date.pred(),
            when: self.when,
        }
    }

    // TODO(rescrv): change this to succ
    pub fn succ_date(&self) -> Self {
        DateTimeOfDay {
            date: self.date.succ(),
            when: self.when,
        }
    }

    pub fn succ_time_of_day(&self) -> Self {
        if self.when == TimeOfDay::Evening {
            DateTimeOfDay {
                date: self.date.succ(),
                when: TimeOfDay::NoPreference,
            }
        } else {
            let when = match self.when {
                TimeOfDay::NoPreference => TimeOfDay::Morning,
                TimeOfDay::Morning => TimeOfDay::Afternoon,
                TimeOfDay::Afternoon => TimeOfDay::Evening,
                TimeOfDay::Evening => {
                    panic!("this should have been taken care of by the conditional above")
                }
            };
            DateTimeOfDay {
                date: self.date,
                when,
            }
        }
    }
}

impl Default for DateTimeOfDay {
    fn default() -> Self {
        DateTimeOfDay::from_ymd(2020, 2, 2, TimeOfDay::NoPreference)
    }
}

impl Display for DateTimeOfDay {
    fn fmt(&self, fmter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let date = self.date.format("%Y-%m-%d");
        write!(fmter, "{}:{}", date, self.when)
    }
}

/////////////////////////////////////////////// Clock //////////////////////////////////////////////

/// Clock abstracts away impure functions of time.  Always use clock as a future version of this
/// code will likely make clock contain a time zone and adapt the outputs accordingly.
///
/// NOTE(rescrv):  This class implements Copy, so keep it light.
#[derive(Copy, Clone)]
pub struct Clock {
    tz: Tz,
}

impl Clock {
    /// Create a new clock.
    pub fn new(tz: Tz) -> Self {
        Clock {
            tz,
        }
    }

    /// This returns a NaiveDate for the current day, adusted to any user preferences that may be
    /// expressed in the clock.
    pub fn today(&self) -> NaiveDate {
        let date_time = self.raw_now();
        date_time.date()
    }

    /// This returns the start of the day for today.
    pub fn start_of_day(&self) -> DateTimeOfDay {
        let mut now  = self.now();
        now.when = TimeOfDay::NoPreference;
        now
    }

    /// This returns the current time of day as a DateTimeOfDay.  This was done so that it is easy
    /// to get a date with time of day all in one.
    pub fn now(&self) -> DateTimeOfDay {
        let date_time = self.raw_now();
        DateTimeOfDay::from_naive_date_time(date_time)
    }

    pub fn raw_now(&self) -> NaiveDateTime {
        let utc_now: NaiveDateTime = Utc::now().naive_local();
        self.tz.from_utc_datetime(&utc_now).naive_local()
    }
}

// TODO(rescrv): testing
