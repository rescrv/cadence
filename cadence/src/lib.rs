mod time;
pub use time::{Clock, DateTimeOfDay, DEFAULT_TIMEZONE, TimeOfDay};

pub mod command_words;
pub mod cmdline;

pub mod rhythms;

pub mod util;

mod id;
pub use id::ID;

pub mod reporting;

mod writer;
pub use writer::Writer;

pub mod core;
pub use crate::core::{CopiedIterator, Event, Events, Rhythms, Cadence};

pub mod app;

pub const AUTHOR_STRING: &'static str = "Robert Escriva <robert@rescrv.net>";

/////////////////////////////////////////////// Error //////////////////////////////////////////////

// NOTE(rescrv):  When extending this enum, extend the PartialOrd below
#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    // TODO(rescrv): kill this.
    StringErrorXXX(String),
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::StringErrorXXX(s)
    }
}

impl From<line_item::Error> for Error {
    fn from(err: line_item::Error) -> Error {
        match err {
            line_item::Error::IO(e) => { Error::IO(e) },
            line_item::Error::InvalidLineRepresentation(e) => { Error::StringErrorXXX(format!("invalid line repr: {}", e)) },
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
    }
}

////////////////////////////////////////////// Result //////////////////////////////////////////////

type Result<T> = std::result::Result<T, Error>;
