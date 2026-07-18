use crate::{config, dates, opts::TagFormat};
use chrono::NaiveDate;
use clap::{ArgAction, Args, Parser, Subcommand, crate_authors, crate_name, crate_version};

// TODO list (delete help commands as I go)
// -i | --issues        Prints currently open issues in present repository.
// -f | --filtered-issues   Prints filtered issues by tag.  By default, prints issues tagged with "enhancement" unless stated otherwise.
// -e | --exclude-issues    Prints issues excluding issues that are tagged with "depricated" and "pdfsearch" unless stated otherwise.
// Also, I have notes on github-linguist which I could add to this app, maybe under a `help` subcommand?
// Also consider using argument groups for things like contrib stats, status, commt counts, etc.

#[derive(Parser)]
#[command(
    name = crate_name!(),
    author = crate_authors!(", "),
    version = crate_version!(),
)]
/// Git log and other personalised git utilities.
///
/// By default (i.e., without any arguments), it will print the last 10 commits nicely.
pub struct Cli {
    /// Display the *least* recent logs (reverse order)
    #[arg(
        long = "rev",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    pub reverse: bool,

    #[clap(flatten)]
    pub log: LogGroup,

    #[clap(flatten)]
    pub dispatch: DispatchGroup,
}

#[derive(Args)]
pub struct LogGroup {
    /// Given a number, will print the last n commits nicely
    ///
    /// By default, the programme will print the last 10 commits.  Can use with --rev to show least recent logs first.  Can also use --all to show all logs
    #[arg(
        // TODO: as well as -n, we should also be able to do -10, -100, -3, etc
        id = "log_count",
        action = ArgAction::Set,
        num_args = 1,
        value_name = "n commits",
        default_value_t = config::DEFAULT_TOP_N_LOG,
        conflicts_with = "dispatch",
    )]
    pub count: usize,

    /// Display git log with absolute commit dates
    #[arg(
        short = 'a',
        long = "abs",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
        conflicts_with = "dispatch",
    )]
    pub absolute: bool,

    /// Display all logs
    #[arg(
        long = "all",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
        conflicts_with = "dispatch",
    )]
    pub all: bool,

    /// Filter log for specified commit author(s)
    #[arg(
        long = "author",
        action = ArgAction::Append,
        num_args = 1..=std::usize::MAX,
        conflicts_with = "dispatch",
    )]
    pub authors: Vec<String>,

    /// Filter log for commit messages matching text
    #[arg(
        long = "grep",
        action = ArgAction::Append,
        num_args = 1..=std::usize::MAX,
        conflicts_with = "dispatch",
    )]
    pub grep: Vec<String>,
}

// This group allows us to switch between different functionalities.  The default
// functionality of this CLI is of course printing the recent git commits (git log),
// but we can change the functional output by specifying alternative flags, listed
// below.
//
// We only want to allow one functional check at a time.  The following group,
// which is flattened in the main Cli struct, should provide such functionality
//
//   https://stackoverflow.com/a/76315811
#[derive(Args)]
#[group(id = "dispatch", multiple = false)]
pub struct DispatchGroup {
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
    pub languages: Option<usize>,

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
    pub status: Option<String>,

    /// Prints the current branch name
    #[arg(
        short = 'b',
        long = "branch",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    pub branch: bool,

    /// Prints all local branches in the current repository
    #[arg(
        short = 'B',
        long = "branches",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    pub local_branches: bool,

    /// Print all remote branches of the current repository
    #[arg(
        short = 'R',
        long = "remotes",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    pub remote_branches: bool,

    /// Print the remote origin URL
    #[arg(
        short = 'o',
        long = "origin",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    pub remote_origin: bool,

    /// Prints the name of the current repository
    #[arg(
        short = 'r',
        long = "repo",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    pub repo_name: bool,

    /// Counts the current number of commits on working branch on the current day
    #[arg(
        short = 'c',
        long = "commit-count",
        action = ArgAction::SetTrue,
        num_args = 0,
        conflicts_with = "commit_count_at",
        default_value_t = false,
    )]
    pub commit_count: bool,

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
    pub commit_count_at: Option<String>,

    /// Displays the number of commits per author
    #[arg(
        short = 'A',
        long = "author-commit-counts",  // TODO: rename to commit-count-authors; will need to update minor version (breaking change)
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    pub author_commit_counts: bool,

    /// Displays some contribution statistics given an author
    #[arg(
        short = 'S',
        long = "author-contrib-stats",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    pub author_contrib_stats: bool,

    /// Display overall contribution statistics as a graph
    #[arg(
        short = 'G',
        long = "contrib-graph",
        action = ArgAction::SetTrue,
        num_args = 0,
        default_value_t = false,
    )]
    pub contrib_graph: bool,

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
    pub count: bool,

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
    pub date: Option<NaiveDate>,

    // TODO: add option to not parse tags as versions?
    // TODO: provide option to create tag based on latest commit message
    // TODO: provide option to bump minor or major by default also?  (current default is to
    //    bump patch version)
    /// List tags
    ///
    /// List all of the tags in the git repository.  By default, the long tag format is used.
    #[arg(
        short = 't',
        long = "tags",
        value_name = "format",
        default_missing_value = "long",
        num_args = 0..=1,
    )]
    pub tags: Option<TagFormat>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Tag and push a new version
    ///
    /// Ask for tag and annotation message.  Creates the tag locally and pushes it to origin.
    /// Note that this assumes tags are semantic versions.
    Tag,
}
