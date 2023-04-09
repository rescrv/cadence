use std::collections::BTreeMap;
use std::rc::Rc;

use crate::*;
use crate::rhythms::Rhythm;
use crate::command_words::COMMAND_WHEN;

use super::PlumbingIterator;
use super::PorcelainIterator;
use super::Schedule as ScheduleTrait;

struct Slot {
    elements: Vec<Rc<Box<dyn Rhythm>>>,
}

pub struct Schedule {
    start: DateTimeOfDay,
    limit: DateTimeOfDay,
    slots: BTreeMap<DateTimeOfDay, Slot>,
}

impl Schedule {
    pub fn new(cadence: &Cadence, start: DateTimeOfDay, limit: DateTimeOfDay) -> Result<Self> {
        if start >= limit {
            return Err(Error::StringErrorXXX(
                "limit must be greater than start".to_string(),
            ));
        }
        let mut bs = Schedule {
            start,
            limit,
            slots: BTreeMap::default(),
        };
        for rhythm in cadence.rhythms.rhythms() {
            let last_seen = match cadence.events.latest_event(rhythm.id()) {
                Some(x) => x.when,
                None => start,
            };
            bs.add_rhythm(last_seen, rhythm)?;
        }
        Ok(bs)
    }

    fn add_rhythm(&mut self, last_seen: DateTimeOfDay, rhythm: Box<dyn Rhythm>) -> Result<()> {
        // NOTE(rescrv):  I'm doing something somewhat stupid here and making a rc holding a box
        // holding the data.  I couldn't figure out a way to leak the data value directly into rc.
        let rhythm: Rc<Box<dyn Rhythm>> = Rc::new(rhythm);
        // TODO(rescrv):  This is buggy.  If the new() initializeds last_seen to start, we will
        // always push it next_beat into the future.
        let mut when = rhythm.next_beat(last_seen);
        while when < self.start {
            when = rhythm.next_beat(when);
        }
        while when < self.limit {
            let slot = match self.slots.get_mut(&when) {
                Some(x) => x,
                None => {
                    let basic = Slot {
                        elements: Vec::new(),
                    };
                    self.slots.insert(when, basic);
                    self.slots.get_mut(&when).unwrap()
                },
            };
            when = rhythm.next_beat(when);
            slot.elements.push(Rc::clone(&rhythm))
        }
        Ok(())
    }
}

impl ScheduleTrait for Schedule {
    fn plumbing(&self) -> PlumbingIterator {
        let mut rhythms = Vec::new();
        for (when, slot) in self.slots.iter() {
            for rhythm in slot.elements.iter() {
                let mut item = rhythm.line_item();
                item.insert(COMMAND_WHEN, &format!("{}", when));
                let ev = Event {
                    id: rhythm.id(),
                    when: *when,
                    item,
                };
                rhythms.push(ev);
            }
        }
        Box::new(CopiedIterator {
            elements: rhythms,
        })
    }

    fn porcelain(&self) -> PorcelainIterator {
        let mut rhythms = Vec::new();
        for (when, slot) in self.slots.iter() {
            for rhythm in slot.elements.iter() {
                rhythms.push(format!("{} @ {}", rhythm.human_line(), when));
            }
        }
        Box::new(CopiedIterator {
            elements: rhythms,
        })
    }
}
