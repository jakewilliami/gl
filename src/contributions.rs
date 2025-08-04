use super::commit::{GitCommit, git_log};
use super::identity::GitIdentity;
use chrono::{Duration, Local, NaiveDate};
use regex::Regex;
use std::cmp::max;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use tabular::{Table, row};
use textplots::{
    Chart, ColorPlot, LabelBuilder, LabelFormat, Shape, TickDisplay, TickDisplayBuilder,
};

// Types

#[derive(Clone)]
pub struct GitContributor {
    id: GitIdentity,
    contributions: GitContributions,
}

#[derive(Clone)]
struct GitContributions {
    commits: Vec<GitCommit>,
    file_contributions: Vec<GitFileContributions>,
}

#[derive(Clone)]
struct GitFileContributions {
    lines_added: usize,
    lines_deleted: usize,
    lines_written: isize,
}

struct ContributionStats {
    #[allow(dead_code)]
    commits: usize,
    file_contributions: GitFileContributions,
    #[allow(dead_code)]
    commit_dates: HashMap<NaiveDate, usize>,
}

// Traits/implementations

trait ContributorStats {
    fn commits(&self) -> usize;
    fn file_contributions(&self) -> GitFileContributions;
    fn commit_dates(&self) -> HashMap<NaiveDate, usize>;
    fn contribution_stats(&self) -> ContributionStats;
}

impl ContributorStats for GitContributor {
    fn commits(&self) -> usize {
        self.contributions.commits.len()
    }

    fn file_contributions(&self) -> GitFileContributions {
        let mut lines_added: usize = 0;
        let mut lines_deleted: usize = 0;
        let mut lines_written: isize = 0;

        for contribution in &self.contributions.file_contributions {
            lines_added += contribution.lines_added;
            lines_deleted += contribution.lines_deleted;
            lines_written += contribution.lines_written;
        }

        GitFileContributions {
            lines_added,
            lines_deleted,
            lines_written,
        }
    }

    fn commit_dates(&self) -> HashMap<NaiveDate, usize> {
        let mut commit_dates = HashMap::new();
        for commit in &self.contributions.commits {
            commit_dates
                .entry(commit.date.abs.date_naive())
                .and_modify(|n| *n += 1)
                .or_insert(1);
        }
        commit_dates
    }

    fn contribution_stats(&self) -> ContributionStats {
        ContributionStats {
            commits: self.commits(),
            file_contributions: self.file_contributions(),
            commit_dates: self.commit_dates(),
        }
    }
}

// Display methods

pub fn display_git_contributions_per_author(contributors: Vec<GitContributor>) {
    let mut contributors_with_summary: Vec<(GitContributor, ContributionStats)> = Vec::new();
    for contributor in contributors {
        let contrib_summary = contributor.contribution_stats();
        contributors_with_summary.push((contributor, contrib_summary));
    }
    // Sort by sum of lines added and deleted (in reverse order)
    contributors_with_summary.sort_by(|a, b| {
        (b.1.file_contributions.lines_added + b.1.file_contributions.lines_deleted)
            .cmp(&(a.1.file_contributions.lines_added + a.1.file_contributions.lines_deleted))
    });

    let mut table = Table::new("{:<}  {:>}  {:>}  {:>}").with_row(row!(
        "Author",
        "Lines added",
        "Lines deleted",
        "Lines of code"
    ));

    for (contributor, contrib_summary) in contributors_with_summary {
        table.add_row(row!(
            contributor.id.email,
            contrib_summary.file_contributions.lines_added,
            contrib_summary.file_contributions.lines_deleted,
            contrib_summary.file_contributions.lines_written,
        ));
    }
    println!("{table}");
}

