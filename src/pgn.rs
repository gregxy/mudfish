use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use bzip2::read::BzDecoder;
use regex::Regex;
use simple_error::simple_error;
use simple_error::SimpleError;
use simple_error::SimpleResult;

#[derive(Debug)]
pub struct RawPgn {
    pub id: String,
    pub tags: BTreeMap<String, String>,
    pub moves: String,
}

impl RawPgn {
    pub fn new(prefix: impl std::fmt::Display, index: usize) -> RawPgn {
        Self {
            id: format!("{}.{}", prefix, index),
            tags: BTreeMap::new(),
            moves: String::new(),
        }
    }
}

#[derive(PartialEq, Debug)]
enum ReaderState {
    Start,
    Tags,
    Moves,
    Ended,
}

pub struct PgnReader {
    prefix: String,
    buf: Box<dyn BufRead>,
    state: ReaderState,
    re_tag: Regex,
    line_number: usize,
    count: usize,
    last_pgn: Option<RawPgn>,
}

#[derive(Debug)]
pub enum ReadOutcome {
    Game(RawPgn),
    Ended,
    EndedUnexpectedly,
    IoError(SimpleError),
    ParseError(SimpleError),
}

impl PgnReader {
    pub fn new(path: &Path) -> SimpleResult<Self> {
        let f = File::open(path).map_err(|e| simple_error!(e.to_string()))?;

        let buf: Box<dyn BufRead> = if path.extension().unwrap() == "bz" {
            Box::new(BufReader::new(BzDecoder::new(f)))
        } else {
            Box::new(BufReader::new(f))
        };

        let prefix: String = path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .to_string();

        Ok(Self {
            buf,
            prefix,
            state: ReaderState::Start,
            re_tag: Regex::new(r#"\[([[:word:]]+)\s+"([^"]*)"\]"#).unwrap(),
            line_number: 0,
            count: 0,
            last_pgn: None,
        })
    }

    pub fn read_next(&mut self) -> ReadOutcome {
        if self.state == ReaderState::Ended {
            return ReadOutcome::Ended;
        }

        let mut pgn = if self.last_pgn.is_some() {
            self.last_pgn.take().unwrap()
        } else {
            RawPgn::new(self.prefix.as_str(), self.count)
        };

        if pgn.tags.is_empty() {
            self.count += 1;
        }

        loop {
            let mut line = String::new();
            self.line_number += 1;

            let read_result = self.buf.read_line(&mut line);

            if let Err(e) = read_result {
                self.state = ReaderState::Ended;
                return ReadOutcome::IoError(simple_error!(
                    "line {}: {}",
                    self.line_number,
                    e.to_string()
                ));
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
                continue;
            }

            if let Some(caps) = self.re_tag.captures(trimmed) {
                match self.state {
                    ReaderState::Moves => {
                        self.state = ReaderState::Tags;
                        let mut new_pgn = RawPgn::new(self.prefix.as_str(), self.count);
                        new_pgn
                            .tags
                            .insert(caps[1].to_string(), caps[2].to_string());
                        self.last_pgn = Some(new_pgn);
                        self.count += 1;
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
                        pgn.moves.push(' ');
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
