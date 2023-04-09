use chrono::NaiveDate;
use chrono::Weekday;
use chrono::Datelike;

use line_item::LineItem;

use crate::ID;
use crate::DateTimeOfDay;

////////////////////////////////////////////// Slider //////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Slider {
    pub before: u32,
    pub after: u32,
}

impl std::fmt::Display for Slider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.before, self.after)
    }
}

////////////////////////////////////////////// Rhythm //////////////////////////////////////////////

/// Rhythm is a recurring event.
pub trait Rhythm {
    fn id(&self) -> ID;

    fn starting_beat(&self, start: DateTimeOfDay, last_seen: DateTimeOfDay) -> DateTimeOfDay {
        // This beat should necessarily the first beat after last_seen.
        let mut beat = self.next_beat(last_seen);
        // Somethimes slider will move to before a given date, e.g. a Thursday task move to
        // Wednesday.  skip_beat_within_slider should == true says that we shouldn't take the beat
        // when it's within slider.before of the last seen.  If it is, advance.
        if self.skip_beat_within_slider() && last_seen.days_apart(beat) < self.slider().before as u64 {
            beat = self.next_beat(beat);
        }
        while beat < start {
            beat = self.next_beat(beat);
        }
        beat
    }

    fn next_naive_beat(&self, date: NaiveDate) -> NaiveDate;

    fn next_beat(&self, dtod: DateTimeOfDay) -> DateTimeOfDay {
        let mut dtod = dtod;
        dtod.date = self.next_naive_beat(dtod.date);
        dtod
    }

    fn prev_naive_beat(&self, date: NaiveDate) -> NaiveDate;

    fn prev_beat(&self, dtod: DateTimeOfDay) -> DateTimeOfDay {
        let mut dtod = dtod;
        dtod.date = self.prev_naive_beat(dtod.date);
        dtod
    }

    fn line_item(&self) -> LineItem;

    fn human_line(&self) -> String;

    fn slider(&self) -> Slider {
        Slider::default()
    }

    fn skip_beat_within_slider(&self) -> bool {
        false
    }
}

/////////////////////////////////////////////// Daily //////////////////////////////////////////////

/// A process that must be done each day.  Daily processes can only be canceled; they cannot
/// rescheduled because every other day has a Daily already.
#[derive(Clone, Debug)]
pub struct Daily {
    /// Unique ID for the cycle.  It's expected to have multiple entries with the same ID in a
    /// schedule.
    pub id: ID,
    /// Command-free description of the process.
    pub desc: String,
}

impl Rhythm for Daily {
    fn id(&self) -> ID {
        self.id.clone()
    }

    fn next_naive_beat(&self, date: NaiveDate) -> NaiveDate {
        date.succ()
    }

    fn prev_naive_beat(&self, date: NaiveDate) -> NaiveDate {
        date.pred()
    }

    fn line_item(&self) -> LineItem {
        unwrap_line_item(&self.id, format!("{} {} type:daily", self.desc, self.id))
    }

    fn human_line(&self) -> String {
        format!("{} every day", &self.desc)
    }
}

////////////////////////////////////////////// Monthly /////////////////////////////////////////////

/// A process that must be done once per month, on a particular day of the month.
#[derive(Clone, Debug)]
pub struct Monthly {
    /// Unique ID for the cycle.  It's expected to have multiple entries with the same ID in a
    /// schedule.
    pub id: ID,
    /// Command-free description of the process.
    pub desc: String,
    /// Day of the month.  An index into the day of the month 1-index
    pub dotm: u32,
    /// Spread how far into the past.0 or the future.1.  This allows for e.g. paying for a car
    /// payment early or wanting something to happen about mid-month, but allow it to move around.
    pub slider: Slider,
}

impl Rhythm for Monthly {
    fn id(&self) -> ID {
        self.id.clone()
    }

    fn next_naive_beat(&self, date: NaiveDate) -> NaiveDate {
        let mut date = if date.day() == self.dotm {
            date.succ()
        } else {
            date
        };
        while date.day() != self.dotm {
            date = date.succ();
        }
        date
    }

    fn prev_naive_beat(&self, date: NaiveDate) -> NaiveDate {
        let mut date = if date.day() == self.dotm {
            date.pred()
        } else {
            date
        };
        while date.day() != self.dotm {
            date = date.pred();
        }
        date
    }

    fn line_item(&self) -> LineItem {
        unwrap_line_item(&self.id, format!( "{} {} type:monthly dotm:{} slider:{}", self.desc, self.id, self.dotm, self.slider))
    }

