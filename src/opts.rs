use clap::ValueEnum;

#[derive(Clone)]
pub struct GitOptions {
    pub colour: bool,
    pub reverse: bool,
    pub log: LogOptions,
}

impl Default for GitOptions {
    fn default() -> Self {
        Self {
            colour: true,
            reverse: false,
            log: LogOptions::default(),
        }
    }
}

#[derive(Clone)]
pub struct LogOptions {
    pub relative: bool, // relative commit dates
    pub all: bool,

    // Filter commits by author or grep
    pub filter: LogFilterOptions,
}

impl Default for LogOptions {
    fn default() -> Self {
        Self {
            relative: true,
            all: false,
            filter: LogFilterOptions::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct LogFilterOptions {
    pub authors: Vec<String>,
    pub needles: Vec<String>,
}

#[derive(Clone, ValueEnum)]
pub enum TagFormat {
    Short,
    Long,
}
