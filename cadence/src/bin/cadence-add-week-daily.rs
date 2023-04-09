use chrono::Weekday;
use clap::{crate_version, App, AppSettings, Arg, Values};

use line_item::LineItem;

use cadence::{ID, Writer};
use cadence::app::*;
use cadence::rhythms::{WeekDaily, Slider};

fn main() {
    let mut app = Application::new_with_var_arg(
        "cadence-add-week-daily",
        "Creates a new rhtythm on a given day of the week (dotw).");
    let mut root = RootArguments::default();
    app.add_args(&mut root);
    app.parse();

    let app = App::new("cadence-add-week-daily")
        .author(cadence::AUTHOR_STRING)
        .version(crate_version!())
        .about("Creates a new week/daily rhtythm.")
        .setting(AppSettings::TrailingVarArg);
    let app = app.arg(Arg::with_name("dotw")
        .long("dotw")
        .takes_value(true));
    let app = app.arg(Arg::with_name("week_daily")
        .multiple(true)
        .takes_value(true));
    let matches = app.get_matches();

    // create a single string from all command-line arguments
    let mut week_daily = String::default();
    let pieces = matches.values_of("week_daily").unwrap_or(Values::default());
    for piece in pieces {
        week_daily += " ";
        week_daily += piece;
    }

    // parse the dotw command line argument
    let dotw = match matches.value_of("dotw") {
        Some(dotw) => dotw,
        None => "1",
    };
    let dotw = match dotw.parse::<Weekday>() {
        Ok(dotw) => dotw,
        Err(e) => panic!("Could not parse day of the week: {:?}", e),
    };

    let li = LineItem::new(&week_daily).unwrap_or(LineItem::new("").unwrap());
    let week_daily = WeekDaily {
        id: ID::rand(),
        desc: li.desc().to_string(),
        dotw,
        slider: Slider::default(),
    };
    let mut writer = Writer::new(root.root().to_string());
    writer.add_rhythm(&week_daily).expect("could not write to rhythms");
}
