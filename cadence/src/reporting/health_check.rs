use std::fmt::Display;

use crate::*;

#[derive(Default)]
pub struct Score {
    value: u64,
    never: u64,
}

impl Display for Score {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "delay: {}, never done: {}", self.value, self.never)
    }
}

pub fn health_check(cadence: &Cadence, boundary: DateTimeOfDay) -> Score {
    let mut score = Score::default();
    for rhythm in cadence.rhythms.rhythms() {
        let ev = match cadence.events.latest_event_before(rhythm.id(), boundary) {
            Some(x) => x,
            None => {
                score.never += 1;
                continue;
            },
        };
        let next = rhythm.next_beat(ev.when);
        if next < boundary {
            score.value += next.days_apart(boundary);
        }
    }
    score
}
