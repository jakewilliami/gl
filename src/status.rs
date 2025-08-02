use super::opts::GitLogOptions;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn get_git_status(dir: &Option<String>, opts: &GitLogOptions) {
    let given_dir: PathBuf = if (dir).is_none() {
        std::env::current_dir().unwrap()
    } else {
        PathBuf::from(dir.clone().unwrap())
    };
    let status: String = git_status(&given_dir.into_os_string(), opts);
    println!("{}", status.trim_end())
}

fn git_status(dir: &OsString, opts: &GitLogOptions) -> String {
    let mut cmd = Command::new("git");
    if opts.colour {
        cmd.arg("-c");
        cmd.arg("color.status=always");
    }
    cmd.arg("status");
    cmd.arg("--short");
    cmd.arg("--branch");
    cmd.arg(dir);

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git status`");

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).into_owned()
    } else {
        println!(
            "An error has occured.  It is likely that you aren't in a git repository, or you may not have `git` installed."
        );

        "".to_string()
    }
}

#[allow(dead_code)]
fn git_diff_exit_code(dir: &OsString) {
    let mut cmd = Command::new("git");
    cmd.arg("-C");
    cmd.arg(dir);
    cmd.arg("diff-index");
    cmd.arg("--quiet");
    cmd.arg("HEAD");
    cmd.arg("--");

    cmd.stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git status`");
}
