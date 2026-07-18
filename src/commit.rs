use crate::{branch, config::SHORT_HASH_LENGTH, count, identity::GitIdentity, opts::GitOptions};
use chrono::{DateTime, Local, NaiveDate};
use regex::Regex;
use std::{
    char,
    process::{Command, Stdio},
    sync::{Arc, LazyLock},
};

// This is a good separating dash, but relies on it not being used inside commit messages!
static META_SEP_CHAR: LazyLock<char> = LazyLock::new(|| char::from_u32(0x2E3A).unwrap());

// Quotes for log metadata
// These need to be unique.  They are not traditional quotes.  See  v3.0.2 and v3.1.2.
static INITIAL_QUOTE_CHAR: LazyLock<char> = LazyLock::new(|| char::from_u32(0x2560).unwrap());
static FINAL_QUOTE_CHAR: LazyLock<char> = LazyLock::new(|| char::from_u32(0x2563).unwrap());

//Regex for commit logs
static UNTIL_FINAL_QUOTE_PAT: LazyLock<String> =
    LazyLock::new(|| format!(r"[^{}]", *FINAL_QUOTE_CHAR));
static DATE_META_PAT: LazyLock<String> =
    LazyLock::new(|| format!(r"(?P<dateabs>{}+)", *UNTIL_FINAL_QUOTE_PAT).quote());
static HASH_META_PAT: LazyLock<String> =
    LazyLock::new(|| String::from(r"(?P<fullhash>[a-f0-9]+)").quote());
static EMAIL_META_PAT: LazyLock<String> =
    LazyLock::new(|| format!(r"(?P<email>{}*)", *UNTIL_FINAL_QUOTE_PAT).quote());
static COMMIT_LOG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        &format!(
            r"^(?P<raw>(?P<hash>[a-f0-9]+)\s\-\s(\((?P<meta>[^\)]+)\)\s)?(?P<message>.+)\((?P<daterepr>[^\)]+)\)\s<(?P<author>[^>]*)>){}dateabs\:\s{},\shash\:\s{},\semail\:\s{}$",
            *META_SEP_CHAR,
            *DATE_META_PAT,
            *HASH_META_PAT,
            *EMAIL_META_PAT,
        ),
    )
        .unwrap()
});

#[derive(Clone)]
pub struct GitCommit {
    pub hash: String,
    #[allow(dead_code)]
    meta: Option<String>,
    #[allow(dead_code)]
    pub message: String,
    pub date: CommitDate,
    pub id: GitIdentity,
    pub raw: String,
}

#[derive(Clone)]
pub struct CommitDate {
    pub abs: DateTime<Local>,
    #[allow(dead_code)]
    repr: String,
}

pub trait HashFormat {
    #[allow(dead_code)]
    fn short(&self) -> String;
}

impl HashFormat for String {
    fn short(&self) -> String {
        // github.com/jakewilliami/mktex/blob/e5430b18/src/remote.rs#L56
        match self.char_indices().nth(SHORT_HASH_LENGTH) {
            None => self.to_string(),
            Some((idx, _)) => (self[..idx]).to_string(),
        }
    }
}

trait Quote {
    fn quote(&self) -> String;
}

impl Quote for String {
    fn quote(&self) -> String {
        format!("{}{}{}", *INITIAL_QUOTE_CHAR, &self, *FINAL_QUOTE_CHAR)
    }
}

pub struct GitLogIter {
    #[allow(dead_code)]
    log_data: Arc<String>,
    lines: std::str::Lines<'static>,
    opts: GitOptions,
}

impl Iterator for GitLogIter {
    type Item = GitCommit;

    fn next(&mut self) -> Option<Self::Item> {
        for log in self.lines.by_ref() {
            // TODO: is this a problem if there are quotes in the string itself?
            let log: String = log.replace('\"', "");
            let log_stripped = strip_ansi_escapes::strip_str(&log);
            if let Some(re_match) = COMMIT_LOG_RE.captures(&log_stripped) {
                return Some(GitCommit {
                    hash: re_match.name("fullhash")?.as_str().to_string(),
                    meta: re_match.name("meta").map(|s| s.as_str().to_string()),
                    message: re_match.name("message")?.as_str().to_string(),
                    date: CommitDate {
                        abs: {
                            let date_str = re_match.name("dateabs")?.as_str();
                            if self.opts.log.relative {
                                DateTime::parse_from_rfc2822(date_str).unwrap().into()
                            } else {
                                // TODO: this is slightly wrong, as it doesn't account for
                                //   the time zone of the commit, it just uses the local
                                //   timezone.  We need to extract the commit time zone from
                                //   the git log command
                                let now = Local::now();
                                let offset = now.offset();
                                NaiveDate::parse_from_str(date_str, "%a %d %b %Y")
                                    .unwrap()
                                    .and_hms_opt(0, 0, 0)
                                    .unwrap()
                                    .and_local_timezone(*offset)
                                    .unwrap()
                                    .into()
                            }
                        },
                        repr: re_match.name("daterepr")?.as_str().to_string(),
                    },
                    id: GitIdentity {
                        email: re_match.name("email")?.as_str().to_string(),
                        names: vec![re_match.name("author")?.as_str().to_string()],
                    },
                    raw: log
                        .split(&META_SEP_CHAR.to_string())
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                });
            }
        }
        None
    }
}

