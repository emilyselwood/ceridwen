pub mod percentage;
pub mod text_tools;

use std::env;
use std::fs;
use std::path;
use std::path::Path;
use std::path::PathBuf;

/// Return a PathBuf pointing at the system root dir ($HOME/.ceridwen)
/// This will panic if the home dir of the current user can not be detected.
pub fn system_root() -> PathBuf {
    let home_option = home::home_dir();
    if home_option.is_none() {
        panic!("No home dir set for this user. We don't know where to find our config")
    }

    home_option.unwrap().join(".ceridwen")
}

/// Return a PathBuf pointing at the system root dir ($HOME/.ceridwen) and make sure that it exists.
/// This will panic if it can not create the directory.
/// This will panic if the home dir of the current user can not be detected.
pub fn enforced_system_root() -> PathBuf {
    let result = system_root();
    if !result.exists() {
        println!("System root directory {:?} not found. Creating it", &result);
        fs::create_dir_all(&result).unwrap_or_else(|e| {
            panic!(
                "Could not create system root directory {:?}: {:?}",
                result, e
            )
        });
    }

    if !result.is_dir() {
        panic!("{:?} already exists but is not a directory", result);
    }

    result
}

/// Get a temp dir for stuff that can be safely deleted once not in use.
/// (we should have open handles to anything currently being used in here)
pub fn temp_dir() -> PathBuf {
    env::temp_dir().join("ceridwen")
}

/// Take a path and turn it into which ever way around the running operating system expects.
pub fn normalise_path<P>(input: P) -> String
where
    P: AsRef<Path>,
{
    input
        .as_ref()
        .to_str()
        .unwrap()
        .to_string()
        .replace(['/', '\\'], path::MAIN_SEPARATOR_STR)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::utils::normalise_path;

    #[test]
    fn test_normalise_path() {
        let input = PathBuf::from("something\\somewhere/with/weird\\things".to_string());
        let result = normalise_path(input);

        let expected =
            PathBuf::from_iter(["something", "somewhere", "with", "weird", "things"].iter())
                .to_str()
                .unwrap()
                .to_string();

        assert_eq!(result, expected);
    }
}
