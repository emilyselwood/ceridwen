use std::path;
use std::path::Path;
use std::path::PathBuf;

pub mod config;
pub mod error;
pub mod index;
pub mod page;

pub fn system_root() -> PathBuf {
    let home_option = home::home_dir();
    if home_option.is_none() {
        panic!("No home dir set for this user. We don't know where to find our config")
    }

    home_option.unwrap().join(".ceridwen")
}

/// Take a path and turn it into
pub fn normalise_path<P>(input: P) -> String
where
    P: AsRef<Path>,
{
    input
        .as_ref()
        .to_str()
        .unwrap()
        .to_string()
        .replace("/", path::MAIN_SEPARATOR_STR)
        .replace("\\", path::MAIN_SEPARATOR_STR)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::normalise_path;

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
