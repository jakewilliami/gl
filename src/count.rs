use super::branch::current_branch;
use super::opts::GitLogOptions;
use super::repo::current_repository;
use chrono::{DateTime, Duration, Local, NaiveTime};
use colored::*;
use std::process::{Command, Output, Stdio};

// const local: DateTime<Local> = Local::now();
// const today = Utc.ymd(local.year(), local.month(), local.day())
// let today: Date<Local> = Local::today();
// const yesterday = today - Duration::days(1);

pub fn get_commit_count(input: &str, opts: &GitLogOptions) {
    // determine commit count
    let commit_count_val: usize;

    if input == "today" {
        commit_count_val = commit_count_today();
    } else if input == "yesterday" {
        commit_count_val = commit_count_yesterday();
    } else {
        let days_ago: usize = input
            .parse()
            .unwrap_or_else(|e| panic!("{e}: argument must be a valid integer, but got {input:?}"));
        commit_count_val = commit_count_since(days_ago);
    }
    // let commit_count_val = commit_count(days_ago, days_ago_end);

    // get repository information
    let repo_name = current_repository();
    let branch_name = current_branch();

    // determine human-readable "since when" relative time
    let plural_maybe = match commit_count_val {
        1 => "",
        _ => "s",
    };
    let when = match input {
        "today" | "yesterday" => String::from(input),
        _ => format!("in the past {input} days"),
    };
    let verb_tense = match input {
        "yesterday" => "were",
        _ => match commit_count_val {
            1 => "has been made",
            _ => "have been made",
        },
    };

    // print output
    // format output nicely (and ensure it's lovely and green)
    let out_message = format!(
        // n commits have been made to {}/{} today
        // n commits were made to {}/{} yesterday
        // n commits have been made to {}/{} in the past {} days
        "{} commit{} {} to {}/{} {}.",
        commit_count_val,
        plural_maybe,
        verb_tense,
        repo_name.unwrap(),
        branch_name.unwrap(),
        when,
    );

    if opts.colour {
        println!("{}", out_message.green().bold());
    } else {
        println!("{out_message}");
    }
}

pub fn get_commit_count_total(opts: &GitLogOptions) {
    // determine commit count
    let commit_count_val = commit_count();

    // get repository information
    let repo_name = current_repository();
    let branch_name = current_branch();

    let plural_maybe = match commit_count_val {
        1 => "",
        _ => "s",
    };
    let have_plural_maybe = match commit_count_val {
        1 => "has",
        _ => "has",
    };

    // format output nicely (and ensure it's lovely and green)
    let out_message = format!(
        "{} commit{} {} been made to {}/{}.",
        commit_count_val,
        plural_maybe,
        have_plural_maybe,
        repo_name.unwrap(),
        branch_name.unwrap(),
    );

    if opts.colour {
        println!("{}", out_message.green().bold());
    } else {
        println!("{out_message}");
    }
}

fn commit_count_today() -> usize {
    // get the date of interest as a number of seconds
    let today_start: i64 = Local::now().with_time(NaiveTime::MIN).unwrap().timestamp();
    let now: i64 = Local::now().timestamp();

    // get the commit count for this period
    commit_count_between(today_start, now)
}

fn commit_count_yesterday() -> usize {
    // get the datetimes of interest
    let today_start: DateTime<Local> = Local::now().with_time(NaiveTime::MIN).unwrap();
    let yesterday_start: DateTime<Local> = today_start - Duration::days(1);
    // calculate those values in seconds
    let today_timestamp: i64 = today_start.timestamp();
    let yersterday_timestamp: i64 = yesterday_start.timestamp();
    // let date_of_interest: i64 = (since - before) as i64;
    // let timestamp_of_interest: i64 = (today - Duration::days(date_of_interest)).timestamp();

    // get the commit count for this period
    commit_count_between(yersterday_timestamp, today_timestamp)
}

fn commit_count_since(n: usize) -> usize {
    // get the datetimes of interest
    let today_start: DateTime<Local> = Local::now().with_time(NaiveTime::MIN).unwrap();
    let since_start: DateTime<Local> = today_start - Duration::days(n as i64);
    let now: i64 = Local::now().timestamp();
    // calculate those values in seconds
    let since_timestamp: i64 = since_start.timestamp();

    // get the commit count for this period
    commit_count_between(since_timestamp, now)
}

fn commit_count_between(since_timestamp: i64, before_timestamp: i64) -> usize {
    // construct git command line arguments
    let mut since_arg = String::new();
    since_arg.push_str("--since=");
    since_arg.push_str(since_timestamp.to_string().as_str());
    let mut before_arg = String::new();
    before_arg.push_str("--before=");
    before_arg.push_str(before_timestamp.to_string().as_str());

    // git rev-list --count --since=$START_TODAY --before=$NOW HEAD
    let since = since_arg.as_str();
    let before = before_arg.as_str();
    commit_count_core(vec![since, before])
}

pub fn commit_count() -> usize {
    commit_count_core(vec![])
}

fn commit_count_core(args: Vec<&str>) -> usize {
    // run command
    // git rev-list --count HEAD
    let mut cmd = Command::new("git");
    cmd.arg("rev-list");
    cmd.arg("--count");
    cmd.arg("--no-merges");
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg("HEAD");

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git rev-list`");

    if let Some(output) = parse_commit_count(output) {
        match output.parse::<usize>() {
            Ok(n) => n,
            Err(e) => {
                eprintln!("[ERROR] Failed to parse {output:?} as `usize`: {e}");
                0
            }
        }
    } else {
        eprintln!("[ERROR] Failed to get output from `git rev-list`");
        0
    }
}

fn parse_commit_count(cmd_out: Output) -> Option<String> {
    // return appropriately (error silently)
    if cmd_out.status.success() {
        let mut commit_count = String::from_utf8_lossy(&cmd_out.stdout).into_owned();

        if commit_count.ends_with('\n') {
            commit_count.pop();
            if commit_count.ends_with('\r') {
                commit_count.pop();
            }
        }
        Some(commit_count)
    } else {
        None
    }
}
