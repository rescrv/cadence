// TODO(rescrv): clean up these words
pub const COMMAND_ID: &str = "id:";

pub const COMMAND_TYPE: &str = "type:";
pub const COMMAND_WHEN: &str = "when:";
pub const COMMAND_SLIDER: &str = "slider:";
pub const COMMAND_STATUS: &str = "status:";
pub const COMMAND_DUE: &str = "due:";

pub const COMMAND_DOTM: &str = "dotm:";
pub const COMMAND_DOTW: &str = "dotw:";

pub const COMMAND_N: &str = "n:";

pub const COMMAND_TOD: &str = "tod:";

// TODO(rescrv) make sure all commands end in :
pub const COMMAND_WORDS: &[&str] = &[
    COMMAND_ID,
    COMMAND_TYPE,
    COMMAND_WHEN,
    COMMAND_SLIDER,
    COMMAND_STATUS,
    COMMAND_DUE,
    COMMAND_DOTM,
    COMMAND_DOTW,
    COMMAND_N,
    COMMAND_TOD,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colons() {
        for word in COMMAND_WORDS.iter() {
            assert!(word.ends_with(":"));
        }
    }
}
