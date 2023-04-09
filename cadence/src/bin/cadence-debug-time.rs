use chrono::{NaiveDateTime, Utc};
use chrono::offset::TimeZone;
use chrono_tz::Tz;

use cadence::DEFAULT_TIMEZONE;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    // Make sure we always do some debugging and default to the author's local time.
    if args.len() < 1 {
        args.push(DEFAULT_TIMEZONE.to_string());
    }

    let utc_now: NaiveDateTime = Utc::now().naive_local();
    println!("UTC: {}", utc_now);

    for arg in args.iter() {
        let tz: Tz = arg.parse().unwrap();
        let tz_aware = tz.from_utc_datetime(&utc_now);
        let tz_agnostic = tz_aware.naive_local();
        println!("\n{}\naware: {}\nnaive: {}", arg, tz_aware, tz_agnostic);
    }
}
