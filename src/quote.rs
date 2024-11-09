use lazy_static::lazy_static;

lazy_static! {
    // Quotes for log metadata
    static ref INITIAL_QUOTE_CHAR: char = char::from_u32(0x201C).unwrap();
    pub static ref FINAL_QUOTE_CHAR: char = char::from_u32(0x201D).unwrap();
}

pub trait Quote {
    fn quote(&self) -> String;
}

impl Quote for String {
    fn quote(&self) -> String {
        format!("{}{}{}", *INITIAL_QUOTE_CHAR, &self, *FINAL_QUOTE_CHAR)
    }
}
