use std::collections::BTreeMap;
use std::rc::Rc;

use crate::*;
use crate::rhythms::EveryNDays;
use crate::rhythms::Rhythm;
use crate::command_words::COMMAND_WHEN;

use super::PlumbingIterator;
use super::PorcelainIterator;
use super::Schedule as ScheduleTrait;

struct SmoothRhythm {
    // heap_key orders by time of day and priority.  Lowest value takes priority.
    heap_key: (DateTimeOfDay, u64),
    // Original ideal date for this rhythm.  This is needed in case e.g. a Thursday-due task gets
    // moved to Wednesday out of concern for over-scheduling Thursday.  Without this, we would go
    // to the next of Wednesday, which is the original Thursday.  This doesn't work for daily or
    // every-n and does work for monthly and week-daily.
    original_target: DateTimeOfDay,
    // remaining_choices captures the slider-derived choices of the rhythm.
    remaining_choices: Vec<(DateTimeOfDay, u64)>,
    // passed_over_choices are choices that were skipped because there were too many other beats in
    // at those DateTimeOfDays.
    passed_over_choices: Vec<DateTimeOfDay>,
    // rhythm is the rhythm used to instantiate a smooth rhythm.
    rhythm: Rc<Box<dyn Rhythm>>,
}

impl SmoothRhythm {
    pub fn new(rhythm: Box<dyn Rhythm>, target: DateTimeOfDay) -> SmoothRhythm {
        SmoothRhythm::new_rc(Rc::new(rhythm), target)
    }

    fn new_rc(rhythm: Rc<Box<dyn Rhythm>>, target: DateTimeOfDay) -> SmoothRhythm {
        let mut remaining_choices = Vec::new();
        let mut priority = 1;
        remaining_choices.push((target, 0));
        // before
        for idx in 0..rhythm.slider().before {
            // It gets too tricky to try and memoize this.  Compute it every time instead.
            let mut dtod = target.prev_date();
            for _ in 0..idx {
                dtod = dtod.prev_date();
            }
            remaining_choices.push((dtod, priority));
            priority += 1;
        }
        // after
        for idx in 0..rhythm.slider().after {
            // ditto
            let mut dtod = target.succ_date();
            for _ in 0..idx {
                dtod = dtod.succ_date();
            }
            remaining_choices.push((dtod, priority));
            priority += 1;
        }
        // Sort it by priority so lowest priority is at the back.  So you can pop.  Then return.
        remaining_choices = remaining_choices.into_iter().rev().collect();
        // Safe to unwrap because we always push target/0 onto the choices.
        let heap_key = remaining_choices.pop().unwrap();
        let original_target = heap_key.0;
        SmoothRhythm {
            heap_key,
            original_target,
            remaining_choices,
            passed_over_choices: Vec::new(),
            rhythm,
        }
    }

    fn next_best_choice(&self) -> Option<SmoothRhythm> {
        let mut remaining_choices = self.remaining_choices.clone();
        let mut passed_over_choices = self.passed_over_choices.clone();
        passed_over_choices.push(self.heap_key.0);
        let heap_key = match remaining_choices.pop() {
            Some(x) => x,
            None => return None,
        };
        let original_target = self.original_target;
        let rhythm = SmoothRhythm {
            heap_key,
            original_target,
            remaining_choices,
            passed_over_choices,
            rhythm: Rc::clone(&self.rhythm),
        };
        Some(rhythm)
    }

    fn next_beat(&self) -> SmoothRhythm {
        let next_beat = if self.rhythm.skip_beat_within_slider() {
            self.rhythm.next_beat(self.original_target)
        } else {
            self.rhythm.next_beat(self.heap_key.0)
        };
        SmoothRhythm::new_rc(Rc::clone(&self.rhythm), next_beat)
    }

