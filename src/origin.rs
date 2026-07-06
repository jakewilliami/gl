use std::process::{Command, Stdio};

pub fn get_remote_origin_url() {
    if let Some(origin_url) = remote_origin_url() {
        println!("{origin_url}")
    }
}

// You can use either of the following:
//     git remote get-url origin
//     git config --get remote.origin.url
//
// <https://stackoverflow.com/a/32991784>
// <https://stackoverflow.com/a/4089452>
pub fn remote_origin_url() -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.arg("remote");
    cmd.arg("get-url");
    cmd.arg("origin");

    let output = cmd
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute `git remote`");

    if output.status.success() {
        let origin_url = String::from_utf8_lossy(&output.stdout)
            .trim()
            .trim_end_matches(".git")
            .to_owned();
        Some(origin_url)
    } else {
        // TODO: String::from_utf8_lossy(&output.stderr).into_owned()
        None
    }
}
