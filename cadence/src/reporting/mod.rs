use crate::*;

pub mod basic_schedule;
pub mod smooth_schedule;

pub mod health_check;
pub mod schedule_convergence;

////////////////////////////////////////////// Report //////////////////////////////////////////////

type PlumbingIterator = Box<dyn Iterator<Item=Event>>;
type PorcelainIterator = Box<dyn Iterator<Item=String>>;

pub trait Schedule {
    // Considered having plumbing iterator + function to map.  Did this instead so that reports can
    // have plumbing and porcelain iterators with a different number of elements.
    fn plumbing(&self) -> PlumbingIterator;
    fn porcelain(&self) -> PorcelainIterator;
}

/////////////////////////////////////////// FileSchedule //////////////////////////////////////////

pub struct FileSchedule {
    events: Vec<Event>,
}

impl FileSchedule {
    pub fn new(filename: &str) -> Result<FileSchedule> {
        let events = Events::new(filename)?.iter().collect();
        let fs = FileSchedule {
            events,
        };
        Ok(fs)
    }
}

impl Schedule for FileSchedule {
    fn plumbing(&self) -> PlumbingIterator {
        let mut beats = self.events.clone();
        beats.sort_by(|lhs, rhs| lhs.when.cmp(&rhs.when));
        Box::new(CopiedIterator {
            elements: beats,
        })
    }

    fn porcelain(&self) -> PorcelainIterator {
        let mut beats = Vec::new();
        for ev in self.plumbing() {
            beats.push(format!("{}", ev));
        }
        Box::new(CopiedIterator {
            elements: beats,
        })
    }
}
