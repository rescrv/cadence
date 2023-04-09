use std::env;
use std::process::exit;

use clap::{crate_version, App};

use cadence::util;

const ACCEPTABLE_COMMANDS: &[&'static str] = &[
    "done",
    "not-now",
    "add-daily",
    "add-monthly",
    "add-week-daily",
    "add-every-n",
    "list-events",
    "healthcheck",
    "report-basic-schedule",
    "report-smooth-schedule",
    "report-schedule-convergence",
    "health-check",
    "debug-time",
];

/////////////////////////////////////////////// main ///////////////////////////////////////////////

fn main() {
    let mut app = App::new("cadence")
        .author(cadence::AUTHOR_STRING)
        .version(crate_version!())
        .about("Maps e.g. \"cadence 'create'\" to \"cadence-create\" subcommand.");
    // TODO(rescrv): use clappy args to fill in the below.

    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("must provide subcommand");
        // TODO(rescrv):  These expects aren't doing what they were intended for.  I want a help
        // message that says why the help message was needed, and this is not that.
        app.print_help().expect("comparing args < 2");
        exit(1);
    }
    if !args[0].ends_with("cadence") {
        println!("subcommand must end in cadence for substitution reasons");
        app.print_help().expect("checking arg ends in /cadence");
        exit(2);
    }
    if !ACCEPTABLE_COMMANDS.contains(&args[1].as_str()) {
        println!("subcommand isn't in the list of valid subcommands");
        app.print_help().expect("command not valid command");
        exit(4);
    }
    util::run_command(&mut args);
    exit(0);
}