pub fn display_git_author_frequency(contributors: Vec<GitContributor>) {
    // Sort by commits (in reverse order)
    let mut contributors_sorted = contributors;
    contributors_sorted.sort_by(|a, b| {
        b.contributions
            .commits
            .len()
            .cmp(&a.contributions.commits.len())
    });

    let mut table = Table::new("{:<}  {:>}").with_row(row!("Author", "Commits"));

    for contributor in contributors_sorted {
        table.add_row(row!(
            contributor.id.email,
            contributor.contributions.commits.len()
        ));
    }

    println!("{table}");
}

pub fn display_git_contributions_graph(contributors: Vec<GitContributor>) {
    let commit_dates_map = git_contributions_by_date(contributors);
    let commit_dates = git_contributions_by_date_vec(&commit_dates_map);

    // Get terminal size to inform graph size
    let tsz = termsize::get().unwrap_or(termsize::Size { rows: 0, cols: 0 });

    // Set reasonable defaults for graph size
    let w: u32 = max(if tsz.cols == 0 { 40 } else { tsz.cols.into() }, 32);
    let h: u32 = max(if tsz.rows == 0 { 60 } else { tsz.rows.into() }, 3);

    // Compute points
    let points = commit_dates
        .iter()
        .enumerate()
        .map(|(i, (_d, n))| (i as f32, *n as f32))
        .collect::<Vec<_>>();

    // Get x bounds
    let xmax = commit_dates.len();
    let xstart = commit_dates[0].0;

    // Construct chart
    // See: github.com/loony-bean/textplots-rs/blob/63a418da/examples/label.rs
    Chart::new(w, h, 0.0, xmax as f32)
        .linecolorplot(
            &Shape::Lines(&points),
            // TODO: consider a more dynamic approach to colour selection as terminal background colour may differ
            rgb::RGB {
                r: 10,
                g: 100,
                b: 200,
            },
        )
        .x_label_format(LabelFormat::Custom(Box::new(move |val| {
            format!("{}", xstart + Duration::days(val as i64))
        })))
        .y_label_format(LabelFormat::Custom(Box::new(move |val| {
            format!("{}", val as isize)
        })))
        .y_tick_display(TickDisplay::Dense)
        .nice();
}

#[allow(dead_code)]
fn coarsen_contributions_by_date_vec(
    contributions_by_date_vec: Vec<(NaiveDate, usize)>,
    bins: usize,
) -> Vec<usize> {
    // TODO: handle bins > length of input vector
    let m = contributions_by_date_vec.len() / bins + 1;
    let mut v = vec![0; bins];
    for (i, (_d, n)) in contributions_by_date_vec.iter().enumerate() {
        let j = i / m;
        v[j] += n;
    }

    v
}

fn git_contributions_by_date_vec(
    contributions_by_date: &HashMap<NaiveDate, usize>,
) -> Vec<(NaiveDate, usize)> {
    let d1 = contributions_by_date.keys().min().unwrap();
    let d2 = Local::now().date_naive();

    let mut contributions = Vec::new();
    let mut d = *d1;
    while d <= d2 {
        let n = contributions_by_date.get(&d).unwrap_or(&0);
        contributions.push((d, *n));
        d += Duration::days(1);
    }

    contributions
}

fn git_contributions_by_date(contributors: Vec<GitContributor>) -> HashMap<NaiveDate, usize> {
    let mut commit_dates: HashMap<NaiveDate, usize> = HashMap::new();
    for contributor in contributors {
        for (date, count) in contributor.commit_dates().iter() {
            commit_dates
                .entry(*date)
                .and_modify(|n| *n += count)
                .or_insert(*count);
        }
    }
    commit_dates
}

// Constructor methods

