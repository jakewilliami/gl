use super::version::{self, AsVersion, Version};
use anyhow::Error;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fmt,
    process::{Command, Stdio},
    str::FromStr,
};

// TODO: Prefer std::sync::LazyLock
lazy_static! {
    static ref MAX_COMMIT_LEN: usize = 69;
    static ref META_SEP_CHAR: char = char::from_u32(0x2E3A).unwrap();
    static ref TAG_RE: Regex = Regex::new(
        &format!(
            r"^(?P<raw>(?P<version>(?P<tag>{}))(?:{}(?P<rest>(?:(?P<description>.+)(?:\s+(?P<trailingver>{}))?)))?)$",
            *version::SEMVER_PAT1,
            *META_SEP_CHAR,
            *version::SEMVER_PAT2,
        ),
    )
        .unwrap();
}

#[derive(Clone, Debug)]
struct Tag {
    version: Version,
    #[allow(unused)]
    description: Option<String>,
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            version: Version::new(0, 0, 0),
            description: None,
        }
    }
}

// TODO: should we make all of the worker functions impl of Tag?
impl Tag {
    fn with_version(version: &Version) -> Self {
        Self {
            version: version.clone(),
            description: None,
        }
    }
}

// Display only prints.  For full annotated tag use `Tag::message`.
impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)
    }
}

// Parse Tag from &str
impl FromStr for Tag {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Case 1: we have a simple tag, sans description
        if let Ok(version) = s.as_version() {
            return Ok(Self::with_version(&version));
        }

        // Case 2: we have a complex tag with the tag message
        //
        // In this case, we do something similar to what we do in `commit.rs` and capture
        // and desconstruct all of the relevant information using regex.
        let m = TAG_RE
            .captures(s)
            .ok_or_else(|| anyhow::anyhow!("could not parse tag: {s:?}"))?;

        let version = m.name("version").unwrap().as_str().as_version()?;

        // Ensure the tag version is the same as that in the tag message
        if let Some(v2) = m.name("trailingver") {
            let trailing = v2.as_str().as_version()?;
            assert_eq!(version, trailing);
        }

        let description = m.name("description").map(|m| m.as_str().to_owned());

        Ok(Tag {
            version,
            description,
        })
    }
}

pub fn get_tags() {
    let tags = tags();
    let mut iter = tags.iter().peekable();

    while let Some(tag) = iter.next() {
        print!("{tag}");

        // Print a new line separator unless this is the last line
        if iter.peek().is_some() {
            println!();
        }
    }
}

fn tags() -> Vec<Tag> {
    let mut cmd = Command::new("git");
    cmd.arg("tag");
    cmd.arg("--list");
    // Sort in reverse order by tag name, which is a version number
    //   <https://stackoverflow.com/a/1064505>
    cmd.arg("--sort=-version:refname");
    // Include tag message as well
    //   <https://stackoverflow.com/a/59356030>
    cmd.arg(format!(
        "--format=%(refname:short){}%(subject)",
        *META_SEP_CHAR
    ));

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git tag`");

    if output.status.success() {
        let tags = String::from_utf8_lossy(&output.stdout).into_owned();
        // TODO: or do we want to _warn_ on non-parseable tags?
        tags.lines()
            .map(|s| s.parse::<Tag>().expect("failed to parse Tag"))
            .collect()
    } else {
        // TODO: String::from_utf8_lossy(&output.stderr).into_owned()
        vec![]
    }
}
