use clap::{ArgMatches};

use crate::time::{DateTimeOfDay};
use crate::core::Cadence;

////////////////////////////////////////// calculate_start /////////////////////////////////////////

pub fn calculate_start(cadence: &Cadence, matches: &ArgMatches) -> DateTimeOfDay {
    let fallback = cadence.clock.start_of_day();
    let start_str = match matches.value_of("start") {
        Some(x) => x,
        None => return fallback,
    };
    DateTimeOfDay::parse(start_str).expect("expected start to conform to DateTimeOfDay")
}

////////////////////////////////////////// calculate_limit /////////////////////////////////////////

// TODO(rescrv):  Make this flexible.  It only "works" for forward ranges.
pub fn calculate_limit(_: &Cadence/* future */, matches: &ArgMatches, start: DateTimeOfDay) -> DateTimeOfDay {
    let mut fallback = start;
    for _ in 0..7 {
        fallback = fallback.succ_date();
    }
    let limit_str = match matches.value_of("limit") {
        Some(x) => x,
        None => return fallback,
    };
    DateTimeOfDay::parse(limit_str).expect("expected limit to conform to DateTimeOfDay")
}