pub fn git_contributors() -> Vec<GitContributor> {
    // Step 1: calculate author-specific contributions
    let logs: Vec<GitCommit> = git_log(None, None);
    let mut commits_per_author: HashMap<String, Vec<GitCommit>> = HashMap::new();
    for log in logs {
        let email = log.clone().id.email;
        commits_per_author
            .entry(email)
            .and_modify(|v| (*v).push(log.clone()))
            .or_insert(vec![log]);
    }

    // Step 2: combine previous commit date data with file contributions
    let author_frequency = git_author_frequency();
    let mut contributors: Vec<GitContributor> = Vec::new();
    for (email, (identity, _n_commits)) in author_frequency {
        contributors.push(GitContributor {
            id: GitIdentity {
                email: email.clone(),
                names: vec![],
            },
            contributions: GitContributions {
                commits: commits_per_author.get(&email).unwrap_or(&vec![]).to_vec(),
                file_contributions: git_file_contributions_per_author(identity),
            },
        });
    }

    contributors
}

fn git_file_contributions_per_author(identity: GitIdentity) -> Vec<GitFileContributions> {
    // git log --no-merges --author="SOME AUTHOR OR EMAIL" --pretty=tformat: --numstat
    let mut cmd = Command::new("git");
    cmd.arg("log");
    cmd.arg("--no-merges");
    cmd.arg(format!("--author={}", identity.email));
    cmd.arg("--pretty=tformat:");
    cmd.arg("--numstat");

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git log`");

    if output.status.success() {
        let git_log = String::from_utf8_lossy(&output.stdout).into_owned();

        git_log
            .split_terminator('\n')
            .map(|s| {
                let mut parts = s.split_whitespace();

                let lines_added: usize = parts.next().unwrap().parse::<usize>().unwrap_or(0);
                let lines_deleted: usize = parts.next().unwrap().parse::<usize>().unwrap_or(0);

                GitFileContributions {
                    lines_added,
                    lines_deleted,
                    lines_written: (lines_added as isize) - (lines_deleted as isize),
                }
            })
            .collect()
    } else {
        println!(
            "An error has occured while attempting to execute `git log` with author {}.",
            identity.email
        );
        vec![]
    }
}

// Returns a map of email -> (git identity, commits)
fn git_author_frequency() -> HashMap<String, (GitIdentity, usize)> {
    // git shortlog -sne --all --no-merges
    let mut cmd = Command::new("git");
    cmd.arg("shortlog");
    cmd.arg("--summary");
    cmd.arg("--numbered");
    cmd.arg("--email");
    cmd.arg("--no-merges");
    cmd.arg("--all");

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git log`");

    if output.status.success() {
        let git_shortlog = String::from_utf8_lossy(&output.stdout).into_owned();

        let author_contribution_freq_re =
            Regex::new(r"\s*(?P<freq>\d+)\s+(?P<author>.*)\s+<(?P<email>.*)>").unwrap();
        let mut author_contribution_frequency: HashMap<String, (GitIdentity, usize)> =
            HashMap::new();
        for line in git_shortlog.lines() {
            if author_contribution_freq_re.is_match(line) {
                if let Some(caps) = author_contribution_freq_re.captures(line) {
                    let freq = caps
                        .name("freq")
                        .unwrap()
                        .as_str()
                        .parse::<usize>()
                        .unwrap();

                    let author = caps.name("author").unwrap().as_str().to_string();
                    let email = caps.name("email").unwrap().as_str().to_string();

                    if let Some(p) = author_contribution_frequency.get_mut(&email) {
                        p.0.names.push(author);
                        p.1 += freq;
                    } else {
                        let identity = GitIdentity {
                            email: email.clone(),
                            names: vec![author],
                        };

                        author_contribution_frequency.insert(email, (identity, freq));
                    }
                } else {
                    println!(
                        "WARN: Unable to parse git frequency line \"{line}\": no matching captures for regex \"{author_contribution_freq_re:?}\""
                    );
                }
            } else {
                println!(
                    "WARN: Unable to parse git frequency line \"{line}\": no matching captures for regex \"{author_contribution_freq_re:?}\""
                );
            }
        }

        author_contribution_frequency
    } else {
        println!(
            "An error has occured.  It is likely that you aren't in a git repository, or you may not have `git` installed."
        );

        HashMap::new()
    }
}
