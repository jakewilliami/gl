use super::commit::{git_log, GitCommit};
use super::opts::GitLogOptions;
use colored::*;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    // En dash
    static ref INFO_SEP_CHAR: char = char::from_u32(0x2013).unwrap();
}

trait Format {
    fn format_parts(&self, opts: &GitLogOptions) -> HashMap<&str, String>;
    fn pretty(&self, opts: &GitLogOptions) -> String;
}

impl Format for GitCommit {
    fn format_parts(&self, opts: &GitLogOptions) -> HashMap<&str, String> {
        let mut parts = HashMap::new();

        // 1. Hash
        parts.insert("hash", self.hash.short.clone());

        // 2. (Optional) ref name(s)
        parts.insert("refnames", {
            let mut refs = String::new();
            if !self.refs.is_empty() {
                refs.push('(');
                for (i, ref_name) in self.refs.iter().enumerate() {
                    refs.push_str(ref_name);

                    if i < self.refs.len() - 1 {
                        refs.push_str(", ");
                    }
                }
                refs.push_str(") ");
            }
            refs
        });

        // 3. Message (head)
        parts.insert("message", self.message.clone().trim().to_string());

        // 4. Date
        parts.insert("date", {
            let mut date = String::new();
            date.push('(');
            if opts.relative {
                date.push_str(&self.date.rel);
            } else {
                date.push_str(&self.date.format_abs());
            }
            date.push(')');
            date
        });

        // 5. Identity
        parts.insert("identity", {
            let mut id = String::new();
            if let Some(auth) = self.id.names.first() {
                id.push_str(&format!("<{}>", auth));
            }
            id
        });

        parts
    }

    fn pretty(&self, opts: &GitLogOptions) -> String {
        let parts = self.format_parts(opts);
        let identity = parts.get("identity").unwrap();

        format!(
            "{} {} {}{} {} {}",
            parts.get("hash").unwrap().yellow().bold(),
            *INFO_SEP_CHAR,
            parts.get("refnames").unwrap().green().bold(),
            parts.get("message").unwrap(),
            parts.get("date").unwrap().red().bold(),
            if self.id.clone().is_me() {
                identity.truecolor(192, 207, 227)
            } else {
                identity.blue().bold()
            }
        )
    }
}

pub fn display_git_log(n: usize, opts: &GitLogOptions) {
    let logs: Vec<GitCommit> = git_log(Some(n), Some(opts));

    for log in logs {
        println!("{}", log.pretty(opts));
    }
}