    fn dtod_limit(&self, recurse_limits: &mut BTreeMap<DateTimeOfDay, u64>) -> u64 {
        let mut total = 0;
        let mut count = 0;
        // remaining choices
        for (dtod, _) in self.remaining_choices.iter() {
            total += 1;
            count += *recurse_limits.entry(*dtod).or_insert(1);
        }
        // current choice
        total += 1;
        let srhythm_limit = *recurse_limits.entry(self.heap_key.0).or_insert(1);
        count += srhythm_limit;
        // passed over choices
        for dtod in self.passed_over_choices.iter() {
            total += 1;
            count += *recurse_limits.entry(*dtod).or_insert(1);
        }
        if total <= 0 {
            0
        } else {
            std::cmp::max(count / total, srhythm_limit)
        }
    }
}

pub struct Schedule {
    slots: BTreeMap<DateTimeOfDay, Vec<SmoothRhythm>>,
}

fn push_rhythms_onto_heap<R: 'static + Rhythm>(cadence: &Cadence, start:DateTimeOfDay, limit: DateTimeOfDay, heap: &mut Vec<SmoothRhythm>, it: &mut dyn Iterator<Item=R>) {
    for rhythm in it {
        // TODO(rescrv):  This is buggy.  If the new() initializeds last_seen to start, we will
        // always push it next_beat into the future.
        let last_seen = match cadence.events.latest_event(rhythm.id()) {
            Some(x) => x.when,
            None => start,
        };
        let target = rhythm.starting_beat(start, last_seen);
        if target >= limit {
            continue;
        }
        let srhythm = SmoothRhythm::new(Box::new(rhythm), target);
        heap.push(srhythm);
    }
}

impl Schedule {
    pub fn new(cadence: &Cadence, start: DateTimeOfDay, limit: DateTimeOfDay) -> Result<Self> {
        let recurse_limits = BTreeMap::new();
        Schedule::recursive_new(cadence, start, limit, recurse_limits)
    }

