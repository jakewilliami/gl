mod branch;
mod cli;
mod commit;
mod config;
mod contributions;
mod count;
mod date; // TODO: do we want to merge these files?
mod dates;
mod env;
mod hash;
mod identity;
mod languages;
mod log;
mod opts;
mod origin;
mod repo;
mod status;
mod tag;
mod version;

use crate::cli::{Cli, Commands};
use clap::Parser;
use std::process;

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

    // Handle subcommand and exit
    match cli.group.command {
        Some(Commands::Tag) => {
            tag::tag();
            process::exit(0);
        }
        None => {}
    }

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
        status::display_git_status(&cli.group.status, &opts);
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
        branch::display_branches(branch::BranchListings::Local, &opts);
    } else if cli.group.remote_branches {
        // Show remote branches
        branch::display_branches(branch::BranchListings::Remotes, &opts);
    } else if cli.group.remote_origin {
        // Show remote origin URL
        origin::get_remote_origin_url();
    } else if cli.group.repo_name {
        // Show the current repository
        let current_repo = repo::current_repository();
        if let Some(current_repo) = current_repo {
            println!("{current_repo}");
        }
    } else if let Some(fmt) = cli.group.tags {
        tag::get_tags(fmt);
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