pub fn git_log_iter(
    n: Option<usize>,
    opts: Option<&GitOptions>,
) -> Box<dyn Iterator<Item = GitCommit>> {
    let opts = opts.cloned().unwrap_or_default();
    let log_data = Arc::new(git_log_str(n, &opts));

    // SAFETY: we coerce the lifetime of the `&str` to `'static` because `log_data`
    // is held by the iterator.  This is safe because `lines` doesn't outlive `log_data`.
    let static_ref: &'static str =
        unsafe { std::mem::transmute::<&str, &'static str>(log_data.as_str()) };

    let iter = GitLogIter {
        lines: static_ref.lines(),
        log_data,
        opts: opts.clone(),
    };

    if opts.reverse {
        Box::new(iter.collect::<Vec<_>>().into_iter().rev())
    } else {
        Box::new(iter)
    }
}

pub fn git_log(n: Option<usize>, opts: Option<&GitOptions>) -> Vec<GitCommit> {
    git_log_iter(n, opts).collect()
}

fn git_log_str(n: Option<usize>, opts: &GitOptions) -> String {
    let mut cmd = Command::new("git");
    cmd.arg("log");
    cmd.arg("--color");
    cmd.arg("--no-merges");

    // Specify log format
    // NOTE: at the end of the main format log, we pull additional meta information for the GitCommit struct
    cmd.arg(format!(
        "--pretty=format:\"{}{}dateabs: {}, hash: {}, email: {}\"",
        log_fmt_str(opts),
        *META_SEP_CHAR,
        String::from("%cd").quote(),
        String::from("%H").quote(),
        String::from("%ae").quote(),
    ));

    if opts.log.relative {
        // Even though we don't explicitly print the full date when we show the relative commit time, it is useful to have the RFC-2822 date format for parsing in the GitCommit
        cmd.arg("--date=rfc");
    } else {
        cmd.arg("--date=format:\"%a %d %b %Y\"");
    }

    // Apply log filters
    //
    // Could try with regex:
    //   https://forums.freebsd.org/threads/58555/
    //   https://stackoverflow.com/a/22971024/
    //
    // But it seems to work fine with multiple arguments
    for author in &opts.log.filter.authors {
        // cmd.arg(format!("--author=\"{author}\""));
        cmd.arg("--author").arg(author);
    }

    for needle in &opts.log.filter.needles {
        // cmd.arg(format!("--grep=\"{needle}\""));
        cmd.arg("--grep").arg(needle);
    }

    cmd.arg("--abbrev-commit");

    if let Some(n) = n
        && !opts.log.all
    {
        // If n is defined, restrict the log to only show n of them (only if we don't want to show all logs)
        cmd.arg(format!("-n {n}"));

        // If the number of logs is defined, but so is rev, then we want to skip some number of logs
        // Note: if --all is specified, we don't want to skip anything.  --rev will be handled upstream if needed
        if opts.reverse {
            let log_count = count::commit_count();
            cmd.arg(format!("--skip={}", log_count - n));
        }
    }

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git log`");

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).into_owned()
    } else {
        // TODO: String::from_utf8_lossy(&output.stderr).into_owned()
        println!(
            "An error has occured.  It is likely that you aren't in a git repository, or you may not have `git` installed."
        );

        "".to_string()
    }
}

fn log_fmt_str(opts: &GitOptions) -> String {
    // TODO: add option for commit format H (long hash)
    let commit = colourise_log_fmt("h", Some("bold yellow"), None, None, opts);
    let branch_tag = colourise_log_fmt("d", Some("bold green"), Some("-"), None, opts);
    let msg = colourise_log_fmt("s", None, None, Some(""), opts);
    let time = colourise_log_fmt(
        if opts.log.relative { "cr" } else { "cd" },
        Some("bold red"),
        None,
        Some("()"),
        opts,
    );
    let author = colourise_log_fmt("an", Some("bold blue"), None, Some("<>"), opts);
    format!("{commit} {branch_tag} {msg} {time} {author}")
}

fn colourise_log_fmt(
    fmt: &str,
    colour: Option<&str>,
    prefix: Option<&str>,
    enclosing_chars: Option<&str>,
    opts: &GitOptions,
) -> String {
    let prefix = prefix.unwrap_or("");
    let (enclosing_start, enclosing_end) = get_enclosing(enclosing_chars);
    if opts.colour
        && let Some(colour) = colour
    {
        format!("{prefix}%C({colour}){enclosing_start}%{fmt}{enclosing_end}%Creset")
    } else {
        format!("{prefix}{enclosing_start}%{fmt}{enclosing_end}")
    }
}

fn get_enclosing(enclosing_chars: Option<&str>) -> (&str, &str) {
    let enclosing_chars = enclosing_chars.unwrap_or("");
    if enclosing_chars.is_empty() {
        ("", "")
    } else {
        let i = enclosing_chars.len() / 2 + enclosing_chars.len() % 2;
        let (enclosing_start, enclosing_end) = enclosing_chars.split_at(i);
        (enclosing_start, enclosing_end)
    }
}

pub fn has_commits() -> bool {
    // This function will run `git rev-parse HEAD`
    branch::current_branch().is_some()
}
