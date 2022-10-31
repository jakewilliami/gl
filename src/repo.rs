use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};

pub fn top_level_repo_path() -> Option<String> {
	let mut cmd = Command::new("git");
	cmd.arg("rev-parse");
	cmd.arg("--show-toplevel");
	let output = cmd
		.stdout(Stdio::piped())
		.output()
		.expect("Failed to execute `git rev-parse`");
	
	if output.status.success() {
		let mut current_repo_path = String::from_utf8_lossy(&output.stdout).into_owned();
		
		// strip the output of any new lines
		if current_repo_path.ends_with('\n') {
			current_repo_path.pop();
			if current_repo_path.ends_with('\r') {
            	current_repo_path.pop();
        	}
		}
		Some(current_repo_path)
	} else {
		None
	}
}

pub fn current_repository() -> Option<String> {
	let current_repo_path = top_level_repo_path();
	
	if let Some(current_repo_path) = current_repo_path {
		// get the basename of the path
		let repo_path = Path::new(&current_repo_path);
		let repo_path_basename = PathBuf::from(
			repo_path.file_name().unwrap()
		);
		
		let repo_basename_str: String = repo_path_basename
			.into_os_string()
			.into_string()
			.unwrap();
		
		Some(repo_basename_str)
	} else {
		None
	}
}
