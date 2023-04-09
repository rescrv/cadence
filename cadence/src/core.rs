use std::collections::BTreeSet;
use std::fmt::Display;

use chrono::Weekday;

use line_item::LineItem;
use line_item::iter::RawIterator;

use crate::ID;
use crate::Error;
use crate::DateTimeOfDay;
use crate::rhythms::*;
use crate::command_words::*;
use crate::time::Clock;
use crate::util::parse_slider;
use crate::util::parse_u32;
use crate::util::path_relative_to_root;

pub const FILE_RHYTHMS: &str = "rhythms";
pub const FILE_EVENTS: &str = "events";

////////////////////////////////////////////// Rhythms /////////////////////////////////////////////
/// NOTE(rescrv):  This loads all data in the `new' call into memory so all subsequent only return
/// an error when there's an anticipated control flow error that's not I/O.  It's OK to load
/// everything into memory because data size is kept small.

#[derive(Debug)]
pub struct Rhythms {
    dailies: Vec<Daily>,
    monthlies: Vec<Monthly>,
    week_dailies: Vec<WeekDaily>,
    every_n_dailies: Vec<EveryNDays>,
    errors: Vec<(LineItem, Error)>,
}

impl Rhythms {
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut rhythms = Rhythms {
            dailies: Vec::new(),
            monthlies: Vec::new(),
            week_dailies: Vec::new(),
            every_n_dailies: Vec::new(),
            errors: Vec::new(),
        };
        // TODO(rescrv):  Only do this when the error is that the file doesn't exist.
        if !std::fs::metadata(&path).is_ok() {
            return Ok(rhythms);
        }
        let mut iter = RawIterator::new(&path)?;
        loop {
            let item = match iter.next() {
                Some(Ok(item)) => item,
                Some(Err(e)) => return Err(e.into()),
                None => break,
            };
            // TODO add erroring line items to their own item
            match rhythms.add_line_item(&item) {
                Ok(_) => {},
                Err(e) => {
                    rhythms.errors.push((item, e));
                },
            }
        }
        Ok(rhythms)
    }

    fn add_line_item(&mut self, item: &LineItem) -> Result<(), Error> {
        let ty = lookup(item, COMMAND_TYPE)?;
        let id = lookup(item, COMMAND_ID)?.to_string();
        let id = match ID::new(id) {
            Some(id) => id,
            None => return Err(Error::StringErrorXXX("ID not parseable".to_string())),
        };
        let desc = item.desc().to_string();
        if ty == "daily" {
            let daily = Daily { id, desc };
            self.dailies.push(daily);
        } else if ty == "monthly" {
            let dotm = lookup(item, COMMAND_DOTM)?;
            let dotm: u32 = parse_u32(dotm)?;
            let slider = match item.lookup(COMMAND_SLIDER) {
                Some(x) => parse_slider(x)?,
                None => Slider::default(),
            };
            let monthly = Monthly {
                id,
                desc,
                dotm,
                slider,
            };
            self.monthlies.push(monthly);
        } else if ty == "week-daily" {
            let dotw = lookup(item, COMMAND_DOTW)?;
            // TODO(rescrv):  generate error to return rather than expect.  Not doing it now
            // because I don't want StringErrorXXX to spread further.
            let just_in_case = format!("expected a weekday-convertible string, got {}", dotw);
            let dotw: Weekday = dotw.parse().expect(&just_in_case);
            let slider = match item.lookup(COMMAND_SLIDER) {
                Some(x) => parse_slider(x)?,
                None => Slider::default(),
            };
            let week_daily = WeekDaily {
                id,
                desc,
                dotw,
                slider,
            };
            self.week_dailies.push(week_daily);
        } else if ty == "every-n-days" {
            let n = lookup(item, COMMAND_N)?;
            let n: u32 = parse_u32(n)?;
            let slider = match item.lookup(COMMAND_SLIDER) {
                Some(x) => {
                    parse_slider(x)?
                },
                None => {
                    Slider::default()
                },
            };
            let every_n = EveryNDays {
                id,
                desc,
                n,
                slider,
            };
            self.every_n_dailies.push(every_n);
        } else {
            unimplemented!();
        }
        Ok(())
    }

    pub fn rhythms(&self) -> impl Iterator<Item=Box<dyn Rhythm>> {
        let mut rhythms: Vec<Box<dyn Rhythm>> = Vec::new();
        for daily in self.dailies.iter() {
            rhythms.push(Box::new(daily.clone()));
        }
        for monthly in self.monthlies.iter() {
            rhythms.push(Box::new(monthly.clone()));
        }
        for week_daily in self.week_dailies.iter() {
            rhythms.push(Box::new(week_daily.clone()));
        }
        for every_n_days in self.every_n_dailies.iter() {
            rhythms.push(Box::new(every_n_days.clone()));
        }
        CopiedIterator {
            elements: rhythms,
        }
    }

    pub fn dailies(&self) -> impl Iterator<Item=Daily> {
        CopiedIterator {
            elements: self.dailies.clone(),
        }
    }

    pub fn monthlies(&self) -> impl Iterator<Item=Monthly> {
        CopiedIterator {
            elements: self.monthlies.clone(),
        }
    }

    pub fn week_dailies(&self) -> impl Iterator<Item=WeekDaily> {
        CopiedIterator {
            elements: self.week_dailies.clone(),
        }
    }

    pub fn every_n_dailies(&self) -> impl Iterator<Item=EveryNDays> {
        CopiedIterator {
            elements: self.every_n_dailies.clone(),
        }
    }

    #[cfg(test)]
    pub fn is_empty(self) -> bool {
        self.rhythms().count() == 0
    }
}

