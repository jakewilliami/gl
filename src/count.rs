use super::{
    branch::current_branch,
    opts::GitLogOptions,
    repo::{current_repository, discover_repository},
};
use chrono::{DateTime, Duration, Local, NaiveTime};
use colored::*;

struct TimeRange {
    start: i64,
    end: i64,
}

pub fn get_commit_count(input: &str, opts: &GitLogOptions) {
    // determine commit count
    let commit_count_val: usize;

    if input == "today" {
        commit_count_val = commit_count_today();
    } else if input == "yesterday" {
        commit_count_val = commit_count_yesterday();
    } else {
        let days_ago: usize = input.parse().unwrap_or_else(|e| {
            panic!("{e}: argument must be a valid integer, but got {:?}", input)
        });
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
        _ => format!("in the past {} days", input),
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
        println!("{}", out_message);
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
        println!("{}", out_message);
    }
}

fn commit_count_today() -> usize {
    // get the date of interest as a number of seconds
    let today_start: i64 = Local::now().with_time(NaiveTime::MIN).unwrap().timestamp();
    let now: i64 = Local::now().timestamp();

    // get the commit count for this period
    commit_count_core(Some(TimeRange {
        start: today_start,
        end: now,
    }))
}

fn commit_count_yesterday() -> usize {
    // get the datetimes of interest
    let today_start: DateTime<Local> = Local::now().with_time(NaiveTime::MIN).unwrap();
    let yesterday_start: DateTime<Local> = today_start - Duration::days(1);
    // calculate those values in seconds
    let today_timestamp: i64 = today_start.timestamp();
    let yersterday_timestamp: i64 = yesterday_start.timestamp();

    // get the commit count for this period
    commit_count_core(Some(TimeRange {
        start: yersterday_timestamp,
        end: today_timestamp,
    }))
}

fn commit_count_since(n: usize) -> usize {
    // get the datetimes of interest
    let today_start: DateTime<Local> = Local::now().with_time(NaiveTime::MIN).unwrap();
    let since_start: DateTime<Local> = today_start - Duration::days(n as i64);
    let now: i64 = Local::now().timestamp();
    // calculate those values in seconds
    let since_timestamp: i64 = since_start.timestamp();

    // get the commit count for this period
    commit_count_core(Some(TimeRange {
        start: since_timestamp,
        end: now,
    }))
}

pub fn commit_count() -> usize {
    commit_count_core(None)
}

fn commit_count_core(rng: Option<TimeRange>) -> usize {
    // TODO: we should probably give a good error message here
    let repo = discover_repository().unwrap();

    // Get most recent commit at HEAD
    let commit = repo.head_commit().unwrap();

    // Count non-merge commits on HEAD
    repo.rev_walk([commit.id])
        .all()
        .unwrap()
        .filter(|info| {
            if let Ok(info) = info {
                // Get commit info
                let commit = info.object().unwrap();
                let commit_ref = commit.decode().unwrap();

                // We want to filter out merges.  We can do this by filtering out
                // the commit if it has more than one parent
                let mut parents = commit_ref.parents();
                parents.next();
                if parents.next().is_some() {
                    return false;
                }

                // Filter between the time frame
                if let Some(rng) = &rng {
                    let committer = commit_ref.committer();
                    if rng.start > committer.time.seconds || committer.time.seconds > rng.end {
                        return false;
                    }
                }

                true
            } else {
                false
            }
        })
        .fold(0, |count, _| count + 1)
}
