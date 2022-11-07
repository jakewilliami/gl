mod branch;
mod commitcount;
mod contributions;
mod languages;
mod log;
mod repo;
mod status;

extern crate clap;
use clap::{Arg, Command, crate_version};

// needed for log.rs
extern crate colored;
extern crate regex;

// needed for commitcount.rs
extern crate chrono;

// TODO list (delete help commands as I go)
// -i | --issues		Prints currently open issues in present repository.
// -t | --tags | --labels	Lists this repository's issues' tags/labels .
// -f | --filtered-issues	Prints filtered issues by tag.  By default, prints issues tagged with "enhancement" unless stated otherwise.
// -e | --exclude-issues	Prints issues excluding issues that are tagged with "depricated" and "pdfsearch" unless stated otherwise.
// Also, I have notes on github-linguist which I could add to this app, maybe under a `help` subcommand?

fn main() {
	let matches = Command::new("gl")
                            .version(crate_version!())
                            .author("Jake W. Ireland. <jakewilliami@icloud.com>")
                            .about("Git log and other personalised git utilities.  By default (i.e., without any arguments), it will print the last 10 commits nicely.")
							.arg(Arg::new("LOGNUMBER")
								// TODO: as well as -n we should also be able to do -10, -100, -3, etc
								// .short("n")
								// .long("number")
								// .value_name("FILE")
								.help("Given a number, will print the last n commits nicely.")
								// .index(1)
								// .default_value("10")
								.num_args(1)
								.required(false)
						   	)
							.arg(Arg::new("LANGUAGES")
								.short('l')
								.long("languages")
								.help("Prints language breakdown in present repository.  Will print only top n languages if given value (optional).")
								.num_args(0..=1)
								.required(false)
						   	)
							.arg(Arg::new("STATUS")
								.short('s')
								.long("status")
								.help("Prints current git status minimally.")
								.num_args(0..=1)
								.required(false)
						   	)
							.arg(Arg::new("GLOBAL")
								.short('g')
								.long("global")
								.help("Gets git status for any dirty repositories, defined from file")
								.num_args(0)
								.required(false)
						   	)
							.arg(Arg::new("BRANCH")
								.short('b')
								.long("branch")
								.help("Prints the current branch name")
								.num_args(0)
								.required(false)
						   	)
							.arg(Arg::new("BRANCHES")
								.short('B')
								.long("branches")
								.help("Prints all local branches in the current repository")
								.num_args(0)
								.required(false)
						   	)
							.arg(Arg::new("REMOTES")
								.short('R')
								.long("remotes")
								.help("Prints remote branches of the current repository")
								.num_args(0)
								.required(false)
						   	)
							.arg(Arg::new("REPO")
								.short('r')
								.long("repo")
								.help("Prints the name of the current repository")
								.num_args(0)
								.required(false)
						   	)
							.arg(Arg::new("COMMITCOUNT")
								.short('c')
								.long("commit-count")
								.help("Counts the current number of commits on working branch on the current day")
								.required(false)
								.conflicts_with("COMMITCOUNTWHEN")
						   	)
							.arg(Arg::new("COMMITCOUNTWHEN")
								.short('C')
								.long("commit-count-when")
								.help("Counts the number of commits for a specified day.  Takes values \"today\" (see -c), \"yesterday\", or some number of days ago.")
								// TODO:
								//   If you give it 2 numbers, it will show the number of commits since the first number but before the second number (days ago).
                                //   E.g., given 5, 2, it will get the number of commits since 5 days ago, but before 2 days ago.  Given 5 and 1, it will get the
								//   number of commits in the last 5 days ago, no including anything since yesterday.  This can be done by calculating commits_since(5)
								//   - commits_since(2), etc.  To do this I need to figure out how to use multiple arguments, otherwise I will have to create a separate
                                //   flag
								.num_args(1)
								.required(false)
								// .multiple_values(true)
								.conflicts_with("COMMITCOUNT")
						   	)
							.arg(Arg::new("AUTHORCOMMITCOUNTS")
								.short('A')
								.long("author-commit-counts")
								.help("Prints the number of commits per author")
								.num_args(0)
								.required(false)
						   	)
							.arg(Arg::new("AUTHORCONTRIBSTATS")
								.short('S')
								.long("author-contrib-stats")
								.help("Prints some contribution statistics given an author")
								.num_args(0)
								.required(false)
						   	)
							.get_matches();


	// show the git log
	if !matches.args_present() {
		let n: usize = 10;
		log::get_git_log(n);
		return;
	}
	if matches.get_flag("LOGNUMBER") {
		let n = matches.get_one::<usize>("LOGNUMBER")
			.unwrap_or(&0);
		if n != &0 {
			log::get_git_log(*n);
		}
	}

	// show languages
	if matches.get_flag("LANGUAGES") {
		// This parses _and_ prints the language output
		// languages::parse_language_data();
		let language_summary = languages::construct_language_summary();
		let top_n = matches.get_one::<usize>("LANGUAGES")
			.map(|n| n.to_owned())
			.unwrap_or(language_summary.len());
		languages::print_language_summary(top_n, language_summary);
	};

	// show status of git repo
	if matches.get_flag("STATUS") {
		let dir = matches.get_one::<String>("STATUS")
			.map(|n| n.to_string())
			.unwrap_or_else(|| "".to_string());
		let maybe_dir = if dir.is_empty() {
			None
		} else {
			Some(dir)
		};
		status::get_git_status(&maybe_dir);
	};

	// show statuses of predefined git repos
	if matches.get_flag("GLOBAL") {
		status::global_status();
	};

	// show branch name
	if matches.get_flag("BRANCH") {
		let current_branch = branch::current_branch();
		if current_branch.is_some() {
			println!("{}", current_branch.unwrap());
		}
	}

	// show branches
	if matches.get_flag("BRANCHES") {
		branch::get_branch_names(branch::BranchListings::Local);
	}

	// show the current repository
	if matches.get_flag("REPO") {
		let current_repo = repo::current_repository();
		if current_repo.is_some() {
			println!("{}", current_repo.unwrap());
		}
	}

	// show remote branches
	if matches.get_flag("REMOTES") {
		branch::get_branch_names(branch::BranchListings::Remotes);
	}

	// show commit count
	if matches.get_flag("COMMITCOUNT") {
		commitcount::get_commit_count("today");
	}

	if matches.get_flag("COMMITCOUNTWHEN") {
		let input_raw = matches.get_one::<&str>("COMMITCOUNTWHEN");

		let input = input_raw.unwrap();
		commitcount::get_commit_count(input);
	}

	// show number of commits per author, sorted by commit
	if matches.get_flag("AUTHORCOMMITCOUNTS") {
		let contributors = contributions::git_contributor_stats();
		contributions::display_git_author_frequency(contributors);
	}

	// show contribution stats per author, sorted by lines added + deleted
	if matches.get_flag("AUTHORCONTRIBSTATS") {
		let contributors = contributions::git_contributor_stats();
		contributions::display_git_contributions_per_author(contributors);
	}
}
