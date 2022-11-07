use std::collections::HashMap;
use std::process::{Command, Stdio};

use regex::Regex;
use tabular::{Table, row};

// Types

#[derive(Clone)]
pub struct GitContributor {
	id: GitIdentity,
	commits: usize,
	file_contributions: Vec<GitFileContribution>,
}

#[derive(Clone)]
struct GitIdentity {
	email: String,
	author_names: Vec<String>,
}

#[derive(Clone)]
struct GitFileContribution {
	lines_added: usize,
	lines_deleted: usize,
	lines_written: isize,
}

struct ContributionSummary {
	lines_added: usize,
	lines_deleted: usize,
	lines_written: isize,
}

// Traits/implementations

trait ContributorStats {
	fn contribution_summary(&self) -> ContributionSummary;
}

impl ContributorStats for GitContributor {
	fn contribution_summary(&self) -> ContributionSummary {
		let mut lines_added: usize = 0;
		let mut lines_deleted: usize = 0;
		let mut lines_written: isize = 0;

		for contribution in &self.file_contributions {
			lines_added += contribution.lines_added;
			lines_deleted += contribution.lines_deleted;
			lines_written += contribution.lines_written;
		}

		ContributionSummary{
			lines_added,
			lines_deleted,
			lines_written,
		}
	}
}

// Display methods

pub fn display_git_contributions_per_author(contributors: Vec<GitContributor>) {
	let mut contributors_with_summary: Vec<(GitContributor, ContributionSummary)> = Vec::new();
	for contributor in contributors {
		let contrib_summary = contributor.contribution_summary();
		contributors_with_summary.push((contributor, contrib_summary));
	}
	// Sort by sum of lines added and deleted (in reverse order)
	contributors_with_summary.sort_by(|a, b| {
		(b.1.lines_added + b.1.lines_deleted).cmp(&(a.1.lines_added + a.1.lines_deleted))
	});

	let mut table = Table::new("{:<}  {:>}  {:>}  {:>}")
		.with_row(row!("Author", "Lines added", "Lines deleted", "Lines of code"));

	for (contributor, contrib_summary) in contributors_with_summary {
		table.add_row(row!(
			contributor.id.email,
			contrib_summary.lines_added,
			contrib_summary.lines_deleted,
			contrib_summary.lines_written,
		));
	}
	println!("{}", table);
}

pub fn display_git_author_frequency(contributors: Vec<GitContributor>) {
	let mut contributors_sorted = contributors;
	// Sort by commits (in reverse order)
	contributors_sorted.sort_by(|a, b| b.commits.cmp(&a.commits));

	let mut table = Table::new("{:<}  {:>}")
		.with_row(row!("Author", "Commits"));

	for contributor in contributors_sorted {
		table.add_row(row!(contributor.id.email, contributor.commits));
	}

	println!("{}", table);
}

// Constructor methods

pub fn git_contributor_stats() -> Vec<GitContributor> {
	let author_frequency = git_author_frequency();
	let mut contrib_stats: Vec<GitContributor> = Vec::new();
	for (_email, (identity, freq)) in author_frequency {
		let contributor = git_contributions_per_author(identity, freq);

		contrib_stats.push(contributor);
	}

	contrib_stats
}

fn git_contributions_per_author(identity: GitIdentity, freq: usize) -> GitContributor {
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
		let git_log = String::from_utf8_lossy(&output.stdout)
			.into_owned();

		let file_contributions: Vec<GitFileContribution> = git_log.split_terminator('\n')
		    .into_iter()
		    .map(|s| {
			    let mut parts = s.split_whitespace();

				let lines_added: usize = parts.next().unwrap().parse::<usize>().unwrap_or(0);
				let lines_deleted: usize = parts.next().unwrap().parse::<usize>().unwrap_or(0);

				GitFileContribution{
					lines_added,
					lines_deleted,
					lines_written: (lines_added as isize) - (lines_deleted as isize),
				}
			})
		    .collect();

		GitContributor{
			id: identity,
			commits: freq,
			file_contributions,
		}
	} else {
		println!("An error has occured while attempting to execute `git log` with author {}.", identity.email);

		GitContributor{
			id: identity,
			commits: freq,
			file_contributions: vec![],
		}
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
		let git_shortlog = String::from_utf8_lossy(&output.stdout)
			.into_owned();

		let author_contribution_freq_re = Regex::new(r"\s*(?P<freq>\d+)\s+(?P<author>.*)\s+<(?P<email>.*)>").unwrap();
		let mut author_contribution_frequency: HashMap<String, (GitIdentity, usize)> = HashMap::new();
		for line in git_shortlog.lines() {
			if author_contribution_freq_re.is_match(line) {
				if let Some(caps) = author_contribution_freq_re.captures(line) {
					let freq = caps.name("freq")
				        .unwrap()
				        .as_str()
				        .parse::<usize>()
				        .unwrap();

					let author = caps.name("author").unwrap().as_str().to_string();
					let email = caps.name("email").unwrap().as_str().to_string();

					if let Some(p) = author_contribution_frequency.get_mut(&email) {
						p.0.author_names.push(author);
						p.1 += freq;
					} else {
						let identity = GitIdentity {
							email: email.clone(),
							author_names: vec![author],
						};

						author_contribution_frequency.insert(email, (identity, freq));
					}
				} else {
					println!("WARN: Unable to parse git frequency line \"{}\": no matching captures for regex \"{:?}\"", line, author_contribution_freq_re);
				}
			} else {
				println!("WARN: Unable to parse git frequency line \"{}\": no matching captures for regex \"{:?}\"", line, author_contribution_freq_re);
			}
		}

		author_contribution_frequency
	} else {
		println!("An error has occured.  It is likely that you aren't in a git repository, or you may not have `git` installed.");

		HashMap::new()
	}
}
