use clap::{crate_version, App, AppSettings, Arg, Values};

use line_item::LineItem;

use cadence::{ID, Writer};
use cadence::app::*;
use cadence::rhythms::{Monthly, Slider};

fn main() {
    let mut app = Application::new_with_var_arg(
        "cadence-add-monthly",
        "Creates a new rhtythm on a given day of the month (dotm).");
    let mut root = RootArguments::default();
    app.add_args(&mut root);
    app.parse();

    let app = App::new("cadence-add-monthly")
        .author(cadence::AUTHOR_STRING)
        .version(crate_version!())
        .about("Creates a new monthly rhtythm.")
        .setting(AppSettings::TrailingVarArg);
    let app = app.arg(Arg::with_name("dotm")
        .long("dotm")
        .takes_value(true));
    let app = app.arg(Arg::with_name("monthly")
        .multiple(true)
        .takes_value(true));
    let matches = app.get_matches();

    let mut monthly = String::default();
    let pieces = matches.values_of("monthly").unwrap_or(Values::default());
    for piece in pieces {
        monthly += " ";
        monthly += piece;
    }

    let dotm = match matches.value_of("dotm") {
        Some(dotm) => dotm,
        None => "1",
    };
    let dotm = cadence::util::parse_u32(dotm).expect("n value out of bounds");
    if dotm < 1 || dotm > 31 {
        panic!("dotm out of bounds");
    }

    let li = LineItem::new(&monthly).unwrap_or(LineItem::new("").unwrap());
    let monthly = Monthly {
        id: ID::rand(),
        desc: li.desc().to_string(),
        dotm: dotm as u32,
        slider: Slider::default(),
    };
    let mut writer = Writer::new(root.root().to_string());
    writer.add_rhythm(&monthly).expect("could not write to rhythms");
}
