use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use bzip2::read::BzDecoder;
use regex::Regex;
use simple_error::SimpleError;
use simple_error::SimpleResult;

#[derive(Default, Debug)]
pub struct RawPgn {
    pub tags: HashMap<String, String>,
    pub moves: String,
}

#[derive(PartialEq, Debug)]
enum ReaderState {
    Start,
    Tags,
    Moves,
    Ended,
}

pub struct PgnReader {
    buf: Box<dyn BufRead>,
    state: ReaderState,
    re_tag: Regex,
    line_number: u64,
}

#[derive(Debug)]
pub enum ReadOutcome {
    Game(RawPgn),
    Ended,
    EndedUnexpectedly,
    IoError(std::io::Error),
    ParseError(SimpleError),
}

impl PgnReader {
    pub fn new(path: &str) -> SimpleResult<Self> {
        let f = File::open(path).map_err(|e| SimpleError::new(e.to_string()))?;

        let buf: Box<dyn BufRead> = if path.ends_with(".bz2") {
            Box::new(BufReader::new(BzDecoder::new(f)))
        } else {
            Box::new(BufReader::new(f))
        };

        Ok(Self {
            buf,
            state: ReaderState::Start,
            re_tag: Regex::new(r#"\[([[:word:]]+)\s+"([^"]+)"\]"#).unwrap(),
            line_number: 0,
        })
    }

    pub fn read_next(&mut self) -> ReadOutcome {
        if self.state == ReaderState::Ended {
            return ReadOutcome::Ended;
        }

        let mut pgn = RawPgn::default();
        loop {
            let mut line = String::new();
            self.line_number += 1;

            let read_result = self.buf.read_line(&mut line);

            if let Err(e) = read_result {
                self.state = ReaderState::Ended;
                return ReadOutcome::IoError(e);
            }

            if read_result.unwrap() == 0 {
                // end
                match self.state {
                    ReaderState::Moves => {
                        self.state = ReaderState::Ended;
                        return ReadOutcome::Game(pgn);
                    }
                    ReaderState::Tags => {
                        self.state = ReaderState::Ended;
                        return ReadOutcome::EndedUnexpectedly;
                    }
                    _ => {
                        self.state = ReaderState::Ended;
                        return ReadOutcome::Ended;
                    }
                }
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                match self.state {
                    ReaderState::Moves => {
                        self.state = ReaderState::Start;
                        return ReadOutcome::Game(pgn);
                    }
                    _ => continue,
                }
            }

            if let Some(caps) = self.re_tag.captures(trimmed) {
                match self.state {
                    ReaderState::Moves => {
                        self.state = ReaderState::Ended;
                        return ReadOutcome::ParseError(SimpleError::new(format!(
                            "No empty line between moves and tags ({})",
                            self.line_number
                        )));
                    }
                    _ => {
                        self.state = ReaderState::Tags;
                        pgn.tags.insert(caps[1].to_string(), caps[2].to_string());
                        continue;
                    }
                }
            }

            match self.state {
                ReaderState::Moves => {
                    if !pgn.moves.is_empty() {
                        pgn.moves.push_str(" ");
                    }
                    pgn.moves.push_str(trimmed);
                }
                ReaderState::Tags => {
                    self.state = ReaderState::Moves;
                    pgn.moves.push_str(trimmed);
                }
                _ => {
                    self.state = ReaderState::Ended;
                    return ReadOutcome::ParseError(SimpleError::new(format!(
                        "Unexpected line ({}): {}",
                        self.line_number, line
                    )));
                }
            }
        }
    }
}