    // Base case is when dtod_limit is greater than some multiple of the number of rhythms.
    fn recursive_new(cadence: &Cadence, start: DateTimeOfDay, limit: DateTimeOfDay, recurse_limits: BTreeMap<DateTimeOfDay, u64>) -> Result<Self> {
        // TODO(rescrv)
        //eprintln!("recurse: start:{} limit:{}, recurse_limits:{:?}", start, limit, recurse_limits);
        let mut recurse_limits = recurse_limits;
        // Stop chaos.
        if start >= limit {
            return Err(Error::StringErrorXXX(
                "limit must be greater than start".to_string(),
            ));
        }
        let mut heap = Vec::new();
        // First we lay down the dailies.
        push_rhythms_onto_heap(cadence, start, limit, &mut heap, &mut cadence.rhythms.dailies());
        // Then we add the monthlies.
        push_rhythms_onto_heap(cadence, start, limit, &mut heap, &mut cadence.rhythms.monthlies());
        // Then we add the week-dailies.
        push_rhythms_onto_heap(cadence, start, limit, &mut heap, &mut cadence.rhythms.week_dailies());
        // Finally the every-n-days.
        // These we will sort in decreasing order by the N value.
        // TODO(rescrv): This is totally useless and blown away in the first sort of the heap.
        let mut every_n: Vec<EveryNDays> = cadence.rhythms.every_n_dailies().collect();
        every_n.sort_by(|lhs, rhs| rhs.n.cmp(&lhs.n));
        push_rhythms_onto_heap(cadence, start, limit, &mut heap, &mut every_n.into_iter());
        // Now do something with the rhythms.
        let mut sched = Schedule {
            slots: BTreeMap::new(),
        };
        while heap.len() > 0 {
            // Sort so that lower heap keys end up last.  Yes, that's technically an expensive
            // heap, but it allows poking and prodding which I once thought I needed, but might not
            // anymore.
            let sort_by = |lhs: &SmoothRhythm, rhs: &SmoothRhythm| {
                // So why do we do all these comparisons up front?  It's prettier and lends to more
                // readable code.
                let sort_by_heap_key = rhs.heap_key.cmp(&lhs.heap_key);
                let sort_by_choices = rhs.remaining_choices.len().cmp(&lhs.remaining_choices.len());
                match sort_by_heap_key {
                    std::cmp::Ordering::Equal => sort_by_choices,
                    _ => sort_by_heap_key,
                }
            };
            heap.sort_by(sort_by);
            // Sorting can't remove anything so loop invariant holds.
            let srhythm = heap.pop().unwrap();
            let target_dtod = srhythm.heap_key.0;
            let v = sched.slots.entry(target_dtod).or_insert(Vec::new());
            let dtod_limit = srhythm.dtod_limit(&mut recurse_limits) as usize;
            // TODO(rescrv)
            //eprintln!("heap_key:{},{} v.len():{} dtod_limit:{} {}",
            //          srhythm.heap_key.0, srhythm.heap_key.1, v.len(), dtod_limit, srhythm.rhythm.line_item());
            if v.len() >= dtod_limit {
                if let Some(mut srhythm_next) = srhythm.next_best_choice() {
                    if srhythm_next.heap_key.0 < start {
                        srhythm_next.heap_key.0 = start;
                    }
                    // NOTE(rescrv) There's a chance the next best choice will fall above limit,
                    // but some less-preferred choice will fall within [start, limit).  If the task
                    // prefers to fall out of bounds, we can let that happen.
                    heap.push(srhythm_next);
                } else {
                    // We need to recurse here, but we need to do something to change the path of
                    // this mostly deterministic algorithm.  I've tried recursing with a larger
                    // shrythm.original_target, but that has problems when things cluster on the
                    // same original_target (e.g. twelve things on Thursday).  As things smooth
                    // out, the clustered values will be smoothed out too, but they will create a
                    // spike on original_target that will not smooth out.
                    //
                    // We could also smooth on heap_key.0, but that requires things to be
                    // sufficiently far away from the ideal original target that it might not be
                    // ideal.  In fact the only way we reach this point is if we bounced from
                    // option to option with each of the others being full.  Smoothing heap_key.0
                    // is the only way I know to guarantee that this shrythm gets a spot in the
                    // recursion.  I don't even know that, because the little bubble may be taken
                    // because we've maded v.len() >= dtod_limit for a previous shrythm on this
                    // target_dtod.
                    //
                    // This last insight is important:  Nothing we can do from this state
                    // guarantees us a slot here.  Some other srhythm can take it and leave free a
                    // less preferential target_dtod that this shrythm will not accept.
                    //
                    // That leaves us with one option:  Water.  Expand the target_dtod as water
                    // would fill a depression.  Do it only for passed-over choices.
                    let mut water_mark = dtod_limit as u64;
                    for choice in srhythm.passed_over_choices.iter() {
                        let limit = *recurse_limits.entry(*choice).or_insert(1);
                        if limit < water_mark {
                            water_mark = limit;
                        }
                    }
                    water_mark += 1;
                    // Fill to the water mark.  We fill all recurse_limits and not just our own
                    // because we're going for the smoothest possible layout and this may not hurt
                    // that.  Under adversarial situations it will always be possible to manipulate
                    // sliders and change this outcome.  Recursion limits bounded by srhythms.
                    for (dtod, water) in recurse_limits.iter_mut() {
                        if *water < water_mark {
                            *water = water_mark;
                        }
                    }
                    return Schedule::recursive_new(cadence, start, limit, recurse_limits);
                }
            } else {
                let next_srhythm = srhythm.next_beat();
                if next_srhythm.heap_key.0 < limit {
                    heap.push(next_srhythm);
                }
                v.push(srhythm);
            }
        }
        Ok(sched)
    }
}

impl ScheduleTrait for Schedule {
    fn plumbing(&self) -> PlumbingIterator {
        let mut rhythms = Vec::new();
        for (when, slot) in self.slots.iter() {
            for s in slot.iter() {
                let mut item = s.rhythm.line_item();
                item.insert(COMMAND_WHEN, &format!("{}", when));
                let ev = Event {
                    id: s.rhythm.id(),
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
            for s in slot.iter() {
                rhythms.push(format!("{} @ {}", s.rhythm.human_line(), when));
            }
        }
        Box::new(CopiedIterator {
            elements: rhythms,
        })
    }
}
