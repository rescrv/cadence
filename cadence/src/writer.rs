use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

use line_item::iter::RawIterator;

use crate::command_words::COMMAND_ID;
use crate::rhythms::*;
use crate::time::Clock;
use crate::Error;
use crate::ID;

////////////////////////////////////////////// Writer //////////////////////////////////////////////

pub struct Writer {
    root: String,
}

impl Writer {
    pub fn new(root: String) -> Self {
        Writer { root }
    }

    pub fn done(&mut self, clock: &Clock, id: ID) -> Result<(), Error> {
        self.log_line(clock, id, "done")
    }

    pub fn notnow(&mut self, clock: &Clock, id: ID) -> Result<(), Error> {
        self.log_line(clock, id, "notnow")
    }

    fn log_line(&mut self, clock: &Clock, id: ID, status: &'static str) -> Result<(), Error> {
        let (rhythms, _) = file_names(&self.root);
        let mut iter = RawIterator::new(&rhythms)?;

        let item = loop {
            let item = match iter.next() {
                Some(Ok(item)) => item,
                Some(Err(e)) => return Err(e.into()),
                None => return Err(Error::StringErrorXXX("no such item".to_string())),
            };
            let internal_id = match item.lookup(COMMAND_ID) {
                Some(x) => x,
                None => unimplemented!(),
            };
            let internal_id = match ID::new(internal_id.to_string()) {
                Some(id) => id,
                None => unimplemented!(),
            };
            if id == internal_id {
                break item;
            }
        };

        // TODO(rescrv):  Make sure no commands for when or status;
        let now = clock.now();
        let mut events_file = self.file_for_events(OpenOptions::new().append(true))?;
        match write!(events_file, "when:{} status:{} {}\n", now, status, item) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn add_rhythm(&mut self, rhythm: &dyn Rhythm) -> Result<(), Error> {
        let mut file = self.file_for_rhythms(OpenOptions::new().append(true))?;
        write!(file, "{}\n", rhythm.line_item())?;
        Ok(())
    }

    fn file_for_rhythms(&self, options: &mut OpenOptions) -> Result<File, Error> {
        let (rhythms, _) = file_names(&self.root);
        match options.open(rhythms) {
            Ok(f) => Ok(f),
            Err(e) => Err(Error::IO(e)),
        }
    }

    fn file_for_events(&self, options: &mut OpenOptions) -> Result<File, Error> {
        let (_, events) = file_names(&self.root);
        match options.open(events) {
            Ok(f) => Ok(f),
            Err(e) => Err(Error::IO(e)),
        }
    }
}

//////////////////////////////////////////// file_names ////////////////////////////////////////////


// TODO(rescrv):  This function hasn't made sense ever since it wasn't making any sense.
// Deprecated
pub fn file_names(root: &str) -> (String, String) {
    // Public constants in the core
    const FILE_RHYTHMS: &str = "rhythms";
    const FILE_EVENTS: &str = "events";
    // NOTE(rescrv):  We assume that `root` is validated here.  There's not much we could do to
    // validate except to pass in additional strings or objects.  Then we'd have to assume they
    // were validated and then use them to validate root.  Easier to assume that root is valid.
    //
    // NOTE(rescrv):  The above note has three lines of equal length :)
    let rhythms = root.to_string() + "/" + FILE_RHYTHMS;
    let events = root.to_string() + "/" + FILE_EVENTS;
    (rhythms, events)
}
