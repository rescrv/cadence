use std::collections::BTreeMap;
use std::fmt::Display;

pub mod iter;

pub const SPECIAL_CHARS: &[char] = &[
    ':',
    '\r',
    '\n',
];

/////////////////////////////////////////////// Error //////////////////////////////////////////////

pub enum Error {
    IO(std::io::Error),
    InvalidLineRepresentation(String),
}

//////////////////////////////////////////// Description ///////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Description {
    desc: String,
}

impl Description {
    pub fn new(desc: String) -> Option<Description> {
        for c in desc.chars() {
            if SPECIAL_CHARS.contains(&c) {
                return None
            }
        }

        let desc = Description {
            desc,
        };
        Some(desc)
    }
}

/////////////////////////////////////////// CommandWords ///////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandWords {
    command_words: BTreeMap<String, String>,
}

impl CommandWords {
    fn has(&self, key: &str) -> bool {
        assert!(key.ends_with(":"));
        self.command_words.contains_key(key)
    }

    fn lookup(&self, key: &str) -> Option<&str> {
        assert!(key.ends_with(":"));
        match self.command_words.get(key) {
            Some(x) => Some(&x),
            None => None,
        }
    }

    fn insert(&mut self, key: &str, value: &str) {
        assert!(key.ends_with(":"));
        self.command_words.insert(key.to_string(), value.to_string());
    }

    fn remove(&mut self, key: &str) {
        assert!(key.ends_with(":"));
        self.command_words.remove(key);
    }
}

impl From<BTreeMap<String, String>> for CommandWords {
    fn from(command_words: BTreeMap<String, String>) -> CommandWords {
        CommandWords {
            command_words,
        }
    }
}

///////////////////////////////////////////// LineItem /////////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct LineItem {
    desc: Description,
    cmdw: CommandWords,
}

impl LineItem {
    pub fn new(line: &str) -> Option<LineItem> {
        let line = line.trim();
        let split = line.split(" ");
        let mut normal = Vec::new();
        let mut command = BTreeMap::new();

        for elem in split {
            let mut is_cmd = false;
            if let Some(idx) = elem.find(":") {
                let mut elem = elem.to_string();
                let value = elem.split_off(idx + 1);
                command.insert(elem, value);
                is_cmd = true;
            }
            if !is_cmd && !elem.is_empty() {
                normal.push(elem);
            }
        }

        let desc = normal.join(" ");
        let desc = Description::new(desc)?;
        let cmdw: CommandWords = command.into();

        let li = LineItem {
            desc,
            cmdw,
        };

        Some(li)
    }

    // Public methods

    pub fn repr(&self) -> String {
        let mut line_item = String::new();
        line_item += self.desc();
        for (key, value) in self.cmdw.command_words.iter() {
            line_item += " ";
            line_item += key;
            line_item += value;
        }
        line_item
    }

    // Proxy Description

    pub fn desc(&self) -> &str {
        &self.desc.desc
    }

    // Proxy CommandWords

    pub fn has(&self, key: &str) -> bool {
        self.cmdw.has(key)
    }

    pub fn lookup(&self, key: &str) -> Option<&str> {
        self.cmdw.lookup(key)
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.cmdw.insert(key, value)
    }

    pub fn remove(&mut self, key: &str) {
        self.cmdw.remove(key)
    }

    // Private methods
}

impl Display for LineItem {
    fn fmt(&self, fmter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmter, "{}", self.repr())
    }
}

/////////////////////////////////////////////// tests //////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    mod description {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn success() {
                let desc = Description::new("this is a valid description".to_string());
                assert_eq!(Some(Description { desc: "this is a valid description".to_string() }), desc);
            }

            #[test]
            fn failure() {
                let desc = Description::new("this is an in:valid description".to_string());
                assert_eq!(None, desc);
            }
        }
    }

    mod command_words {
        use super::*;

        #[test]
        fn from() {
            let mut map = BTreeMap::new();
            map.insert("id:".to_string(), "rescrv".to_string());
            let map2 = map.clone();
            let got: CommandWords = map.into();
            let exp = CommandWords {
                command_words: map2,
            };
            assert_eq!(exp, got);
        }

        #[test]
        fn lookup() {
            let mut map = BTreeMap::new();
            map.insert("id:".to_string(), "rescrv".to_string());
            let cmd_words: CommandWords = map.into();
            let exp = cmd_words.lookup("id:").unwrap();
            let got = "rescrv";
            assert_eq!(exp, got);
        }

        #[test]
        fn remove() {
            let mut map = BTreeMap::new();
            map.insert("id:".to_string(), "rescrv".to_string());
            let mut cmd_words: CommandWords = map.into();
            cmd_words.remove("id:");
            let exp = cmd_words.lookup("id:");
            let got = None;
            assert_eq!(exp, got);
        }
    }

    mod line_item {
        use super::*;

        #[test]
        fn empty_input() {
            let got = LineItem::new("").unwrap();
            let exp = LineItem {
                desc: Description::new("".to_string()).unwrap(),
                cmdw: CommandWords {
                    command_words:  BTreeMap::new(),
                }
            };
            assert_eq!(exp, got);
        }

        fn cmd_word_exp() -> LineItem {
            let mut map = BTreeMap::new();
            map.insert("id:".to_string(), "rescrv".to_string());
            LineItem {
                desc: Description::new("this is a test".to_string()).unwrap(),
                cmdw: CommandWords {
                    command_words:  map,
                }
            }
        }

        #[test]
        fn cmd_word_first() {
            let exp = cmd_word_exp();
            let got = LineItem::new("id:rescrv this is a test").unwrap();
            assert_eq!(exp, got);
        }

        #[test]
        fn cmd_word_last() {
            let exp = cmd_word_exp();
            let got = LineItem::new("this is a test id:rescrv").unwrap();
            assert_eq!(exp, got);
        }

        #[test]
        fn cmd_word_middle() {
            let exp = cmd_word_exp();
            let got = LineItem::new("this is id:rescrv a test").unwrap();
            assert_eq!(exp, got);
        }

        #[test]
        fn repr_no_commands() {
            let li = LineItem::new("this is a test").unwrap();
            let got = li.repr();
            let exp = "this is a test";
            assert_eq!(exp, got);
        }

        #[test]
        fn repr_with_commands() {
            let li = LineItem::new("this is id:rescrv a test").unwrap();
            let got = li.repr();
            let exp = "this is a test id:rescrv";
            assert_eq!(exp, got);
        }
    }
}
