use cadence::Events;
use cadence::app::*;
use cadence::core::FILE_EVENTS;
use cadence::util::path_relative_to_root;

/////////////////////////////////////////////// main ///////////////////////////////////////////////

pub fn main() {
    let mut app = Application::new_with_var_arg(
        "cadence-list-events",
        "List events logged in Cadence");
    let mut root = RootArguments::default();
    app.add_args(&mut root);
    app.parse();

    let path = path_relative_to_root(&root.root(), FILE_EVENTS);
    let events = Events::new(&path).expect("could not load all events");
    for event in events.iter() {
        println!("{}", event);
    }
}
