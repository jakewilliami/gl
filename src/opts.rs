#[derive(Clone, Copy)]
pub struct GitLogOptions {
    pub relative: bool, // relative commit dates
    pub colour: bool,
    pub reverse: bool,
    pub all: bool,
}

impl Default for GitLogOptions {
    fn default() -> Self {
        Self {
            relative: true,
            colour: true,
            reverse: false,
            all: false,
        }
    }
}