/////////////////////////////////////////////// Event //////////////////////////////////////////////

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Event {
    pub id: ID,
    pub when: DateTimeOfDay,
    pub item: LineItem,
}

impl Display for Event {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let mut line_item = self.item.clone();

        line_item.remove(&format!("{}", COMMAND_ID));
        line_item.remove(&format!("{}", COMMAND_WHEN));
        write!(fmt, "{} when:{} {}", self.id, self.when, line_item)
    }
}

////////////////////////////////////////////// Events //////////////////////////////////////////////
/// NOTE(rescrv):  As with other core types, the data is loaded into memory so other calls only
/// fail for control-flow reasons.

pub struct Events {
    events: BTreeSet<Event>,
    errors: Vec<(LineItem, Error)>,
}

impl Events {
    pub fn new(path: &str) -> Result<Events, Error> {
        let mut events = Events {
            events: BTreeSet::new(),
            errors: Vec::new(),
        };
        // TODO(rescrv):  Only do this when the error is that the file doesn't exist.
        if !std::fs::metadata(&path).is_ok() {
            return Ok(events);
        }
        let mut iter = RawIterator::new(&path)?;
        loop {
            let item = match iter.next() {
                Some(Ok(item)) => item,
                Some(Err(e)) => return Err(e.into()),
                None => break,
            };
            match events.add_line_item(&item) {
                Ok(_) => {},
                Err(e) => {
                    events.errors.push((item, e));
                },
            }
        }
        Ok(events)
    }

