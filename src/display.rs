use crate::opts::GitOptions;

pub trait Format {
    fn pretty(&self, opts: &GitOptions) -> String;
}
