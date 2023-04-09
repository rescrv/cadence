use crate::*;
use super::Schedule;

fn first_in_schedule(sched: &dyn Schedule, id: ID) -> Option<Event> {
    let mut event: Option<Event> = None;
    for ev in sched.plumbing() {
        if ev.id == id  && (event.is_none() || event.clone().unwrap().when > ev.when) {
            event = Some(ev)
        }
    }
    event
}

pub fn convergence(cadence: &Cadence, sched: &dyn Schedule, boundary: DateTimeOfDay) -> DateTimeOfDay {
    let mut horizon = DateTimeOfDay::default();
    let mut _infinite = false;
    let mut _new_event_unscheduled = false;
    for rhythm in cadence.rhythms.rhythms() {
        let last_seen = cadence.events.latest_event_before(rhythm.id(), boundary);
        let next_scheduled = first_in_schedule(sched, rhythm.id());
        // TODO(rescrv):  This is horribly broken.
        match (last_seen, next_scheduled) {
            // Steady state.  Hopefully.
            (Some(ls), Some(ns)) => {
                // When the next beat after last_seen is in the past, we know that we're behind.
                // This task should push the horizon to the point where it's back in compliance.
                // We don't consider rhythms whose next beat is after the boundary because the
                // schedule can still bring them into compliance in the future before they are
                // delayed; hopefully they will be brought into compliance by a future schedule.
                // Lots of hope in this section.
                if rhythm.next_beat(ls.when) < boundary && ns.when > boundary {
                    horizon = ns.when;
                }
            },
            // A new event is scheduled!
            (None, Some(x)) => {
                if x.when >= horizon {
                    horizon = x.when;
                }
            },
            // We've seen the event in the past, but it doesn't appear in the schedule.
            (Some(_), None) => {
                _infinite = true;
            },
            // There's a new event not on the schedule.
            (None, None) => {
                _new_event_unscheduled = true;
            },
        }
    }
    // Three things to take action on.  First we have the horizon from scheduled tasks, and then we
    // have two corner cases for new and unscheduled events.  The two corner cases are really one
    // and almost certainly due to the schedule having a short horizon to it.
    //
    // TODO(rescrv):  Fix these corner cases by changing the schedule to take a window and then
    // progressively open the window until all events have a next-scheduled value.  This inherently
    // pushes things to the steady state and new event cases.  Keep them for debugging for now.
    horizon
}
