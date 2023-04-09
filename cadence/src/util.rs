use std::env;
use std::path::PathBuf;
use std::process::Command;

use dirs::data_dir;

use crate::{Error, Result};
use crate::rhythms::Slider;

/////////////////////////////////////// path_relative_to_root //////////////////////////////////////

pub fn path_relative_to_root(root: &str, path: &str) -> String {
    // TODO(rescrv):  Test this function will do the right thing when path has a /
    // TODO(rescrv):  Make this function work with PathBuf instead of string.
    let mut full_path = String::with_capacity(256);
    full_path.push_str(root);
    full_path.push_str("/");
    full_path.push_str(path);
    full_path
}

///////////////////////////////////////////// parse_u32 ////////////////////////////////////////////

pub fn parse_u32(value: &str) -> Result<u32> {
    match value.parse() {
        Ok(x) => Ok(x),
        Err(_) => Err(Error::StringErrorXXX(format!("expected a u32-convertible string, got {}", value))),
    }
}

/////////////////////////////////////////// parse_slider ///////////////////////////////////////////

pub fn parse_slider(value: &str) -> Result<Slider> {
    let mut value = value.to_string();
    if value.len() != value.bytes().len() {
        return Err(Error::StringErrorXXX(format!("expected a slider, but got some multi-byte unicode {}", value)));
    }
    let idx = match value.find(',') {
        Some(x) => x,
        None => {
            return Err(Error::StringErrorXXX(format!("expected a slider, but cannot find the comma {}", value)));
        }
    };
    let mut after = value.split_off(idx);
    after.remove(0);
    let before = value;
    let before = parse_u32(&before)?;
    let after = parse_u32(&after)?;
    let slider = Slider {
        before,
        after,
    };
    Ok(slider)
}

//////////////////////////////////// expand_basename_using_path ////////////////////////////////////

pub fn expand_basename_using_path(basename: &str) -> String {
    const PATH_VAR: &'static str = "PATH";

    let paths = match env::var_os(PATH_VAR) {
        Some(paths) => paths,
        None => return basename.to_string(),
    };

    for path in env::split_paths(&paths) {
        let bname: PathBuf = basename.into();
        let mut full_path = PathBuf::with_capacity(256);
        full_path.push(path);
        full_path.push(bname);
        if std::fs::metadata(&full_path).is_ok() {
            match full_path.to_str() {
                Some(s) => return s.to_string(),
                // TODO(rescrv):  This path should only take when converting a bytes
                // item to unicode and it fails.  Cadence is unicode, and we can
                // assume that the pieces of the PATH var are proper unicode.  And
                // we can assume the basename is unicode.  Unicode throughout, so
                // this case is really unlikely to be seen.  Safe to fall through
                // to basename.
                None => {}
            }
        }
    }
    basename.to_string()
}

//////////////////////////////////////////// run_command ///////////////////////////////////////////

// NOTE(rescrv):  Human oriented can panic.
pub fn run_command(args: &mut [String]) {
    let exec = &args[0];
    let temp = expand_basename_using_path(exec);
    let exec = if exec.contains('/') {
        exec
    } else {
        &temp
    };
    args[0] = match std::fs::canonicalize(&exec) {
        Ok(p) => {
            match p.to_str() {
                Some(s) => s.to_string(),
                None => panic!("cannot canonicalize {}", exec),
            }
        }
        Err(e) => panic!("cannot fetch metadata for program:  {}", e),
    };
    let cmd = args[0].clone() + "-" + &args[1];
    let mut cmd = Command::new(cmd);
    let _ = match cmd.args(&args[2..]).status() {
        Ok(status) => status,
        Err(e) => panic!("spawning internal process failed:  {}", e),
    };
}

/////////////////////////////////////////// get_root_dir ///////////////////////////////////////////

pub fn get_root_dir() -> Option<String> {
    match env::var("CADENCE_DATADIR") {
        Ok(root) => return Some(root),
        Err(_) => {},
    };
    // TODO(rescrv): test this
    match env::var("CADENCE_TEST_SAFELY") {
        Ok(_) => return None,
        Err(_) => {},
    };
    match data_dir() {
        Some(data_dir) => {
            match data_dir.into_os_string().into_string() {
                Ok(data_dir) => {
                    let dd = data_dir + "/cadence";
                    Some(dd)
                },
                Err(_) => None,
            }
        }
        None => None,
    }
}

/////////////////////////////////////////////// tests //////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::parse_u32 as parseu32;

    mod parse_u32 {
        use super::*;

        #[test]
        fn zero() {
            assert_eq!(0, parseu32("0").unwrap());
        }

        #[test]
        fn one() {
            assert_eq!(1, parseu32("1").unwrap());
        }

        #[test]
        fn many() {
            assert_eq!(4294967295, parseu32("4294967295").unwrap());
        }

        #[test]
        fn too_many() {
            match parseu32("4294967296") {
                Ok(_) => panic!("OK when it shouldn't be"),
                Err(_) => {},
            }
        }
    }
}
