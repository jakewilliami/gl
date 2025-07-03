fn is_set(var: &str) -> bool {
    let val = std::env::var(var);

    // Value must be set and non-empty
    if let Ok(val) = val {
        !val.is_empty()
    } else {
        false
    }
}

// https://no-color.org
fn no_colour() -> bool {
    is_set("NO_COLOR") || is_set("NO_COLOUR")
}

pub fn colour() -> bool {
    !no_colour()
}
