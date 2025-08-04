use super::commit::{GitCommit, git_log_iter};
use super::hash::HashFormat;
use super::opts::GitLogOptions;
use chrono::NaiveDate;

pub fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| format!("Invalid date format: {e}"))
}

// Sometimes, I find a reference to a repository on trunk from a long time ago.  I know
// the date at which it was referenced, but the repo has obviously progressed since then,
// so the reference is no longer valid.  However, if we knew the commit hash at the date
// it was referenced, we could construct a permalink to the (likely) version of the repo
// as it was referenced.
//
// To do this, we can look through the repository's commits and, assuming nothing has been
// overwritten in the repository's history, find the first commit before the date on
// which it was referenced; therefore, the likely version of the repo that the person
// who referenced it was looking at.
//
// Of course, this doesn't take into account time of reference, which we seldom know, but
// it gives us a good idea.
pub fn find_first_commit_before_date(date: NaiveDate) {
    if let Some(commit) = find_first_commit_before_date_internal(date) {
        println!("{}", commit.hash.short())
    }
}

fn find_first_commit_before_date_internal(date: NaiveDate) -> Option<GitCommit> {
    let mut logs_iter = git_log_iter(
        Some(1),
        Some(&GitLogOptions {
            all: true,
            reverse: true,
            ..Default::default()
        }),
    )
    .peekable();

    // Option 1: the target date is before the start of the repository
    if let Some(first_log_date) = logs_iter
        .peek()
        .map(|next_log| next_log.date.abs.date_naive())
    {
        let first_log_after_target_date = date < first_log_date;
        if first_log_after_target_date {
            return None;
        }
    }

    while let Some(log) = logs_iter.next() {
        let log_date = log.date.abs.date_naive();
        let log_before_target_date = log_date < date;

        if let Some(next_log_date) = logs_iter
            .peek()
            .map(|next_log| next_log.date.abs.date_naive())
        {
            // Option 2: the commit we are looking at is before the target date
            // but the next one is not.
            let next_log_not_before_target_date = next_log_date >= date;

            if log_before_target_date && next_log_not_before_target_date {
                return Some(log);
            }
        } else {
            // Option 3: the commit we are looking at is before the target date
            // and there is no newer commit.
            return Some(log);
        }
    }

    // Option 4: if we get here, we found no matching commit, likely meaning that
    // all commits in the repo occur *after* the target date.
    None
}
