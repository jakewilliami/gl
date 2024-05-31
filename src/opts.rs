pub struct GitLogOptions {
    pub colour: bool,
}

impl Default for GitLogOptions {
    fn default() -> Self {
        Self { colour: true }
    }
}
