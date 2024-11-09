use super::{
    count,
    date::CommitDate,
    identity::GitIdentity,
    opts::GitLogOptions,
    quote::{Quote, FINAL_QUOTE_CHAR},
};
use chrono::{DateTime, Local, NaiveDate};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    char,
    process::{Command, Stdio},
};

lazy_static! {
    // This is a good separating dash, but relies on it not being used inside commit messages!
    static ref META_SEP_CHAR: char = char::from_u32(0x2E3A).unwrap();

    //Regex for commit logs
    static ref UNTIL_FINAL_QUOTE_PAT: String = format!(r"[^{}]", *FINAL_QUOTE_CHAR);
    static ref DATE_ABS_META_PAT: String = format!(r"(?P<dateabs>{}+)", *UNTIL_FINAL_QUOTE_PAT).quote();
    static ref DATE_REL_META_PAT: String = format!(r"(?P<daterel>{}+)", *UNTIL_FINAL_QUOTE_PAT).quote();
    static ref HASH_META_PAT: String = String::from(r"(?P<fullhash>[a-f0-9]+)").quote();
    static ref EMAIL_META_PAT: String = format!(r"(?P<email>{}*)", *UNTIL_FINAL_QUOTE_PAT).quote();
    static ref COMMIT_LOG_RE: Regex = Regex::new(
        &format!(
            r"^(?P<raw>(?P<hash>[a-f0-9]+)\s\-\s(\((?P<refs>[^\)]+)\)\s)?(?P<message>.+)\((?P<daterepr>[^\)]+)\)\s<(?P<author>[^>]*)>){}dateabs\:\s{},\sdaterel\:\s{},\shash\:\s{},\semail\:\s{}$",
            *META_SEP_CHAR,
            *DATE_ABS_META_PAT,
            *DATE_REL_META_PAT,
            *HASH_META_PAT,
            *EMAIL_META_PAT,
        ),
    )
        .unwrap();
}

#[derive(Clone)]
pub struct CommitHash {
    full: String,
    pub short: String,
}

#[derive(Clone)]
pub struct GitCommit {
    pub hash: CommitHash,
    pub message: String,
    pub refs: Vec<String>,
    pub date: CommitDate,
    pub id: GitIdentity,
    pub raw: String,
}

pub fn git_log(n: Option<usize>, opts: Option<&GitLogOptions>) -> Vec<GitCommit> {
    let opts = if let Some(opts) = opts {
        opts.clone()
    } else {
        GitLogOptions::default()
    };

    let mut logs: Vec<GitCommit> = Vec::new();
    let logs_str = git_log_str(n, &opts);
    for log in logs_str.split_terminator('\n') {
        let log: String = log.replace('\"', "");
        let log_stripped = strip_ansi_escapes::strip_str(&log);
        let re_match = COMMIT_LOG_RE.captures(&log_stripped).unwrap();

        logs.push(GitCommit {
            hash: CommitHash {
                full: re_match.name("fullhash").unwrap().as_str().to_string(),
                short: re_match.name("hash").unwrap().as_str().to_string(),
            },
            refs: re_match
                .name("refs")
                .iter()
                .map(|s| s.as_str().to_string())
                .collect(),
            message: re_match.name("message").unwrap().as_str().to_string(),
            date: CommitDate {
                abs: {
                    let date_str = re_match.name("dateabs").unwrap().as_str();
                    if opts.relative {
                        DateTime::parse_from_rfc2822(date_str).unwrap().into()
                    } else {
                        // TODO: this is slightly wrong, as it doesn't account for the time zone of the commit, it just uses the local timezone.  We need to extract the commit time zone from the git log command
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
                repr: re_match.name("daterepr").unwrap().as_str().to_string(),
                rel: re_match.name("daterel").unwrap().as_str().to_string(),
            },
            id: GitIdentity {
                email: re_match.name("email").unwrap().as_str().to_string(),
                names: vec![re_match.name("author").unwrap().as_str().to_string()],
            },
            // If the separating char is used in the commit message then it's Joever
            raw: log
                .split(&META_SEP_CHAR.to_string())
                .next()
                .unwrap_or("")
                .trim()
                .to_string(),
        });
    }

    // Account for reverse option
    if opts.reverse {
        logs.into_iter().rev().collect()
    } else {
        logs
    }
}

fn git_log_str(n: Option<usize>, opts: &GitLogOptions) -> String {
    let mut cmd = Command::new("git");
    cmd.arg("log");
    cmd.arg("--color");
    cmd.arg("--no-merges");

    // Specify log format
    // NOTE: at the end of the main format log, we pull additional meta information for the GitCommit struct
    cmd.arg(format!(
        "--pretty=format:\"{}{}dateabs: {}, daterel: {}, hash: {}, email: {}\"",
        log_fmt_str(opts),
        *META_SEP_CHAR,
        String::from("%cd").quote(),
        String::from("%cr").quote(),
        String::from("%H").quote(),
        String::from("%ae").quote(),
    ));

    if opts.relative {
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
    for author in &opts.authors {
        // cmd.arg(format!("--author=\"{author}\""));
        cmd.arg("--author").arg(author);
    }

    for needle in &opts.needles {
        // cmd.arg(format!("--grep=\"{needle}\""));
        cmd.arg("--grep").arg(needle);
    }

    cmd.arg("--abbrev-commit");

    if let Some(n) = n {
        if !opts.all {
            // If n is defined, restrict the log to only show n of them (only if we don't want to show all logs)
            cmd.arg(format!("-n {}", n));

            // If the number of logs is defined, but so is rev, then we want to skip some number of logs
            // Note: if --all is specified, we don't want to skip anything.  --rev will be handled upstream if needed
            if opts.reverse {
                let log_count = count::commit_count();
                cmd.arg(format!("--skip={}", log_count - n));
            }
        }
    }

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git log`");

    if output.status.success() {
        let git_log = String::from_utf8_lossy(&output.stdout).into_owned();

        git_log
    } else {
        println!("An error has occured.  It is likely that you aren't in a git repository, or you may not have `git` installed.");

        "".to_string()
    }
}

fn log_fmt_str(opts: &GitLogOptions) -> String {
    // TODO: add option for commit format H (long hash)
    let commit = colourise_log_fmt("h", Some("bold yellow"), None, None, opts);
    let branch_tag = colourise_log_fmt("d", Some("bold green"), Some("-"), None, opts);
    let msg = colourise_log_fmt("s", None, None, Some(""), opts);
    let time = colourise_log_fmt(
        if opts.relative { "cr" } else { "cd" },
        Some("bold red"),
        None,
        Some("()"),
        opts,
    );
    let author = colourise_log_fmt("an", Some("bold blue"), None, Some("<>"), opts);
    format!("{} {} {} {} {}", commit, branch_tag, msg, time, author)
}

fn colourise_log_fmt(
    fmt: &str,
    colour: Option<&str>,
    prefix: Option<&str>,
    enclosing_chars: Option<&str>,
    opts: &GitLogOptions,
) -> String {
    let prefix = prefix.unwrap_or("");
    let (enclosing_start, enclosing_end) = get_enclosing(enclosing_chars);
    if opts.colour && colour.is_some() {
        let colour = colour.unwrap();
        format!(
            "{}%C({}){}%{}{}%Creset",
            prefix, colour, enclosing_start, fmt, enclosing_end
        )
    } else {
        format!("{}{}%{}{}", prefix, enclosing_start, fmt, enclosing_end)
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
