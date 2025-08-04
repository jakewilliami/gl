use super::{
    date::CommitDate, identity::GitIdentity, opts::GitLogOptions, repo::discover_repository,
};
use gix::{bstr::ByteSlice, revision::walk::Sorting, traverse::commit::simple::CommitTimeOrder};
use lazy_static::lazy_static;
use regex::Regex;
use std::char;
use std::collections::HashMap;

lazy_static! {
    // This is a good separating dash, but relies on it not being used inside commit messages!
    static ref META_SEP_CHAR: char = char::from_u32(0x2E3A).unwrap();

    // Quotes for log metadata
    // These need to be unique.  They are not traditional quotes.  See  v3.0.2 and v3.1.2.
    static ref INITIAL_QUOTE_CHAR: char = char::from_u32(0x2560).unwrap();
    static ref FINAL_QUOTE_CHAR: char = char::from_u32(0x2563).unwrap();

    //Regex for commit logs
    static ref UNTIL_FINAL_QUOTE_PAT: String = format!(r"[^{}]", *FINAL_QUOTE_CHAR);
    static ref DATE_META_PAT: String = format!(r"(?P<dateabs>{}+)", *UNTIL_FINAL_QUOTE_PAT).quote();
    static ref HASH_META_PAT: String = String::from(r"(?P<fullhash>[a-f0-9]+)").quote();
    static ref EMAIL_META_PAT: String = format!(r"(?P<email>{}*)", *UNTIL_FINAL_QUOTE_PAT).quote();
    static ref COMMIT_LOG_RE: Regex = Regex::new(
        &format!(
            r"^(?P<raw>(?P<hash>[a-f0-9]+)\s\-\s(\((?P<meta>[^\)]+)\)\s)?(?P<message>.+)\((?P<daterepr>[^\)]+)\)\s<(?P<author>[^>]*)>){}dateabs\:\s{},\shash\:\s{},\semail\:\s{}$",
            *META_SEP_CHAR,
            *DATE_META_PAT,
            *HASH_META_PAT,
            *EMAIL_META_PAT,
        ),
    )
        .unwrap();
}

#[derive(Clone)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub refs: Vec<String>,
    pub date: CommitDate,
    pub id: GitIdentity,
}

trait Quote {
    fn quote(&self) -> String;
}

impl Quote for String {
    fn quote(&self) -> String {
        format!("{}{}{}", *INITIAL_QUOTE_CHAR, &self, *FINAL_QUOTE_CHAR)
    }
}

// TODO: temporary function; should use iterator (see https://github.com/jakewilliami/gl/commit/44df7970eda30677b1199903a09a660f6367c1bd)
pub fn git_log_iter(
    n: Option<usize>,
    opts: Option<&GitLogOptions>,
) -> Box<dyn Iterator<Item = GitCommit>> {
    Box::new(git_log(n, opts).into_iter())
}

pub fn git_log(n: Option<usize>, opts: Option<&GitLogOptions>) -> Vec<GitCommit> {
    let opts = if let Some(opts) = opts {
        opts
    } else {
        &GitLogOptions::default()
    };

    // TODO: we should probably give a good error message here
    let repo = discover_repository().unwrap();

    // Get most recent commit at head
    let commit = repo.head_commit().unwrap();

    // The following is a pre-computation step developed to collate a list of ref names
    // associated with each commit.  This process was discovered by following the trail
    // from `gix`'s `describe` functionality:
    //   https://github.com/GitoxideLabs/gitoxide/blob/ccd6525c/gitoxide-core/src/repository/commit.rs#L59-L78
    //
    // Namely, I adapted the following:
    //   https://github.com/GitoxideLabs/gitoxide/blob/ccd6525c/gix/src/commit.rs#L108-L145
    //
    // This was previously computed and formatted directly within `git`:
    //   https://github.com/jakewilliami/gl/blob/v3.1.1/src/commit.rs#L144-L198
    //
    // Still TODO: iterate over commits from HEAD, rather than all commits; and consider attempting to clean up ref names?  Prefix with `tag:`; `HEAD ->` where applicable; shorten or not; sort them properly; check target ID; etc.
    let platform = repo.references().unwrap();
    let mut refs = HashMap::<String, Vec<String>>::new();
    for r in platform.all().unwrap().filter_map(Result::ok) {
        // let target_id = r.clone().target().try_id().map(ToOwned::to_owned);
        let peeled_id = r.clone().peel_to_id_in_place().ok().unwrap();
        refs.entry(peeled_id.to_hex().to_string())
            .or_default()
            .push(r.inner.name.shorten().to_string());
    }

    // Create an iterator over relevant commits, to use downstream (e.g., for logging
    // or contribution statistics).  This was developed (alongside wading through the
    // `gix` crate documentation) partly based on the following example:
    //   https://github.com/GitoxideLabs/gitoxide/blob/ccd6525c/examples/log.rs#L94-L157
    let log_iter: Box<dyn Iterator<Item = GitCommit>> = Box::new(
        repo.rev_walk([commit.id])
            .sorting(Sorting::ByCommitTime(CommitTimeOrder::NewestFirst))
            .all()
            .unwrap()
            // TODO: filter out merges
            .filter_map(|info| {
                if let Ok(info) = info {
                    // Get commit info
                    let commit = info.object().unwrap();
                    let commit_ref = commit.decode().unwrap();

                    // We want to filter out merges.  We can do this by filtering out
                    // the commit if it has more than one parent
                    let mut parents = commit_ref.parents();
                    parents.next();
                    if parents.next().is_some() {
                        return None;
                    }

                    // Get author info
                    // TODO: allow GitIdentity by author rather than committer
                    let author = commit_ref.author.actor();
                    let committer = commit_ref.committer();

                    if commit.short_id().unwrap().to_string() == *"8d38e3e" {
                        dbg!(&commit);
                        dbg!(&commit_ref);
                    }

                    // Filter for author
                    // TODO: why is this so slow?  Without this match, the change since v3.1.0 is ∼1.8 × faster, but with this filter it is ∼2.6 × slower
                    if !opts.authors.is_empty() {
                        // NOTE: we intentionally currently filter on committer (to match
                        // `git`'s `--author` behaviour)---just the specified author,
                        // even though that can be contrived.
                        let author_identities = [author.email, author.name];

                        let is_author_match = opts.authors.iter().any(|author| {
                            author_identities
                                .iter()
                                .any(|possible_author| possible_author.contains_str(author))
                        });

                        if !is_author_match {
                            return None;
                        }
                    }

                    // Filter for needles in commit messages
                    // TODO: this is also ∼2.8 × slower than the git equivalent, `--author`.  Why?
                    if !opts.needles.is_empty() {
                        let is_needle_match = opts
                            .needles
                            .iter()
                            .all(|needle| commit_ref.message.contains_str(needle));

                        if !is_needle_match {
                            return None;
                        }
                    }

                    Some(GitCommit {
                        hash: commit.id().to_hex().to_string(),
                        refs: refs
                            .get(&commit.id().to_hex().to_string())
                            .unwrap_or(&vec![])
                            .to_vec(),
                        message: commit_ref.message().title.to_str_lossy().into_owned(),
                        date: CommitDate::from(committer.time),
                        id: GitIdentity {
                            email: author.email.to_str_lossy().into_owned(),
                            names: vec![author.name.to_str_lossy().into_owned()],
                        },
                    })
                } else {
                    None
                }
            }),
    );

    let mut logs: Vec<GitCommit> = log_iter.collect();

    // TODO: It would be ideal to get this working without collecting the logs first.
    // `gix` should be able to handle sorting in a different order.  I have requested
    // community help with this:
    //   https://github.com/GitoxideLabs/gitoxide/discussions/1669
    //
    // The ideal sorting algorithm is as follows:
    //   .sorting(Sorting::ByCommitTime(if !opts.reverse {
    //       CommitTimeOrder::NewestFirst
    //   } else {
    //       CommitTimeOrder::OldestFirst
    //   }))
    //
    // Hopefully I can get this working soon, as collecting and turning it back into
    // an iterator after reversing seems quite inefficient.
    if opts.reverse {
        logs.reverse()
    }

    if let Some(n) = n {
        if !opts.all {
            logs = logs.into_iter().take(n).collect();
        }
    }

    logs
}

