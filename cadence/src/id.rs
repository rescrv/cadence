use std::fmt::Display;

use rand::Rng;

const IDEAL_ID_LENGTH: usize = 6;
const ID_LENGTH_LB: usize = 4; // inclusive
const ID_LENGTH_UB: usize = 8; // inclusive

//////////////////////////////////////////////// ID ////////////////////////////////////////////////

/// A unique identifier for a process.  Identifiers will be unique across process types.
// TODO(rescrv):  Consider making this non-pub.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct ID {
    id: String,
}

impl ID {
    pub fn new(id: String) -> Option<ID> {
        let mut id = id;
        if id.starts_with("id:") {
            // NOTE(rescrv):  replace_range([..3usize], "") did not work here.
            for _ in 0..3 {
                id.remove(0);
            }
        }
        if id.len() < ID_LENGTH_LB || id.len() > ID_LENGTH_UB {
            return None;
        }
        for c in id.chars() {
            if !c.is_ascii_alphanumeric() {
                return None
            }
        }
        let id = ID {
            id,
        };
        Some(id)
    }

    pub fn rand() -> ID {
        let mut rng = rand::thread_rng();
        let id_bytes: [u8; IDEAL_ID_LENGTH] = rng.gen();
        let mut id: String = String::default();
        // NOTE(rescrv):  I couldn't find this constant in std, so I made it.
        //
        // python -c 'import string; print(string.ascii_letters + string.digits)'    
        let characters: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        for idx in 0..IDEAL_ID_LENGTH {
            let char_idx: usize = id_bytes[idx] as usize % characters.len();
            id.push(characters.as_bytes()[char_idx] as char);
        }
        ID {
            id,
        }
    }
}

impl Display for ID {
    fn fmt(&self, fmter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmter, "id:{}", self.id)
    }
}

/////////////////////////////////////////////// tests //////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_id() {
        let id = ID::new("id:m5QVZdcb".to_string());
        assert_eq!(Some(ID { id: "m5QVZdcb".to_string() }), id);
    }

    #[test]
    fn without_id() {
        let id = ID::new("m5QVZdcb".to_string());
        assert_eq!(Some(ID { id: "m5QVZdcb".to_string() }), id);
    }

    #[test]
    fn over_length() {
        let id = ID::new("m5QVZdcbL".to_string());
        assert_eq!(None, id);
        let id = ID::new("id:m5QVZdcbL".to_string());
        assert_eq!(None, id);
    }

    #[test]
    fn under_length() {
        let id = ID::new("Zdc".to_string());
        assert_eq!(None, id);
        let id = ID::new("id:Zdc".to_string());
        assert_eq!(None, id);
    }

}
