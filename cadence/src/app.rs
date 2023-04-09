use chrono_tz::Tz;

use clap::{crate_version, App, AppSettings, Arg, ArgMatches};

use crate::AUTHOR_STRING;
use crate::DEFAULT_TIMEZONE;
use crate::Clock;
use crate::DateTimeOfDay;
use crate::util;

//////////////////////////////////////////// Application ///////////////////////////////////////////

pub struct Application<'a, 'b, 'c> {
    app: App<'b, 'c>,
    args: Vec<&'a mut dyn ArgumentSet>,
}

impl<'a, 'b, 'c> Application<'a, 'b, 'c> {
    pub fn new(exe: &'static str, help: &'static str) -> Self {
        let app = App::new(exe)
            .author(AUTHOR_STRING)
		    .version(crate_version!())
            .about(help);
        let args = Vec::new();
        Application {
            app,
            args,
        }
    }

    pub fn new_with_var_arg(exe: &'static str, help: &'static str) -> Self {
        let app = App::new(exe)
            .author(AUTHOR_STRING)
		    .version(crate_version!())
            .about(help)
            // TODO(rescrv):  Lift this call so there's Application::setting.
            .setting(AppSettings::TrailingVarArg);
        let args = Vec::new();
        Application {
            app,
            args,
        }
    }

    pub fn add_args(&mut self, args: &'a mut dyn ArgumentSet) {
        self.app = args.arg(self.app.clone());
        self.args.push(args);
    }

    pub fn parse(mut self) {
        let matches = self.app.get_matches();
        for arg in self.args.iter_mut() {
            arg.parse(&matches);
        }
    }
}

//////////////////////////////////////////// ArgumentSet ///////////////////////////////////////////

pub trait ArgumentSet {
    fn arg<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b>;
    fn parse(&mut self, matches: &ArgMatches);
}

/////////////////////////////////////////// RootArguments //////////////////////////////////////////

#[derive(Debug)]
pub struct RootArguments {
    // Convert to a Path type if one conveniently exists.
    root: String,
}

impl RootArguments {
    pub fn root(&self) -> &str {
        &self.root
    }
}

impl Default for RootArguments {
    fn default() -> Self {
        RootArguments {
            root: ".".to_string(),
        }
    }
}

impl ArgumentSet for RootArguments {
    fn arg<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(Arg::with_name("root")
            .long("root")
            .takes_value(true)
            .help("Root directory for Cadence data."))
    }

    fn parse(&mut self, matches: &ArgMatches) {
        self.root = match matches.value_of("root") {
            Some(root) => root.to_string(),
            None => util::get_root_dir().expect("could not find root directory"),
        };
    }
}

///////////////////////////////////////// DisplayArguments /////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum DisplayMode {
    Plumbing,
    Porcelain,
}

#[derive(Debug)]
pub struct DisplayArguments {
    mode: DisplayMode
}

impl DisplayArguments {
    pub fn display(&self) -> DisplayMode {
        self.mode
    }
}

impl Default for DisplayArguments {
    fn default() -> Self {
        DisplayArguments {
            mode: DisplayMode::Porcelain,
        }
    }
}

impl ArgumentSet for DisplayArguments {
    fn arg<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b> {
        let app = app.arg(Arg::with_name("plumbing")
            .long("--plumbing")
            .help("Print out the plubming format, one scheduled rhythm per line."));
        let app = app.arg(Arg::with_name("porcelain")
            .long("--porcelain")
            .help("Print out the porcelain format, one scheduled rhythm per line."));
        app
    }

    fn parse(&mut self, matches: &ArgMatches) {
        if matches.is_present("plumbing") {
            self.mode = DisplayMode::Plumbing;
        }
        if matches.is_present("porcelain") {
            self.mode = DisplayMode::Porcelain;
        }
    }
}

///////////////////////////////////////// TimezoneArguments ////////////////////////////////////////

#[derive(Debug)]
pub struct TimezoneArguments {
    tz: Tz
}

impl TimezoneArguments {
    pub fn clock(&self) -> Clock {
        Clock::new(self.tz.clone())
    }
}

impl Default for TimezoneArguments {
    fn default() -> Self {
        Self {
            // TODO(rescrv): Test this expectation
            tz: DEFAULT_TIMEZONE.parse().expect("expected DEFAULT_TIMEZONE to parse"),
        }
    }
}

impl ArgumentSet for TimezoneArguments {
    fn arg<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(Arg::with_name("timezone")
            .long("timezone")
            .takes_value(true)
            .help("Timezone for operation (spaces to _) e.g. \"America/Los_Angeles\""))
    }

    fn parse(&mut self, matches: &ArgMatches) {
        let tz_string = matches.value_of("timezone").unwrap_or(DEFAULT_TIMEZONE);
        self.tz = match tz_string.parse() {
            Ok(x) => x,
            Err(e) => panic!("could not understand timezone {}: {}", tz_string, e),
        };
    }
}

////////////////////////////////////////// WindowArguments /////////////////////////////////////////

#[derive(Debug, Eq, PartialEq)]
pub enum WindowDirection {
    Forward,
    Backward,
}

#[derive(Debug)]
pub struct WindowArguments {
    direction: WindowDirection,
    start_help: &'static str,
    limit_help: &'static str,
    start: Option<DateTimeOfDay>,
    limit: Option<DateTimeOfDay>,
}

impl WindowArguments {
    pub fn new(dir: WindowDirection, start_help: &'static str, limit_help: &'static str) -> Self {
        WindowArguments {
            direction: dir,
            start_help,
            limit_help,
            start: None,
            limit: None,
        }
    }

    pub fn window(&self, clock: &Clock) -> (DateTimeOfDay, DateTimeOfDay) {
        let start = match self.start {
            Some(x) => x,
            None => clock.start_of_day(),
        };
        let limit = match self.limit {
            Some(x) => x,
            None => {
                let mut limit = start;
                for _ in 0..7 {
                    match self.direction {
                        WindowDirection::Forward => limit = limit.succ_date(),
                        WindowDirection::Backward => limit = limit.prev_date(),
                    }
                }
                limit
            },
        };
        match self.direction {
            WindowDirection::Forward => {
                assert!(start <= limit, "Limit must be greater than start for forward ranges: ({}, {})", start, limit);
                (start, limit)
            },
            WindowDirection::Backward => {
                assert!(limit <= start, "Start must be greater than limit for backward ranges: ({}, {})", limit, start);
                (limit, start)
            },
        }
    }
}

impl ArgumentSet for WindowArguments {
    fn arg<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b> {
        let app = app.arg(Arg::with_name("start")
            .long("--start")
            .takes_value(true)
            .value_name("START")
            .help(self.start_help));
        let app = app.arg(Arg::with_name("limit")
            .long("--limit")
            .takes_value(true)
            .value_name("LIMIT")
            .help(self.limit_help));
        app
    }

    fn parse(&mut self, matches: &ArgMatches) {
        self.start = match matches.value_of("start") {
            Some(x) => Some(DateTimeOfDay::parse(x).expect(&format!("expected start/{} to conform to DateTimeOfDay", x))),
            None => None,
        };
        self.limit = match matches.value_of("limit") {
            Some(x) => Some(DateTimeOfDay::parse(x).expect(&format!("expected limit/{} to conform to DateTimeOfDay", x))),
            None => None,
        };
    }
}
