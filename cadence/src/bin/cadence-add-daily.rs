use clap::{crate_version, App, AppSettings, Arg, Values};

use line_item::LineItem;

use cadence::{ID, Writer};
use cadence::app::*;
use cadence::rhythms::Daily;

fn main() {
    let mut app = Application::new_with_var_arg(
        "cadence-add-daily",
        "Creates a new daily rhtythm.");
    let mut root = RootArguments::default();
    app.add_args(&mut root);
    app.parse();

    let app = App::new("cadence-add-daily")
        .author(cadence::AUTHOR_STRING)
        .version(crate_version!())
        .about("Creates a new daily rhtythm.")
        .setting(AppSettings::TrailingVarArg);
    let app = app.arg(Arg::with_name("daily")
        .multiple(true)
        .takes_value(true));
    let matches = app.get_matches();

    let mut daily = String::default();
    let pieces = matches.values_of("daily").unwrap_or(Values::default());
    for piece in pieces {
        daily += " ";
        daily += piece;
    }

    let li = LineItem::new(&daily).unwrap_or(LineItem::new("").unwrap());
    let daily = Daily {
        id: ID::rand(),
        desc: li.desc().to_string(),
    };
    let mut writer = Writer::new(root.root().to_string());
    writer.add_rhythm(&daily).expect("could not write to rhythms");
}
