use super::config::SHORT_HASH_LENGTH;

pub trait HashFormat {
    #[allow(dead_code)]
    fn short(&self) -> String;
}

impl HashFormat for String {
    fn short(&self) -> String {
        // github.com/jakewilliami/mktex/blob/e5430b18/src/remote.rs#L56
        match self.char_indices().nth(SHORT_HASH_LENGTH) {
            None => self.to_string(),
            Some((idx, _)) => (self[..idx]).to_string(),
        }
    }
}