    fn add_line_item(&mut self, item: &LineItem) -> Result<(), Error> {
        let id = lookup(item, COMMAND_ID)?;
        let id = match ID::new(id.to_string()) {
            Some(id) => id,
            None => return Err(Error::StringErrorXXX("ID not parseable".to_string())),
        };
        let when = lookup(item, COMMAND_WHEN)?;
        let when = DateTimeOfDay::parse(when)?;
        let item: LineItem = (*item).clone();
        let event = Event {
            id,
            when,
            item,
        };
        self.events.insert(event);
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item=Event> {
        let mut iter = CopiedIterator {
            elements: Vec::new(),
        };
        for event in self.events.iter() {
            iter.elements.push(event.clone());
        }
        iter
    }

    pub fn earliest_event_overall(&self) -> Option<Event> {
        match self.events.iter().min_by_key(|ev| ev.when) {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }

    pub fn latest_event_overall(&self) -> Option<Event> {
        match self.events.iter().max_by_key(|ev| ev.when) {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }

    pub fn latest_event(&self, id: ID) -> Option<Event> {
        let mut event: Option<Event> = None;
        for ev in self.events.iter() {
            // It is the proper ID and what we've currently held is earlier than what we're
            // proposing in this loop iteration.
            if ev.id == id && event.clone().unwrap_or(ev.clone()).when <= ev.when {
                event = Some(ev.clone());
            }
        }
        event
    }

    pub fn latest_event_before(&self, id: ID, boundary: DateTimeOfDay) -> Option<Event> {
        let mut event: Option<Event> = None;
        for ev in self.events.iter() {
            if ev.id == id && event.clone().unwrap_or(ev.clone()).when <= ev.when && ev.when < boundary {
                event = Some(ev.clone());
            }
        }
        event
    }
}

////////////////////////////////////////////// Cadence /////////////////////////////////////////////

pub struct Cadence {
    // TODO(rescrv): reevaluate having these pub
    pub rhythms: Rhythms,
    pub events: Events,
    pub clock: Clock,
}

impl Cadence {
    pub fn new(clock: Clock, root: &str) -> Result<Cadence, Error> {
        ensure_root_initialized(root)?;
        let path = path_relative_to_root(root, FILE_RHYTHMS);
        let rhythms = Rhythms::new(&path)?;
        let path = path_relative_to_root(root, FILE_EVENTS);
        let events = Events::new(&path)?;
        let cadence = Cadence {
            rhythms,
            events,
            clock,
        };
        Ok(cadence)
    }
}

////////////////////////////////////////// CopiedIterator //////////////////////////////////////////

pub struct CopiedIterator<C> {
    pub elements: Vec<C>,
}

impl<C> Iterator for CopiedIterator<C> {
    type Item = C;

    fn next(&mut self) -> Option<C> {
        if self.elements.len() > 0 {
            // NOTE(rescrv):  Yes it's inefficient, but it steps around the alternative of having
            // an index where vec[idx] is a move, necessitating a clone, creating a problem for the
            // generic rhythm trait.
            Some(self.elements.remove(0))
        } else {
            None
        }
    }
}

/////////////////////////////////////////////// util ///////////////////////////////////////////////

// Lookup method for commands required to be present in the LineItem.
fn lookup<'a>(item: &'a LineItem, cmd: &'a str) -> Result<&'a str, Error> {
    match item.lookup(cmd) {
        Some(x) => Ok(x),
        None => Err(Error::StringErrorXXX(format!("required field({}) not present", cmd))),
    }
}

fn ensure_root_initialized(root: &str) -> Result<(), Error> {
    std::fs::create_dir_all(root)?;
    Ok(())
}

/////////////////////////////////////////////// tests //////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::TimeOfDay;

    use super::*;

    mod rhythms {
        use std::path::PathBuf;
        use super::*;

        #[test]
        fn empty_rhtyhms_empty_iterators() {
            let rhythms = Rhythms::new("/dev/null").expect("could not open /dev/null");
            for _ in rhythms.rhythms() {
                assert!(false);
            }
            for _ in rhythms.dailies() {
                assert!(false);
            }
            for _ in rhythms.monthlies() {
                assert!(false);
            }
            for _ in rhythms.week_dailies() {
                assert!(false);
            }
            for _ in rhythms.every_n_dailies() {
                assert!(false);
            }
        }

        mod file {
            use super::*;

            #[test]
            fn does_not_exist() {
                let mut file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                file.push("resources");
                file.push("does-not-exist");
                let got = Rhythms::new(file.to_str().unwrap()).unwrap();
                assert!(got.is_empty());
            }

            #[test]
            fn already_exists() {
                let mut file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                file.push("resources");
                file.push("empty");
                let got = Rhythms::new(file.to_str().unwrap()).unwrap();
                assert!(got.is_empty());
            }
        }

        mod slider {
            use super::*;

            #[test]
            fn defaults_to_0_0() {
                let exp = Slider {
                    before: 0,
                    after: 0,
                };
                let got = Slider::default();
                assert_eq!(exp, got);
            }

            #[test]
            fn can_read_0_0() {
                let mut rhythms = Rhythms::new("/dev/null").expect("should be able to read empty file");
                rhythms.add_line_item(&LineItem::new("slider=0,0 type:monthly id:9fuEEECe dotm:18 do it").unwrap()).unwrap();
                let exp = Slider {
                    before: 0,
                    after: 0,
                };
                assert_eq!(rhythms.monthlies.len(), 1);
                let got = rhythms.monthlies[0].slider;
                assert_eq!(exp, got);
            }

            #[test]
            fn can_be_2_1() {
                let mut rhythms = Rhythms::new("/dev/null").expect("should be able to read empty file");
                rhythms.add_line_item(&LineItem::new("slider:2,1 type:monthly id:9fuEEECe dotm:18 do it").unwrap());
                let exp = Slider {
                    before: 2,
                    after: 1,
                };
                assert_eq!(rhythms.monthlies.len(), 1);
                let got = rhythms.monthlies[0].slider;
                assert_eq!(exp, got);
            }
        }

        #[test]
        fn bad_id() {
            let mut rhythms = Rhythms::new("/dev/null").expect("should be able to read empty file");
            let exp = Error::StringErrorXXX("ID not parseable".to_string());
            let got: Error = rhythms.add_line_item(&LineItem::new("type:daily id:X bad_id").unwrap()).unwrap_err().into();
            // XXX need Error to implement Eq assert_eq!(exp, got);
        }

        #[test]
        fn missing_id() {
            let mut rhythms = Rhythms::new("/dev/null").expect("should be able to read empty file");
            let got = rhythms.add_line_item(&LineItem::new("type:daily missing_id").unwrap());
            panic!(got);
        }
    }

    mod events {
        use super::*;

        #[test]
        fn event_display() {
            let item = LineItem::new("id:foo when:bar x:y description here").expect("expected non-none return; means the line item is bad");
            let item_str = format!("{}", item);
            assert_eq!("description here id:foo when:bar x:y", item_str);
            let id = ID::new("123456".to_string()).expect("couldn't make ID");
            let when = DateTimeOfDay::from_ymd(2021, 8, 24, TimeOfDay::Morning);
            let event = Event {
                id,
                when,
                item,
            };
            let event_str = format!("{}", event);
            assert_eq!("id:123456 when:2021-08-24:M description here x:y", event_str);
        }

        #[test]
        fn event_display_no_id_display() {
            let item = LineItem::new("x:y description here").expect("expected non-none return; means the line item is bad");
            let item_str = format!("{}", item);
            assert_eq!("description here x:y", item_str);
            let id = ID::new("123456".to_string()).expect("couldn't make ID");
            let when = DateTimeOfDay::from_ymd(2021, 8, 24, TimeOfDay::Morning);
            let event = Event {
                id,
                when,
                item,
            };
            let event_str = format!("{}", event);
            assert_eq!("id:123456 when:2021-08-24:M description here x:y", event_str);
        }

        #[test]
        fn file_does_not_exist() {
            unimplemented!();
        }

        #[test]
        fn file_exists() {
            unimplemented!();
        }

        #[test]
        fn bad_id() {
            unimplemented!();
        }

        #[test]
        fn missing_id() {
            unimplemented!();
        }

        #[test]
        fn bad_when() {
            unimplemented!();
        }

        #[test]
        fn missing_when() {
            unimplemented!();
        }

        #[test]
        fn earliest_event_overall() {
            unimplemented!();
        }

        #[test]
        fn earliest_event_overall_empty() {
            unimplemented!();
        }

        #[test]
        fn latest_event_overall() {
            unimplemented!();
        }

        #[test]
        fn latest_event_overall_empty() {
            unimplemented!();
        }
    }
}
