use cadence::*;
use cadence::app::*;
use cadence::reporting::Schedule as ScheduleTrait;
use cadence::reporting::smooth_schedule::Schedule;

// TODO(rescrv):  De-dupe this with other schedules because it's only static strings and imports
// that change.  Literally three of these forty lines.  Only worth it if there's a third schedule.
fn main() {
    let mut app = Application::new(
        "cadence-report-smooth-schedule",
        "Create a schedule with a smoothed rhythm to the beats.");
    let mut root = RootArguments::default();
    let mut disp = DisplayArguments::default();
    let mut tz = TimezoneArguments::default();
    let mut win = WindowArguments::new(
        WindowDirection::Forward,
        "Starting date for the schedule.",
        "Ending date for the schedule.  It will not be included in the readout.");
    app.add_args(&mut root);
    app.add_args(&mut disp);
    app.add_args(&mut tz);
    app.add_args(&mut win);
    app.parse();

    let clock = tz.clock();
    let (start, limit) = win.window(&clock);
    let cadence = Cadence::new(clock, &root.root()).expect("cadence should instantiate");
    let sched = Schedule::new(&cadence, start, limit).expect("smooth schedule should instantiate");

    match disp.display() {
        DisplayMode::Plumbing => {
            for event in sched.plumbing() {
                println!("{}", event);
            }
        }
        DisplayMode::Porcelain => {
            for event in sched.porcelain() {
                println!("{}", event);
            }
        }
    }
}
