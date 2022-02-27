use std::collections::HashMap;

#[derive(Debug)]
pub struct RawPgn {
    pub id: String,
    pub tags: HashMap<String, String>,
    pub tags_text: String,
    pub moves_text: String,
}

impl RawPgn {
    pub fn new(prefix: impl std::fmt::Display, index: usize) -> RawPgn {
        Self {
            id: format!("{}.{}", prefix, index),
            tags: HashMap::new(),
            tags_text: String::new(),
            moves_text: String::new(),
        }
    }
}

mod reader;
pub use reader::{PgnReader, ReadOutcome};

mod parser;
pub use parser::ExtractMove;
