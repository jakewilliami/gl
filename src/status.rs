use super::{opts::GitLogOptions, repo::discover_repository};
use colored::*;
use gix::{
    Repository,
    dir::entry::Status,
    remote::Direction,
    status::{
        index_worktree::iter::Item::{self as StatusItem},
        plumbing::index_as_worktree::EntryStatus,
    },
    submodule::Status as SubmoduleStatus,
};
use std::{cmp::Ordering, ffi::OsString, path::PathBuf};

enum GitChange {
    Modification {
        relative_path: String,
        status: EntryStatus<(), SubmoduleStatus>,
    },
    DirectoryContents {
        relative_path: String,
        status: Status,
    },
    // Rewrite {},
}

// We want to order certain change variants before others
trait ChangePriority {
    fn priority(&self) -> u8;
}

impl ChangePriority for GitChange {
    fn priority(&self) -> u8 {
        match self {
            GitChange::Modification { .. } => 1,
            // GitChange::Rename { .. } => 2,
            GitChange::DirectoryContents { .. } => 3,
        }
    }
}

// Helper trait to extract a sortable key for status as statuses are public structs in gix
trait StatusKey {
    fn status_key(&self) -> u8;
}

impl StatusKey for GitChange {
    fn status_key(&self) -> u8 {
        match self {
            GitChange::Modification { status, .. } => match status {
                EntryStatus::Change(_) => 0,
                EntryStatus::Conflict(_) => 1,
                EntryStatus::NeedsUpdate(_) => 2,
                EntryStatus::IntentToAdd => 3,
            },
            GitChange::DirectoryContents { status, .. } => match status {
                Status::Untracked => 0,
                Status::Tracked => 1,
                Status::Pruned => 2,
                Status::Ignored(_) => 3,
            },
        }
    }
}

// We want to extract the relevant path for
trait ChangePath {
    fn path(&self) -> &str;
}

impl ChangePath for GitChange {
    fn path(&self) -> &str {
        match self {
            GitChange::Modification { relative_path, .. } => relative_path.as_ref(),
            GitChange::DirectoryContents { relative_path, .. } => relative_path.as_ref(),
        }
    }
}

impl Ord for GitChange {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare with the priority of the variant (e.g., modifications before
        // directory content), then by status, and finally by relevant path
        self.priority()
            .cmp(&other.priority())
            .then_with(|| self.status_key().cmp(&other.status_key()))
            .then_with(|| self.path().cmp(other.path()))
    }
}

impl PartialOrd for GitChange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for GitChange {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for GitChange {}

impl From<StatusItem> for GitChange {
    fn from(item: StatusItem) -> Self {
        match item {
            // Most likely, a file was edited
            StatusItem::Modification {
                rela_path, status, ..
            } => Self::Modification {
                relative_path: rela_path.to_string(),
                status,
            },

            // Likely a new file
            StatusItem::DirectoryContents { entry, .. } => Self::DirectoryContents {
                relative_path: entry.rela_path.to_string(),
                status: entry.status,
            },

            // Likely, a file was renamed (and optionally edited)
            // TODO: Instead of panicing I should warn and exit
            StatusItem::Rewrite { .. } => todo!(),
        }
    }
}

trait ChangeCodeShort {
    fn code(&self) -> ColoredString;
}

impl<T, U> ChangeCodeShort for EntryStatus<T, U> {
    fn code(&self) -> ColoredString {
        match self {
            EntryStatus::Change(_) => format!("{:>2}", "M").red(),
            EntryStatus::Conflict(_) => format!("{:>2}", "UU").red(),
            // TODO: I think somewhere here is an M variant that's green and left aligned, after resolving merge conflict
            // TODO: Instead of panicing I should warn and exit
            EntryStatus::NeedsUpdate(_) => todo!(),
            EntryStatus::IntentToAdd => format!("{:>2}", "A").red(),
        }
    }
}

impl ChangeCodeShort for Status {
    fn code(&self) -> ColoredString {
        match self {
            Status::Untracked => format!("{:<2}", "??").red(),
            Status::Tracked => format!("{:<2}", "A").green(),
            Status::Pruned => format!("{:<2}", "D").green(),
            Status::Ignored(_) => format!("{:<2}", "!!").red(),
        }
    }
}

trait FormatChange {
    fn display(&self) -> String;
}

impl FormatChange for GitChange {
    fn display(&self) -> String {
        match self {
            Self::Modification {
                relative_path,
                status,
            } => format!("{} {relative_path}", status.code()),
            Self::DirectoryContents {
                relative_path,
                status,
            } => format!("{} {relative_path}", status.code()),
        }
    }
}

pub fn display_git_status(dir: &Option<String>, opts: &GitLogOptions) {
    let given_dir: PathBuf = if (dir).is_none() {
        std::env::current_dir().unwrap()
    } else {
        PathBuf::from(dir.clone().unwrap())
    };

    let repo = discover_repository().unwrap();
    if let Some(header) = branch_header(&repo) {
        println!("{header}");
    }

    let mut changes = git_status(&repo, &given_dir.into_os_string(), opts);
    changes.sort();
    for change in changes {
        println!("{}", change.display());
    }
}

fn branch_header(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;
    let branch_name = head.referent_name().map(|name| name.shorten().to_string());

    // TODO: show `ahead/behind n`
    if let Some(branch_name) = branch_name {
        // Option 1: we are checked out to a branch
        let mut header = format!("## {}", branch_name.green());

        let remote_name = {
            let referent = head.try_into_referent()?;
            let remote = referent.remote_tracking_ref_name(Direction::Push)?.ok()?;
            Some(remote.shorten().to_string())
        };

        // Option 1.a: we also have a remote defined that we can specify
        if let Some(remote_name) = remote_name {
            header = format!("{}...{}", header, remote_name.red());
        }

        Some(header)
    } else {
        // Option 2: we are not checked out to a branch
        let head_name = head.name().as_bstr().to_string();
        Some(format!("## {} {}", head_name.red(), "(no branch)".red()))
    }
}

fn git_status(repo: &Repository, _dir: &OsString, _opts: &GitLogOptions) -> Vec<GitChange> {
    // TODO: report on current branch vs remote (push) branch
    // TODO: Handle in-rebase or merge things (InProgress)?
    // TODO: handle status
    let stat = repo.status(gix::progress::Discard);

    // In comparing the current index to the worktree status, we compare files stages for commit (index) and changes made but not yet staged (worktree), which effectively shows the changes to the status.  Comparing head to index is not yet implemented, meaning newly added files staged for commit are not here:
    //   https://github.com/GitoxideLabs/gitoxide/discussions/1680
    stat.unwrap()
        // TODO: I can filter dir in here
        .into_index_worktree_iter([])
        .unwrap()
        .flatten()
        .map(GitChange::from)
        .collect()
}
