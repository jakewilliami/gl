use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

use crate::config;
use crate::opts::GitLogOptions;

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
        let git_status = String::from_utf8_lossy(&output.stdout).into_owned();

        git_status
    } else {
        println!("An error has occured.  It is likely that you aren't in a git repository, or you may not have `git` installed.");

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

pub fn global_status(opts: &GitLogOptions) {
    let mut input_file: OsString = config::BASE_DIR.into();
    input_file.push("/scripts/rust/gl/src/global.txt");
    let input =
        std::fs::read_to_string(input_file).expect("Something went wrong reading the input file");
    let v: Vec<&str> = input
        .split('\n')
        .filter(|s| !s.is_empty())
        .filter(|s| s.get(..1).unwrap() != "#")
        .collect::<Vec<&str>>();

    for r in v {
        let mut constructed_path: OsString = config::BASE_DIR.into();
        constructed_path.push("/");
        constructed_path.push(r);
        constructed_path.push("/");

        let status = git_status(&constructed_path, opts);
        let length_of_output: usize = status.split_terminator('\n').count(); // can also use .len()

        if length_of_output == 1_usize {
            continue;
        }

        println!("We are looking at {}", constructed_path.to_str().unwrap());
        println!("{}", status);
    }
}

// 25 function parse_git_dirty {
// 26     status=$(git status 2>&1 | tee)
// 27     dirty=$(echo -n "${status}" 2> /dev/null | grep "modified:" &> /dev/null; echo "$?")
// 28     untracked=$(echo -n "${status}" 2> /dev/null | grep "Untracked files" &> /dev/null; echo "$?")
// 29     ahead=$(echo -n "${status}" 2> /dev/null | grep "Your branch is ahead of" &> /dev/null; echo "$?")
// 30     newfile=$(echo -n "${status}" 2> /dev/null | grep "new file:" &> /dev/null; echo "$?")
// 31     renamed=$(echo -n "${status}" 2> /dev/null | grep "renamed:" &> /dev/null; echo "$?")
// 32     deleted=$(echo -n "${status}" 2> /dev/null | grep "deleted:" &> /dev/null; echo "$?")
// 33     bits=''
// 34     if [ "${renamed}" == "0" ]; then
// 35         bits=">${bits}"
// 36     fi
// 37     if [ "${ahead}" == "0" ]; then
// 38         bits="*${bits}"
// 39     fi
// 40     if [ "${newfile}" == "0" ]; then
// 41         bits="+${bits}"
// 42     fi
// 43     if [ "${untracked}" == "0" ]; then
// 44         bits="?${bits}"
// 45     fi
// 46     if [ "${deleted}" == "0" ]; then
// 47         bits="x${bits}"
// 48     fi
// 49     if [ "${dirty}" == "0" ]; then
// 50         bits="!${bits}"
// 51     fi
// 52     if [ ! "${bits}" == "" ]; then
// 53         echo " ${bits}"
// 54     else
// 55         echo ""
// 56     fi
// 57 }
