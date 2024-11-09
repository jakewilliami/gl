use chrono::{DateTime, Local};

#[derive(Clone)]
pub struct CommitDate {
    pub abs: DateTime<Local>,
    pub repr: String,
    pub rel: String,
}

impl CommitDate {
    pub fn format_abs(&self) -> String {
        self.abs.format("%a %d %b %Y").to_string()
    }
}
