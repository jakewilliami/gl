use chrono::NaiveDate;
use clap::{ArgAction, Args, Parser, crate_version};

mod branch;
mod commit;
mod config;
mod contributions;
mod count;
mod dates;
mod env;
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
    /// Display git log with absolute commit dates
    #[arg(
        short = 'a',
        long = "abs",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    absolute: bool,

    /// Display the *least* recent logs (reverse order)
    #[arg(
        long = "rev",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    reverse: bool,

    /// Display all logs
    #[arg(
        long = "all",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
        conflicts_with = "log_number",
    )]
    all: bool,

    /// Filter log for specified commit author(s)
    #[arg(
        long = "author",
        action = ArgAction::Append,
        num_args = 1..=std::usize::MAX,
    )]
    authors: Vec<String>,

    /// Filter log for commit messages matching text
    #[arg(
        long = "grep",
        action = ArgAction::Append,
        num_args = 1..=std::usize::MAX,
    )]
    grep: Vec<String>,

    #[clap(flatten)]
    group: Group,
}

// We only want to allow one functional check at a time.  The following group,
// which is flattened in the main Cli struct, should provide such functionality
//
//   https://stackoverflow.com/a/76315811
#[derive(Args)]
#[group(multiple = false)]
pub struct Group {
    /// Given a number, will print the last n commits nicely
    ///
    /// By default, the programme will print the last 10 commits.  Can use with --rev to show least recent logs first.  Can also use --all to show all logs
    #[arg(
        // TODO: as well as -n, we should also be able to do -10, -100, -3, etc
        action = ArgAction::Set,
        num_args = 1,
        value_name = "n commits",
        default_value_t = config::DEFAULT_TOP_N_LOG,
    )]
    log_number: usize,

    /// Prints language breakdown in present repository
    ///
    /// Will print only top n languages if given value (optional).  Defaults to displaying all languages (you can also specify n = 0 for this behaviour)
    #[arg(
        short = 'l',
        long = "languages",
        action = ArgAction::Set,
        num_args = 0..=1,
        value_name = "n languages",
        default_missing_value = "0",
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
        default_missing_value = ".",
    )]
    status: Option<String>,

    /// Prints the current branch name
    #[arg(
        short = 'b',
        long = "branch",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    branch: bool,

    /// Prints all local branches in the current repository
    #[arg(
        short = 'B',
        long = "branches",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    local_branches: bool,

    /// Print all remote branches of the current repository
    #[arg(
        short = 'R',
        long = "remotes",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    remote_branches: bool,

    /// Prints the name of the current repository
    #[arg(
        short = 'r',
        long = "repo",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    repo_name: bool,

    /// Counts the current number of commits on working branch on the current day
    #[arg(
        short = 'c',
        long = "commit-count",
        action = ArgAction::SetTrue,
        num_args = 0,
        conflicts_with = "commit_count_at",
        default_value_t = false,
    )]
    commit_count: bool,

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
        long = "commit-count-at",
        action = ArgAction::Set,
        num_args = 0..=1,
        value_name = "relative day quantifier",
        conflicts_with = "commit_count",
        default_missing_value = "total",
    )]
    commit_count_at: Option<String>,

    /// Displays the number of commits per author
    #[arg(
        short = 'A',
        long = "author-commit-counts",  // TODO: rename to commit-count-authors; will need to update minor version (breaking change)
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    author_commit_counts: bool,

    /// Displays some contribution statistics given an author
    #[arg(
        short = 'S',
        long = "author-contrib-stats",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    author_contrib_stats: bool,

    /// Display overall contribution statistics as a graph
    #[arg(
        short = 'G',
        long = "contrib-graph",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    contrib_graph: bool,

    /// Display count of commits
    ///
    /// See also -C/--commit-count-at
    #[arg(
        long = "count",
        action = ArgAction::SetTrue,
        num_args = 0,
        conflicts_with = "commit_count_at",
        default_value_t = false,
    )]
    count: bool,

    /// Find commit ref at date
    ///
    /// Given a date, will search through the repository to find the commit ref at that date.
    #[arg(
        short = 'd',
        long = "date",
        action = ArgAction::Set,
        num_args = 1,
        value_name = "date (yyyy-mm-dd)",
        value_parser = dates::parse_date,
    )]
    date: Option<NaiveDate>,
}

fn main() {
    let cli = Cli::parse();
    let opts = opts::GitLogOptions {
        relative: !cli.absolute,
        colour: env::colour(),
        reverse: cli.reverse,
        all: cli.all,

        // Filters
        authors: cli.authors,
        needles: cli.grep,
    };

    // Because all of these options are in a group, at most one branch should
    // ever be matched, so it is safe to put this in an if-else chain
    if let Some(n) = cli.group.languages {
        // This parses _and_ prints the language output
        let language_summary = languages::construct_language_summary();
        // If no argument was provided, it will print all languages
        let top_n = if n == 0 { language_summary.len() } else { n };
        languages::print_language_summary(top_n, language_summary, &opts);
    } else if cli.group.status.is_some() {
        // Show status of git repo
        status::get_git_status(&cli.group.status, &opts);
    // } else if cli.group.global_status {
    //     // Show statuses of predefined git repos (not yet implemented)
    //     todo!()
    //     // status::global_status(&opts);
    } else if cli.group.branch {
        // Show current branch name
        let current_branch = branch::current_branch();
        if let Some(current_branch) = current_branch {
            println!("{current_branch}");
        }
    } else if cli.group.local_branches {
        // Show local branches
        branch::get_branch_names(branch::BranchListings::Local, &opts);
    } else if cli.group.remote_branches {
        // Show remote branches
        branch::get_branch_names(branch::BranchListings::Remotes, &opts);
    } else if cli.group.repo_name {
        // Show the current repository
        let current_repo = repo::current_repository();
        if let Some(current_repo) = current_repo {
            println!("{current_repo}");
        }
    } else if cli.group.commit_count {
        // Show commit count
        count::get_commit_count("today", &opts);
    } else if cli.group.count {
        // Equivalent to -C without arguments (i.e., commit_count_at = total)
        count::get_commit_count_total(&opts);
    } else if let Some(commit_count_at) = cli.group.commit_count_at {
        // Show commit count for a  specific time
        if commit_count_at == "total" {
            count::get_commit_count_total(&opts);
        } else {
            count::get_commit_count(&commit_count_at, &opts);
        }
    } else if cli.group.author_commit_counts
        || cli.group.author_contrib_stats
        || cli.group.contrib_graph
    {
        // Handle different contributor stats options
        let contributors = contributions::git_contributors();
        if cli.group.author_commit_counts {
            contributions::display_git_author_frequency(contributors.clone());
        } else if cli.group.author_contrib_stats {
            // Show contribution stats per author, sorted by lines added + deleted
            contributions::display_git_contributions_per_author(contributors.clone());
        } else if cli.group.contrib_graph {
            // Show contributions graph
            contributions::display_git_contributions_graph(contributors.clone());
        }
    } else if let Some(date) = cli.group.date {
        dates::find_first_commit_before_date(date);
    } else {
        log::display_git_log(cli.group.log_number, &opts);
    }
}
