use std::collections::HashMap;

#[derive(Debug)]
pub struct Pgn {
    pub id: String,
    pub tags: HashMap<String, String>,
    pub moves: Vec<String>,
    pub tags_text: String,
    pub moves_text: String,
}

impl Pgn {
    pub fn new(prefix: impl std::fmt::Display, index: usize) -> Pgn {
        Self {
            id: format!("{}.{}", prefix, index),
            tags: HashMap::new(),
            tags_text: String::new(),
            moves_text: String::new(),
            moves: Vec::new(),
        }
    }
}

mod reader;
pub use reader::{ReadOutcome, Reader};

pub(crate) mod extractor;