    fn human_line(&self) -> String {
        format!("{} every {} day of the month", self.desc.clone(), self.dotm)
    }

    fn slider(&self) -> Slider {
        self.slider
    }

    fn skip_beat_within_slider(&self) -> bool {
        true
    }
}

///////////////////////////////////////////// WeekDaily ////////////////////////////////////////////

/// A process that should be done on a particular day of the week.
#[derive(Clone, Debug)]
pub struct WeekDaily {
    /// Unique ID for the cycle.  It's expected to have multiple entries with the same ID in a
    /// schedule.
    pub id: ID,
    /// Command-free description of the process.
    pub desc: String,
    /// Day of the week.  Uses a chrono::Weekday.
    // TODO(rescrv): pub use chrono::Weekday as Weekday at top level.  Don't forget to change the
    // comment.
    pub dotw: Weekday,
    /// Spread how far into the past.0 or the future.1.  This allows for e.g. putting the trash out
    /// early, or allow a Friday evening task to happen Saturday evening as well.
    pub slider: Slider,
}

impl Rhythm for WeekDaily {
    fn id(&self) -> ID {
        self.id.clone()
    }

    fn next_naive_beat(&self, date: NaiveDate) -> NaiveDate {
        let mut date = if date.weekday() == self.dotw {
            date.succ()
        } else {
            date
        };
        while date.weekday() != self.dotw {
            date = date.succ();
        }
        date
    }

    fn prev_naive_beat(&self, date: NaiveDate) -> NaiveDate {
        let mut date = if date.weekday() == self.dotw {
            date.pred()
        } else {
            date
        };
        while date.weekday() != self.dotw {
            date = date.pred();
        }
        date
    }

    fn line_item(&self) -> LineItem {
        unwrap_line_item(&self.id, format!("{} {} type:week-daily dotw:{} slider:{}", self.desc, self.id, self.dotw, self.slider))
    }

    fn human_line(&self) -> String {
        format!("{} every {}", self.desc.clone(), self.dotw)
    }

    fn slider(&self) -> Slider {
        self.slider
    }

    fn skip_beat_within_slider(&self) -> bool {
        true
    }
}

//////////////////////////////////////////// EveryNDays ////////////////////////////////////////////

/// A flexible process that recurs at approximately every N days.  The scheduling system takes into
/// account the N value and decides the flexibility of the process based upon history.
#[derive(Clone, Debug)]
pub struct EveryNDays {
    /// Unique ID for the cycle.  It's expected to have multiple entries with the same ID in a
    /// schedule.
    pub id: ID,
    /// Command-free description of the process.
    pub desc: String,
    /// Cycle recurs every n days
    pub n: u32,
    /// Spread how far into the past.0 or the future.1.  This allows the cycle to move around.
    pub slider: Slider,
}

impl Rhythm for EveryNDays {
    fn id(&self) -> ID {
        self.id.clone()
    }

    fn next_naive_beat(&self, date: NaiveDate) -> NaiveDate {
        let mut date = date;
        for _ in 0..self.n {
            date = date.succ();
        }
        date
    }

    fn prev_naive_beat(&self, date: NaiveDate) -> NaiveDate {
        let mut date = date;
        for _ in 0..self.n {
            date = date.succ();
        }
        date
    }

    fn line_item(&self) -> LineItem {
        unwrap_line_item(&self.id, format!("{} {} type:every-n-days n:{} slider:{}", self.desc, self.id, self.n, self.slider))
    }

    fn human_line(&self) -> String {
        format!("{} every {} days", self.desc.clone(), self.n)
    }

    fn slider(&self) -> Slider {
        self.slider
    }
}

/////////////////////////////////////////////// util ///////////////////////////////////////////////

fn unwrap_line_item(id: &ID, line_item: String) -> LineItem {
    match LineItem::new(&line_item) {
        Some(lr) => lr,
        None => {
            // TODO(rescrv): make a test that tests this is true, even if it's making this string a
            // top level part of this module.  No guarantee on ID, but that's supposed to be valid
            // because it's in form.
            let line_item = format!{"{} type:error status:invalid invalid line represntation", id};
            LineItem::new(&line_item).expect("this representation must always be valid")
        },
    }
}

#[cfg(test)]
mod tests {
    // TODO(rescrv): Tests
    // - daily line_item
    // - monthly line_item
    // - week-dailly line_item
    // - every-n-days  line_item
}
