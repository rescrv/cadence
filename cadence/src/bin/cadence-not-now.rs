use std::env;

use cadence::{Clock, ID, Writer};

/////////////////////////////////////////////// main ///////////////////////////////////////////////

pub fn main() {
    // TODO(rescrv):  Use clap for the arg parsing.
    let args: Vec<String> = env::args().collect();
    let root = cadence::util::get_root_dir().expect("cannot find data directory");
    let mut writer = Writer::new(root);
    // TODO(rescrv):  Don't hardcode this as America/Los_Angeles.
    let tz_string = "America/Los_Angeles";
    let clock = Clock::new(tz_string.parse().unwrap()/*XXX*/);
    for idx in 1..args.len() {
        writer.notnow(&clock, ID::new(args[idx].clone()).unwrap()).unwrap();
    }
}
