use clap::{crate_version, value_parser, ArgAction, Parser};

mod branch;
mod commit;
mod config;
mod contributions;
mod count;
mod identity;
mod languages;
mod log;
mod opts;
mod repo;
mod status;

// TODO list (delete help commands as I go)
// -i | --issues        Prints currently open issues in present repository.
// -t | --tags | --labels   Lists this repository's issues' tags/labels .
// -f | --filtered-issues   Prints filtered issues by tag.  By default, prints issues tagged with "enhancement" unless stated otherwise.
// -e | --exclude-issues    Prints issues excluding issues that are tagged with "depricated" and "pdfsearch" unless stated otherwise.
// Also, I have notes on github-linguist which I could add to this app, maybe under a `help` subcommand?
// Also consider using argument groups for things like contrib stats, status, commt counts, etc.

#[derive(Parser)]
#[command(
    name = "gl",
    author = "Jake·W.·Ireland.·<jakewilliami@icloud.com>",
    version = crate_version!(),
)]
/// Git log and other personalised git utilities.
///
/// By default (i.e., without any arguments), it will print the last 10 commits nicely.
struct Cli {
    /// Given a number, will print the last n commits nicely
    ///
    /// By default, the programme will print the last 10 commits.  Can use with --rev to show least recent logs first.  Can also use --all to show all logs
    #[arg(
        // TODO: as well as -n, we should also be able to do -10, -100, -3, etc
        action = ArgAction::Set,
        num_args = 1,
        value_parser = value_parser!(usize),
        value_name = "n commits",
        // default_missing_value = "10",
    )]
    log_number: Option<usize>,

    /// Prints language breakdown in present repository
    ///
    /// Will print only top n languages if given value (optional).  Defaults to displaying all languages (you can also specify n = 0 for this behaviour)
    #[arg(
        short = 'l',
        long = "languages",
        action = ArgAction::Set,
        num_args = 0..=1,
        value_parser = value_parser!(usize),
        value_name = "n languages",
        default_missing_value = "0",  // TODO: consider making this an isize, and using allow_negative_numbers
    )]
    languages: Option<usize>,

    /// Prints current git status minimally
    ///
    /// Defaults to the current directory, but you can specify a directory
    #[arg(
        short = 's',
        long = "status",
        action = ArgAction::Set,
        num_args = 0..=1,
        value_name = "dir",
        default_missing_value = "",
    )]
    status: Option<String>,

    /// Gets git status for any dirty repositories, defined from file (WIP)
    #[arg(
        short = 'g',
        long = "global",
        action = ArgAction::SetTrue,
        num_args = 0,
    )]
    global_status: Option<bool>,

    /// Prints the current branch name
    #[arg(
        short = 'b',
        long = "branch",
        action = ArgAction::SetTrue,
        num_args = 0,
    )]
    branch: Option<bool>,

    /// Prints all local branches in the current repository
    #[arg(
        short = 'B',
        long = "branches",
        action = ArgAction::SetTrue,
        num_args = 0,
    )]
    local_branches: Option<bool>,

    /// Print all remote branches of the current repository
    #[arg(
        short = 'R',
        long = "remotes",
        action = ArgAction::SetTrue,
        num_args = 0,
    )]
    remote_branches: Option<bool>,

    /// Prints the name of the current repository
    #[arg(
        short = 'r',
        long = "repo",
        action = ArgAction::SetTrue,
        num_args = 0,
    )]
    repo_name: Option<bool>,

    /// Counts the current number of commits on working branch on the current day
    #[arg(
        short = 'c',
        long = "commit-count",
        action = ArgAction::SetTrue,
        num_args = 0,
        conflicts_with = "commit_count_when",
    )]
    commit_count: Option<bool>,

    /// Counts the number of commits for a specified day, or all time
    ///
    /// Given value "today" (see also -c), "yesterday", or some number of days ago.  If no value is given, it will default to all time (you can also specify C = total for this behaviour)
    #[arg(
        // TODO:
        //   If you give it 2 numbers, it will show the number of commits since the first number but before the second number (days ago).
        //   E.g., given 5, 2, it will get the number of commits since 5 days ago, but before 2 days ago.  Given 5 and 1, it will get the
        //   number of commits in the last 5 days ago, no including anything since yesterday.  This can be done by calculating commits_since(5)
        //   - commits_since(2), etc.  To do this I need to figure out how to use multiple arguments, otherwise I will have to create a separate
        //   flag
        short = 'C',
        long = "commit-count-when",  // TODO: rename to "commit_count_at"; will need to update minor version (breaking change)
        action = ArgAction::Set,
        num_args = 0..=1,
        value_name = "relative day quantifier",
        conflicts_with = "commit_count",
        default_missing_value = "total",
    )]
    commit_count_when: Option<String>,

    /// Displays the number of commits per author
    #[arg(
        short = 'A',
        long = "author-commit-counts",  // TODO: rename to commit-count-authors; will need to update minor version (breaking change)
        action = ArgAction::SetTrue,
        num_args = 0,
    )]
    author_commit_counts: Option<bool>,

    /// Displays some contribution statistics given an author
    #[arg(
        short = 'S',
        long = "author-contrib-stats",
        action = ArgAction::SetTrue,
        num_args = 0,
    )]
    author_contrib_stats: Option<bool>,

    /// Display overall contribution statistics as a graph
    #[arg(
        short = 'G',
        long = "contrib-graph",
        action = ArgAction::SetTrue,
        num_args = 0,
    )]
    contrib_graph: Option<bool>,

    /// Display git log with absolute commit dates
    #[arg(
        short = 'a',
        long = "abs",
        action = ArgAction::SetTrue,
        num_args = 0,
    )]
    absolute: Option<bool>,

    /// Display the *least* recent logs (reverse order)
    #[arg(
        long = "rev",
        action = ArgAction::SetTrue,
        num_args = 0,
    )]
    reverse: Option<bool>,

    /// Display all logs
    #[arg(
        long = "all",
        action = ArgAction::SetTrue,
        num_args = 0,
        conflicts_with = "log_number",
    )]
    all: Option<bool>,

    /// Display count of commits
    ///
    /// See also -C/--commit-count-when
    #[arg(
        long = "count",
        action = ArgAction::SetTrue,
        num_args = 0,
        conflicts_with = "commit_count_when",
    )]
    count: Option<bool>,
}

