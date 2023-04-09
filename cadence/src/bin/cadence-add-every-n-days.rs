use clap::{crate_version, App, AppSettings, Arg, Values};

use line_item::LineItem;

use cadence::{ID, Writer};
use cadence::app::*;
use cadence::rhythms::{EveryNDays, Slider};

fn main() {
    let mut app = Application::new_with_var_arg(
        "cadence-add-every-n-days",
        "Creates a new rhtythm that happens every so many days (n).");
    let mut root = RootArguments::default();
    app.add_args(&mut root);
    app.parse();

    let app = App::new("cadence-add-every-n-days")
        .author(cadence::AUTHOR_STRING)
        .version(crate_version!())
        .about("Creates a new cyclical rhtythm.")
        .setting(AppSettings::TrailingVarArg);
    let app = app.arg(Arg::with_name("n")
        .short("n")
        .takes_value(true));
    let app = app.arg(Arg::with_name("desc")
        .multiple(true)
        .takes_value(true));
    let matches = app.get_matches();

    let mut every_n = String::default();
    let pieces = matches.values_of("desc").unwrap_or(Values::default());
    for piece in pieces {
        every_n += " ";
        every_n += piece;
    }

    let n = match matches.value_of("n") {
        Some(dotm) => dotm,
        None => "1",
    };
    let n = cadence::util::parse_u32(n).expect("n value out of bounds");
    if n < 1 || n > 365 {
        panic!("dotm out of bounds [1, 365]");
    }

    let li = LineItem::new(&every_n).unwrap_or(LineItem::new("").unwrap());
    let every_n = EveryNDays {
        id: ID::rand(),
        desc: li.desc().to_string(),
        n: n as u32,
        slider: Slider::default(),
    };
    let mut writer = Writer::new(root.root().to_string());
    writer.add_rhythm(&every_n).expect("could not write to rhythms");
}
