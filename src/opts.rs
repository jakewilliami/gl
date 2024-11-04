#[derive(Clone)]
pub struct GitLogOptions {
    pub relative: bool, // relative commit dates
    pub colour: bool,
    pub reverse: bool,
    pub all: bool,

    // Filter commits by author or grep
    pub authors: Vec<String>,
    pub needles: Vec<String>,
}

impl Default for GitLogOptions {
    fn default() -> Self {
        Self {
            relative: true,
            colour: true,
            reverse: false,
            all: false,
            authors: Vec::new(),
            needles: Vec::new(),
        }
    }
}
