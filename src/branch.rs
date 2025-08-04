use super::{opts::GitLogOptions, repo::discover_repository};
use colored::*;
use gix::{Repository, refs::TargetRef::*};

pub enum BranchListings {
    Local,
    Remotes,
}

struct LocalBranch {
    name: String,
    current_branch: bool,
}

struct RemoteBranch {
    name: String,
    target: Option<String>,
}

trait DisplayBranch {
    fn pretty(&self) -> String;
}

impl DisplayBranch for LocalBranch {
    fn pretty(&self) -> String {
        if self.current_branch {
            format!("{} {}", "*".bold(), self.name.green())
        } else {
            format!("  {}", self.name)
        }
    }
}

impl DisplayBranch for RemoteBranch {
    fn pretty(&self) -> String {
        let mut buf = format!("  {}", self.name.red());
        if let Some(target) = &self.target {
            buf.push_str(" -> ");
            buf.push_str(target);
        }
        buf
    }
}

// TODO: surely there is a better way; see citati for advanced generics; also xmemhash digest; also std::any::TypeId
fn get_branches(bt: BranchListings, _opts: &GitLogOptions) -> Vec<Box<dyn DisplayBranch>> {
    match bt {
        BranchListings::Local => local_branches()
            .into_iter()
            .map(|b| Box::new(b) as Box<dyn DisplayBranch>)
            .collect(),
        BranchListings::Remotes => remote_branches()
            .into_iter()
            .map(|b| Box::new(b) as Box<dyn DisplayBranch>)
            .collect(),
    }
}

pub fn display_branches(bt: BranchListings, opts: &GitLogOptions) {
    let branches = get_branches(bt, opts);

    for branch in branches.iter() {
        println!("{}", branch.pretty());
    }
}

fn current_branch_name_from_repo(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;
    head.referent_name().map(|name| name.shorten().to_string())
}

pub fn current_branch() -> Option<String> {
    // TODO: we should probably give a good error message here
    let repo = discover_repository().unwrap();
    current_branch_name_from_repo(&repo)
}

fn local_branches() -> Vec<LocalBranch> {
    let repo = discover_repository().unwrap();
    let current_branch = current_branch_name_from_repo(&repo);

    repo.references()
        .unwrap()
        .local_branches()
        .unwrap()
        .flatten()
        .map(|branch| {
            let branch_name = branch.name().shorten();
            LocalBranch {
                name: branch_name.to_string(),
                current_branch: current_branch
                    .as_ref()
                    .map(|current_branch| current_branch == branch_name)
                    .unwrap_or(false),
            }
        })
        .collect()
}

// all remote branches that your local Git repository is aware of
fn remote_branches() -> Vec<RemoteBranch> {
    let repo = discover_repository().unwrap();

    repo.references()
        .unwrap()
        .remote_branches()
        .unwrap()
        .flatten()
        .map(|branch| RemoteBranch {
            name: branch.name().shorten().to_string(),
            target: match branch.target() {
                Symbolic(target) => Some(target.shorten().to_string()),
                Object(_) => None,
            },
        })
        .collect()
}
