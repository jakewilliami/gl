use std::process::{Command, Stdio};

pub enum BranchListings {
	Local,
	Remotes,
}

pub fn get_branch_names(bt: BranchListings) {
	let branch_names: Option<String> = match bt {
		BranchListings::Local => branch_names(),
		BranchListings::Remotes => remote_branches(),
	};
	
	if let Some(mut stripped_branch_names) = branch_names {
		if stripped_branch_names.ends_with('\n') {
			stripped_branch_names.pop();
			
			if stripped_branch_names.ends_with('\r') {
            	stripped_branch_names.pop();
        	}
		}
		
		for b in stripped_branch_names.split_terminator('\n') {
			println!("{}", b);
		}
	}
}

pub fn current_branch() -> Option<String> {
	let mut cmd = Command::new("git");
	cmd.arg("rev-parse");
	cmd.arg("--abbrev-ref");
	cmd.arg("HEAD");
	
	let output = cmd
		.stdout(Stdio::piped())
		.output()
		.expect("Failed to execute `git branch`");
	
	if output.status.success() {
		let mut current_branch_name = String::from_utf8_lossy(&output.stdout).into_owned();
		
		if current_branch_name.ends_with('\n') {
			current_branch_name.pop();
			
			if current_branch_name.ends_with('\r') {
            	current_branch_name.pop();
        	}
		}
		
		Some(current_branch_name)
	} else {
		None
	}
}

fn branch_names() -> Option<String> {
	let mut cmd = Command::new("git");
	cmd.arg("branch");
	cmd.arg("--color");
	
	let output = cmd
		.stdout(Stdio::piped())
		.output()
		.expect("Failed to execute `git branch`");
	
	if output.status.success() {
		let branch_names = String::from_utf8_lossy(&output.stdout).into_owned();
		
		Some(branch_names)
	} else {
		None
	}
}

fn remote_branches() -> Option<String> {
	let mut cmd = Command::new("git");
	cmd.arg("branch");
	cmd.arg("--color");
	cmd.arg("--remotes");
	
	let output = cmd
		.stdout(Stdio::piped())
		.output()
		.expect("Failed to execute `git branch`");
	
	if output.status.success() {
		let branch_names = String::from_utf8_lossy(&output.stdout).into_owned();
		
		Some(branch_names)
	} else {
		None
	}
}

