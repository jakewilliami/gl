use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct GitContributor {
	author: String,
	file_contributions: Vec<GitFileContribution>,
}

#[derive(Debug)]
struct GitFileContribution {
	lines_added: usize,
	lines_deleted: usize,
	lines_written: isize,
}

#[derive(Debug)]
struct ContributionSummary {
	lines_added: usize,
	lines_deleted: usize,
	lines_written: isize,
}

trait ContributorStats {
	fn contribution_summary(&self) -> ContributionSummary;
}

impl ContributorStats for GitContributor {
	fn contribution_summary(&self) -> ContributionSummary {
		let mut lines_added: usize = 0;
		let mut lines_deleted: usize = 0;
		let mut lines_written: isize = 0;
		
		for contribution in &self.file_contributions {
			lines_added += contribution.lines_added;
			lines_deleted += contribution.lines_deleted;
			lines_written += contribution.lines_written;
		}
		
		ContributionSummary{
			lines_added,
			lines_deleted,
			lines_written,
		}
	}
}

pub fn git_contributions(author: String) -> GitContributor {
	// git log --no-merges --author="Jake Ireland" --pretty=tformat: --numstat
    let mut cmd = Command::new("git");
	cmd.arg("log");
	cmd.arg("--no-merges");
	cmd.arg("--author=\"Jake Ireland\"");
	cmd.arg("--pretty=tformat: ");  // todo: this is returning blank!
	cmd.arg("--numstat");
	println!("{:?}", cmd);
	
	let output = cmd
		.stdout(Stdio::piped())
		.output()
		.expect("Failed to execute `git log`");
	println!("{:?}", output);
	
	if output.status.success() {
		let git_log = String::from_utf8_lossy(&output.stdout)
			.into_owned();
		
		let file_contributions: Vec<GitFileContribution> = git_log.split_terminator('\n')
		    .into_iter()
		    .map(|s| {
			    let mut parts = s.split_whitespace();
				let lines_added: usize = parts.next().unwrap().parse::<usize>().unwrap();
				let lines_deleted: usize = parts.next().unwrap().parse::<usize>().unwrap();
				
				GitFileContribution{
					lines_added,
					lines_deleted,
					lines_written: (lines_added as isize) - (lines_deleted as isize),
				}
			})
		    .collect();
		
		GitContributor{
			author,
			file_contributions,
		}
	} else {
		println!("An error has occured.  It is likely that you aren't in a git repository, or you may not have `git` installed.");
		
		GitContributor{
			author,
			file_contributions: vec![],
		}
	}
}

