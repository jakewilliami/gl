use colored::*;
use regex::Regex;
use std::process::{Command, Stdio};

// https://stackoverflow.com/a/49476448/12069968 (comment #2)
use crate::config;
use crate::opts::GitLogOptions;

pub fn get_git_log(n: usize, opts: &GitLogOptions) {
    let log: String = git_log(n, opts);
    let tidied: Vec<String> = tidy_git_log(log, opts);

    for l in tidied {
        println!("{}", l);
    }
}

fn tidy_git_log(log: String, opts: &GitLogOptions) -> Vec<String> {
    let re_named = Regex::new(r"<(?P<author>[^>]*)>").unwrap();
    let re = Regex::new(r"<([^>]*)>").unwrap();
    let mut out_log: Vec<String> = Vec::new();
    for l in log.split_terminator('\n') {
        let cleaned_l: String = l.replace('\"', "");
        let auth = re_named
            .captures(&cleaned_l)
            .unwrap()
            .name("author") // using named groups
            .unwrap()
            .as_str()
            .to_string();

        // Need not colour author if colour not set
        // TODO: do I need to use more regex here?  Can I not replace the regex to just match with the author's name (which we already obtained)?
        if opts.colour && config::ME_IDENTITY.contains(&auth.as_str()) {
            let colourised_l = &re.replace(&cleaned_l, |caps: &regex::Captures| {
                format!(
                    "{}{}{}{}",
                    "".normal().white(), // need this to clear the current line of any colours
                    "<".truecolor(192, 207, 227), // this is the light blue colour I have, defined by \e[0m\e[36m$&\e[39m\e[0m
                    &caps[1].truecolor(192, 207, 227),
                    ">".truecolor(192, 207, 227)
                )
            });

            out_log.push(colourised_l.to_string());
        } else {
            out_log.push(cleaned_l.to_string());
        }
    }

    out_log
}

fn git_log(n: usize, opts: &GitLogOptions) -> String {
    let mut n_str = String::new();
    n_str.push('-');
    n_str.push_str(&n.to_string());

    let mut cmd = Command::new("git");
    cmd.arg("log");
    cmd.arg("--color");
    cmd.arg("--no-merges");

    // Specify log format
    cmd.arg(format!("--pretty=format:\"{}\"", log_fmt_str(opts)));
    if !opts.relative {
        cmd.arg("--date=format:\"%a %d %b %Y\"");
    }

    cmd.arg("--abbrev-commit");
    cmd.arg(&n_str);

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