/*pub struct GitLogIter {
    #[allow(dead_code)]
    log_data: Arc<String>,
    lines: std::str::Lines<'static>,
    opts: GitLogOptions,
}

impl Iterator for GitLogIter {
    type Item = GitCommit;

    fn next(&mut self) -> Option<Self::Item> {
        for log in self.lines.by_ref() {
            let log: String = log.replace('\"', "");
            let log_stripped = strip_ansi_escapes::strip_str(&log);
            if let Some(re_match) = COMMIT_LOG_RE.captures(&log_stripped) {
                return Some(GitCommit {
                    hash: re_match.name("fullhash")?.as_str().to_string(),
                    meta: re_match.name("meta").map(|s| s.as_str().to_string()),
                    message: re_match.name("message")?.as_str().to_string(),
                    date: CommitDate {
                        abs: {
                            let date_str = re_match.name("dateabs")?.as_str();
                            if self.opts.relative {
                                DateTime::parse_from_rfc2822(date_str).unwrap().into()
                            } else {
                                // TODO: this is slightly wrong, as it doesn't account for
                                //   the time zone of the commit, it just uses the local
                                //   timezone.  We need to extract the commit time zone from
                                //   the git log command
                                let now = Local::now();
                                let offset = now.offset();
                                NaiveDate::parse_from_str(date_str, "%a %d %b %Y")
                                    .unwrap()
                                    .and_hms_opt(0, 0, 0)
                                    .unwrap()
                                    .and_local_timezone(*offset)
                                    .unwrap()
                                    .into()
                            }
                        },
                        repr: re_match.name("daterepr")?.as_str().to_string(),
                    },
                    id: GitIdentity {
                        email: re_match.name("email")?.as_str().to_string(),
                        names: vec![re_match.name("author")?.as_str().to_string()],
                    },
                    raw: log
                        .split(&META_SEP_CHAR.to_string())
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                });
            }
        }
        None
    }
}

pub fn git_log_iter(
    n: Option<usize>,
    opts: Option<&GitLogOptions>,
) -> Box<dyn Iterator<Item = GitCommit>> {
    let opts = opts.cloned().unwrap_or_default();
    let log_data = Arc::new(git_log_str(n, &opts));

    // SAFETY: we coerce the lifetime of the `&str` to `'static` because `log_data`
    // is held by the iterator.  This is safe because `lines` doesn't outlive `log_data`.
    let static_ref: &'static str =
        unsafe { std::mem::transmute::<&str, &'static str>(log_data.as_str()) };

    let iter = GitLogIter {
        lines: static_ref.lines(),
        log_data,
        opts: opts.clone(),
    };

    if opts.reverse {
        Box::new(iter.collect::<Vec<_>>().into_iter().rev())
    } else {
        Box::new(iter)
    }
}

pub fn git_log(n: Option<usize>, opts: Option<&GitLogOptions>) -> Vec<GitCommit> {
    git_log_iter(n, opts).collect()
}*/
