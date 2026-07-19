use crate::{
    commit::{GitCommit, git_log_iter, has_commits},
    display::Format,
    opts::{GitOptions, TagFormat},
    origin::remote_origin_url,
    version::{self, AsVersion, Bump, NextVersion, Version},
};
use anyhow::{Error, anyhow};
use colored::*;
use dialoguer::{Confirm, Input};
use itertools::Itertools;
use regex::Regex;
use std::{
    fmt,
    process::{Command, Stdio},
    str::FromStr,
    sync::LazyLock,
};

const MAX_COMMIT_LEN: usize = 69;
static META_SEP_CHAR: LazyLock<char> = LazyLock::new(|| char::from_u32(0x2E3A).unwrap());
static TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^(?P<raw>(?:(?P<first>(?P<version>(?P<tag>{}))){})?(?P<rest>(?:(?P<description>.+?)(?:\s+(?P<trailingver>{}))?))?)$",
        *version::SEMVER_PAT1,
        *META_SEP_CHAR,
        *version::SEMVER_PAT2,
    )).unwrap()
});

#[derive(Clone, Debug)]
struct Tag {
    version: Version,
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

    fn with_description(version: &Version, description: impl Into<String>) -> Self {
        Self {
            version: version.clone(),
            description: Some(description.into()),
        }
    }

    // TODO: should I take a description or always pull it from Self?
    fn message(&self) -> String {
        if let Some(description) = &self.description {
            format!("{}!  {}", description, self.version)
        } else {
            format!("{}", self.version)
        }
    }
}

// Display only prints.  For full annotated tag use `Tag::message`.
impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)
    }
}

// Pretty formatting with colours
impl Format for Tag {
    fn pretty(&self, opts: &GitOptions) -> String {
        let version = self.version.to_string();
        let version = if opts.colour {
            version.bold().green().to_string()
        } else {
            version
        };

        match &self.description {
            Some(desc) => format!("{} {} {}", version, "-", desc),
            None => version,
        }
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
            .ok_or_else(|| anyhow!("could not parse tag: {s:?}"))?;

        // Fall back on trailingver in case there is no `first` (as sometimes we are just
        // parsing tag from commit message)
        let version = m
            .name("version")
            .or_else(|| m.name("trailingver"))
            .ok_or_else(|| anyhow!("no version found in: {s:?}"))?
            .as_str()
            .as_version()?;

        // Ensure the tag version is the same as that in the tag message
        if let Some(v2) = m.name("trailingver") {
            let trailing = v2.as_str().as_version()?;
            assert_eq!(version, trailing);
        }

        let description = m
            .name("description")
            .map(|m| m.as_str().trim().trim_end_matches('!').trim().to_owned());

        Ok(Tag {
            version,
            description,
        })
    }
}

// Some commit messages look like tags.  We can try to parse them as such.
impl TryFrom<&GitCommit> for Tag {
    type Error = Error;

    fn try_from(commit: &GitCommit) -> Result<Self, Self::Error> {
        commit.message.trim().parse::<Tag>()
    }
}

pub fn tag() {
    if !has_commits() {
        eprintln!("[ERROR] Cannot create a tag on a repository with no commits");
        return;
    }

    let existing_tags = tags();
    let latest_tag = existing_tags.clone().into_iter().next().unwrap_or_default();
    let latest_commit = git_log_iter(Some(1), None)
        .exactly_one()
        .unwrap_or_else(|_| panic!("expected exactly one commit"));

    // Step 1: ask user for new tag version
    let tag_version = prompt_tag_version(&existing_tags, &latest_tag, &latest_commit);

    // Step 2: get tag annotation/description from user
    let description = prompt_description(&tag_version, &latest_commit);

    // Step 3: create tag with user-input values
    let tag = Tag::with_description(&tag_version, description);

    // Step 4: confirm tag generation with user
    if !confirm_create_tag(&tag) {
        return;
    }

    create_tag(&tag);

    // Step 5: ask user if they want to push the tag to remote/origin
    if confirm_push() {
        push_tag(&tag);

        // Open relevant in origin so that the user can create a release based on this tag
        //
        // TODO: add option to disable opening these resources?
        open_release_urls(&latest_commit);
    }
}

