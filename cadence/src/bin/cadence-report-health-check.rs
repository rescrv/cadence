use clap::{crate_version, App, Arg};
use cadence::cmdline::calculate_start;

use cadence::*;
use cadence::app::*;
use cadence::reporting::health_check::health_check;

fn main() {
    let mut app = Application::new(
        "cadence-report-health-check",
        "Report the health of an individual on a daily basis based upon their activity.");
    let mut root = RootArguments::default();
    let mut tz = TimezoneArguments::default();
    app.add_args(&mut root);
    app.add_args(&mut tz);
    app.parse();

    let app = App::new("cadence-report-health-check")
        .author(cadence::AUTHOR_STRING)
		.version(crate_version!())
        .about("Guess the health or business of an individual.");
    let app = app.arg(Arg::with_name("start")
        .long("--start")
        .takes_value(true)
        .value_name("START")
        .help("Start the schedule at this time."));
    let app = app.arg(Arg::with_name("limit")
        .long("--limit")
        .takes_value(true)
        .value_name("LIMIT")
        .help("Stop the schedule at this time; it will not be included in the schedule."));
    // TODO(rescrv):  Take a window, don't just make one.
    let matches = app.get_matches();
    let cadence = Cadence::new(tz.clock(), &root.root()).expect("cadence should instantiate");
    // TODO(rescrv):  This is horribly broken.
    let limit = calculate_start(&cadence, &matches);
    let mut start = limit;
    for _ in 0..30 {
        start = start.prev_date();
    }
    while start < limit {
        let score = health_check(&cadence, start);
        println!("{} {}", start, score);
        start = start.succ_date();
    }
}
