//! #233 O1: the reviewed `private-authored-test` command grammar for the offscreen executable.
//! Only the `offscreen` feature compiles this module. The window binary `private-authored-test`
//! keeps its own unchanged parser; this one requires `--headless` (no window path exists here).
#[cfg(windows)]
use crate::{authored_content::Content, disk_cache::DiskCache, Error, Result};

/// The same usage text as `src/bin/private-authored-test.rs`.
pub const USAGE: &str = "CONTENT CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY COMMAND_DIRECTORY [--headless]";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Args {
    pub content: String,
    pub cache: String,
    pub address: String,
    pub user: String,
    pub password_file: String,
    pub public_key: String,
    pub directory: std::path::PathBuf,
}

/// `args` are the program arguments after the executable name. Exactly 8, the last `--headless`.
pub fn parse(args: &[String]) -> std::result::Result<Args, &'static str> {
    if args.len() != 8 || args[7] != "--headless" {
        return Err(USAGE);
    }
    Ok(Args {
        content: args[0].clone(),
        cache: args[1].clone(),
        address: args[2].clone(),
        user: args[3].clone(),
        password_file: args[4].clone(),
        public_key: args[5].clone(),
        directory: std::path::PathBuf::from(&args[6]),
    })
}

/// The `private-authored-test` dispatch, unchanged in order: content, remote cache at ADDRESS, recovery wait,
/// mount, then the private authored test with `headless = true`. The viewport entry exists only on Windows.
#[cfg(windows)]
pub fn run(args: Args) -> Result<()> {
    let content = Content::read(std::path::Path::new(&args.content))?;
    let mut cache = DiskCache::open_remote(&args.cache, 64 << 20,
        args.address.parse().map_err(|_| Error::Invalid("server address"))?)?;
    cache.wait_for_recovery(true);
    let cache = content.mount(cache)?;
    crate::viewport::private_authored_test(
        cache,
        &args.address,
        &args.user,
        &args.password_file,
        &args.public_key,
        args.directory,
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(args: &[&str]) -> Vec<String> {
        args.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn parse_requires_exactly_eight_args_ending_in_headless() {
        let seven = ["content", "cache", "127.0.0.1:43651", "user", "password", "key", "dir"];
        for count in 0..=7 {
            assert_eq!(parse(&owned(&seven[..count])), Err(USAGE), "{count} args");
        }
        for flag in ["--Headless", "headless", "", "--headless ", "--windowed"] {
            let mut args = owned(&seven);
            args.push(flag.to_string());
            assert_eq!(parse(&args), Err(USAGE), "{flag:?}");
        }
        let mut nine = owned(&seven);
        nine.extend(owned(&["--headless", "extra"]));
        assert_eq!(parse(&nine), Err(USAGE));
        let mut args = owned(&seven);
        args.push("--headless".to_string());
        assert_eq!(parse(&args), Ok(Args {
            content: "content".into(),
            cache: "cache".into(),
            address: "127.0.0.1:43651".into(),
            user: "user".into(),
            password_file: "password".into(),
            public_key: "key".into(),
            directory: std::path::PathBuf::from("dir"),
        }));
    }

    #[test]
    fn usage_is_the_reviewed_grammar_text() {
        assert_eq!(USAGE, "CONTENT CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY COMMAND_DIRECTORY [--headless]");
    }
}
