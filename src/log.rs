use super::commit::{git_log, GitCommit};
use super::config;
use super::opts::GitLogOptions;
use colored::*;
use regex::Regex;

trait Format {
    fn pretty(&self, opts: &GitLogOptions) -> String;
}

impl Format for GitCommit {
    fn pretty(&self, opts: &GitLogOptions) -> String {
        let re_named = Regex::new(r"<(?P<author>[^>]*)>").unwrap();
        let re = Regex::new(r"<([^>]*)>").unwrap();
        // TODO: in future, instead of using raw, we can add colours ourself
        //   This would be extra beneficial as in some repos (for example, grafana), there are commits (for example, aba824a317) that have no author (%an), so we can use their name instead (at least, the first valid thing matching from identity---make an identity display function to find it)
        let log = &self.raw;
        let log: String = log.replace('\"', "");
        let auth = re_named
            .captures(&log)
            .unwrap()
            .name("author") // using named groups
            .unwrap()
            .as_str()
            .to_string();

        // Need not colour author if colour not set
        // TODO: do I need to use more regex here?  Can I not replace the regex to just match with the author's name (which we already obtained)?
        if opts.colour && config::ME_IDENTITY.contains(&auth.as_str()) {
            re.replace(&log, |caps: &regex::Captures| {
                format!(
                    "{}{}{}{}",
                    "".normal().white(), // need this to clear the current line of any colours
                    "<".truecolor(192, 207, 227), // this is the light blue colour I have, defined by \e[0m\e[36m$&\e[39m\e[0m
                    &caps[1].truecolor(192, 207, 227),
                    ">".truecolor(192, 207, 227)
                )
            })
            .to_string()
        } else {
            log.to_string()
        }
    }
}

pub fn display_git_log(n: usize, opts: &GitLogOptions) {
    let logs: Vec<GitCommit> = git_log(Some(n), Some(opts));

    for log in logs {
        println!("{}", log.pretty(opts));
    }
}
