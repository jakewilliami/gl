use gix::Repository;
use std::path::PathBuf;

pub fn discover_repository() -> Option<Repository> {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    gix::discover(current_dir).ok()
}

pub fn top_level_repo_path() -> Option<PathBuf> {
    // TODO: we should probably give a good error message here
    let repo = discover_repository().unwrap();
    repo.git_dir().parent().map(|path| path.to_owned())
}

pub fn current_repository() -> Option<String> {
    top_level_repo_path()?
        .file_name()?
        .to_str()
        .map(|s| s.to_string())
}
