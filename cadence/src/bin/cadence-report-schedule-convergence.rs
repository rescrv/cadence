use cadence::*;
use cadence::app::*;
use cadence::reporting::FileSchedule;
use cadence::reporting::schedule_convergence::convergence;

fn main() {
    let mut app = Application::new(
        "cadence-report-schedule-convergence",
        "List events logged in Cadence");
    let mut root = RootArguments::default();
    let mut tz = TimezoneArguments::default();
    app.add_args(&mut root);
    app.add_args(&mut tz);
    app.parse();

    let cadence = Cadence::new(tz.clock(), &root.root()).expect("cadence should instantiate");
    // TODO(rescrv): allow different files.
    let sched = FileSchedule::new("/dev/stdin").expect("failed to parse file schedule");
    let today = cadence.clock.start_of_day();
    let until = convergence(&cadence, &sched, today);
    println!("converge on {}", until);
}