fn prompt_tag_version(
    existing_tags: &[Tag],
    latest_tag: &Tag,
    latest_commit: &GitCommit,
) -> Version {
    // Suggest next tag from latest commit but fall back to patch bump from latest tag
    //
    // TODO: should we confirm that this matches Cargo.toml?  Or at least the version of
    //   Cargo.toml (or Cargo.lock) that is staged, in case there are unstaged changes
    let next_tag = if let Ok(commit_tag) = Tag::try_from(latest_commit) {
        commit_tag.version
    } else {
        latest_tag.version.bump_patch()
    };

    Input::<Version>::new()
        .with_prompt(format!("Tag name (current: {})", latest_tag.version))
        .default(next_tag)
        .validate_with(|v: &Version| {
            // Validation step 1: check that the tag is not a downgrade
            if *v <= latest_tag.version {
                return Err(format!(
                    "version {} is not greater than current {}",
                    v, latest_tag.version
                ));
            }

            // Validation step 2: check that the tag given by the user doesn't already exist
            if existing_tags.iter().any(|t| t.version == *v) {
                return Err(format!("tag {} already exists", v));
            }

            // Validation step 3: check that the tag given by the user is sequential given
            //   the previous tag
            if let Some(suggested) = latest_tag.version.next_version(v) {
                if !Confirm::new()
                    .with_prompt(format!(
                        "Not a sequential bump (expected {}), continue anyway?",
                        suggested
                    ))
                    .default(false)
                    .interact()
                    .unwrap()
                {
                    return Err(String::from("enter a different version"));
                }
            }

            // Validation step 4: check if the tag given by the user aligns with the commit
            //   message, if the commit message is in a tag-like format
            if let Ok(commit_tag) = Tag::try_from(latest_commit) {
                if commit_tag.version != *v {
                    if !Confirm::new()
                        .with_prompt(format!(
                            "Latest commit is for {}, not {}, continue anyway?",
                            commit_tag.version, v
                        ))
                        .default(false)
                        .interact()
                        .unwrap()
                    {
                        return Err(String::from("enter a different version"));
                    }
                }
            }

            // If we got here, then the tag should be good enough
            Ok(())
        })
        .interact_text()
        .unwrap()
}

fn prompt_description(tag_version: &Version, latest_commit: &GitCommit) -> String {
    // Suggest the commit message description if it is formatted like a tag
    let mut initial = Tag::try_from(latest_commit)
        .ok()
        .and_then(|t| t.description)
        .unwrap_or_default();

    // We loop here because we want to keep asking for the tag message if it was too long
    //
    // TODO: support more emacs keybindings when editing like C-a and M-f?
    // TODO: add grey after line length exceeded?
    // TODO: optionally edit commit message in magit?
    loop {
        let input = Input::<String>::new()
            .with_prompt("Message")
            .with_initial_text(&initial)
            .interact_text()
            .unwrap()
            .trim()
            .trim_end_matches('!')
            .trim()
            .to_owned();

        // Check if description matches the commit message
        if let Ok(commit_tag) = Tag::try_from(latest_commit) {
            if let Some(commit_desc) = &commit_tag.description {
                if &input != commit_desc {
                    if !Confirm::new()
                        .with_prompt(format!(
                            "Message differs from latest commit message ({:?}), continue anyway?",
                            commit_desc
                        ))
                        .default(false)
                        .interact()
                        .unwrap()
                    {
                        initial = input;
                        continue;
                    }
                }
            }
        }

        let msg = Tag::with_description(tag_version, &input).message();

        if msg.len() > MAX_COMMIT_LEN {
            eprintln!(
                "Message must be {} characters or fewer (currently {})",
                MAX_COMMIT_LEN,
                msg.len()
            );
            initial = input;
        } else {
            break input;
        }
    }
}

fn confirm_create_tag(tag: &Tag) -> bool {
    Confirm::new()
        .with_prompt(format!(
            "Create tag {tag} with message {:?}?",
            tag.message()
        ))
        .default(true)
        .interact()
        .unwrap()
}

fn confirm_push() -> bool {
    Confirm::new()
        .with_prompt("Push tag to origin?")
        .default(true)
        .interact()
        .unwrap()
}

fn open_release_urls(latest_commit: &GitCommit) {
    if let Some(origin) = remote_origin_url() {
        // Open latest commit and create a new release
        let _ = open::that(format!("{origin}/commit/{}", latest_commit.hash));
        let _ = open::that(format!("{origin}/releases/new"));
    }
}

pub fn get_tags(opts: &GitOptions) {
    let mut tags = tags();

    if opts.reverse {
        tags.reverse()
    }

    for tag in tags {
        match opts.tag.fmt {
            TagFormat::Short => println!("{}", tag.version),
            TagFormat::Long => println!("{}", tag.pretty(opts)),
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

fn create_tag(tag: &Tag) {
    let mut cmd = Command::new("git");
    cmd.arg("tag");
    cmd.arg("--annotate");
    cmd.arg(tag.version.to_string());
    cmd.arg(format!("--message={}", tag.message()));

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git tag`");

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).into_owned();
        eprintln!("[ERROR] {err}");
    }
}

fn push_tag(tag: &Tag) {
    let mut cmd = Command::new("git");
    cmd.arg("push");
    cmd.arg("origin");
    cmd.arg(tag.version.to_string());

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git push`");

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).into_owned();
        eprintln!("[ERROR] {err}");
    }
}
