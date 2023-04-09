use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use crate::Error;
use crate::LineItem;

//////////////////////////////////////////// RawIterator ///////////////////////////////////////////

/// An iterator that returns LineItems in the order they appear in the provided filename
pub struct RawIterator {
    f: BufReader<File>,
}

impl RawIterator {
    pub fn new(filename: &str) -> Result<Self, Error> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => return Err(Error::IO(e)),
        };
        let f = BufReader::new(file);
        let iter = RawIterator {
            f,
        };
        Ok(iter)
    }

    // Something I don't understand about associted types and inherent impls stops me from using
    // BufReader<File> as the item of the error trait.

    pub fn next(&mut self) -> Option<Result<LineItem, Error>> {
        let mut buf = String::new();
        let line_sz = self.f.read_line(&mut buf);
        let line_sz = match line_sz {
            Ok(sz) => sz,
            Err(e) => return Some(Err(Error::IO(e))),
        };
        if line_sz == 0 {
            return None
        }
        assert_eq!(buf.len(), line_sz);
        let line = buf.trim();
        match LineItem::new(line) {
            Some(lr) => Some(Ok(lr)),
            None => Some(Err(Error::InvalidLineRepresentation(line.to_string()))),
        }
    }
}

////////////////////////////////////////// CopiedIterator //////////////////////////////////////////