fn main() {
    let cli = Cli::parse();
    let opts = opts::GitLogOptions {
        relative: !cli.absolute.unwrap_or(true),
        // https://no-color.org/
        colour: !(std::env::var("NO_COLOR").is_ok() || std::env::var("NO_COLOUR").is_ok()),
        reverse: cli.reverse.unwrap_or(false),
        all: cli.all.unwrap_or(false),
    };

    // We need to handle the default case by setting a logical to check if
    // the user should expect the default behaviour
    let mut non_default_option = false;

    // show languages
    if let Some(n) = cli.languages {
        non_default_option = true;
        // This parses _and_ prints the language output
        // languages::parse_language_data();
        let language_summary = languages::construct_language_summary();
        let top_n = if n == 0 { language_summary.len() } else { n };
        languages::print_language_summary(top_n, language_summary, &opts);
    };

    // show status of git repo
    if let Some(dir) = cli.status {
        non_default_option = true;
        let maybe_dir = if dir.is_empty() { None } else { Some(dir) };
        status::get_git_status(&maybe_dir, &opts);
    };

    // show statuses of predefined git repos
    if let Some(global_status) = cli.global_status {
        if global_status {
            non_default_option = true;
            status::global_status(&opts);
        }
    };

    // show branch name
    if let Some(show_branch) = cli.branch {
        if show_branch {
            non_default_option = true;
            let current_branch = branch::current_branch();
            if let Some(current_branch) = current_branch {
                println!("{}", current_branch);
            }
        }
    }

    // show branches
    if let Some(show_local_branches) = cli.local_branches {
        if show_local_branches {
            non_default_option = true;
            branch::get_branch_names(branch::BranchListings::Local, &opts);
        }
    }

    // show remote branches
    if let Some(show_remote_branches) = cli.remote_branches {
        if show_remote_branches {
            non_default_option = true;
            branch::get_branch_names(branch::BranchListings::Remotes, &opts);
        }
    }

    // show the current repository
    if let Some(show_repo_name) = cli.repo_name {
        if show_repo_name {
            non_default_option = true;
            let current_repo = repo::current_repository();
            if let Some(current_repo) = current_repo {
                println!("{}", current_repo);
            }
        }
    }

    // show commit count
    if let Some(show_commit_count) = cli.commit_count {
        if show_commit_count {
            non_default_option = true;
            count::get_commit_count("today", &opts);
        }
    }

    if let Some(count) = cli.count {
        // Equivalent to -C without arguments (i.e., commit_count_when = total)
        if count {
            non_default_option = true;
            count::get_commit_count_total(&opts);
        }
    }

    if let Some(commit_count_when) = cli.commit_count_when {
        non_default_option = true;
        if commit_count_when == "total" {
            count::get_commit_count_total(&opts);
        } else {
            count::get_commit_count(&commit_count_when, &opts);
        }
    }

    // Calculate contribution stats
    let show_author_commit_counts = cli.author_commit_counts.unwrap_or(false);
    let show_author_contrib_stats = cli.author_contrib_stats.unwrap_or(false);
    let show_contrib_graph = cli.contrib_graph.unwrap_or(false);
    let contributors =
        if show_author_commit_counts || show_author_contrib_stats || show_contrib_graph {
            Some(contributions::git_contributors())
        } else {
            None
        };
    if let Some(contributors) = contributors {
        non_default_option = true;
        // show number of commits per author, sorted by commit
        if show_author_commit_counts {
            contributions::display_git_author_frequency(contributors.clone());
        }
        // show contribution stats per author, sorted by lines added + deleted
        if show_author_contrib_stats {
            contributions::display_git_contributions_per_author(contributors.clone());
        }
        // Show contributions graph
        if show_contrib_graph {
            contributions::display_git_contributions_graph(contributors.clone());
        }
    }

    // Display log (default or "base" behaviour)
    if let Some(n) = cli.log_number {
        log::display_git_log(n, &opts);
    } else if !non_default_option {
        log::display_git_log(config::DEFAULT_TOP_N_LOG, &opts);
    }
}
