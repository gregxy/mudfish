use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use bzip2::read::BzDecoder;
use regex::Regex;
use simple_error::simple_error;
use simple_error::SimpleError;
use simple_error::SimpleResult;

use super::parser::ExtractMove;
use super::RawPgn;

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
    extractor: ExtractMove,
}

#[derive(Debug)]
pub enum ReadOutcome {
    Game(RawPgn),
    KnownBadRecord(RawPgn),
    Ended,
    Error(SimpleError),
}

impl PgnReader {
    pub fn new(path: &Path) -> SimpleResult<Self> {
        let f = File::open(path).map_err(|e| simple_error!(e.to_string()))?;

        let buf: Box<dyn BufRead> = if path.extension().unwrap() == "bz2" {
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
            extractor: ExtractMove::default(),
        })
    }

    pub fn read_next(&mut self) -> ReadOutcome {
        if self.state == ReaderState::Ended {
            return ReadOutcome::Ended;
        }

        let mut pgn = if self.last_pgn.is_some() {
            self.last_pgn.take().unwrap()
        } else {
            self.count += 1;
            RawPgn::new(self.prefix.as_str(), self.count)
        };

        loop {
            let mut line = String::new();
            self.line_number += 1;

            let read_result = self.buf.read_line(&mut line);

            if let Err(e) = read_result {
                self.state = ReaderState::Ended;
                return ReadOutcome::Error(simple_error!(
                    "Line {}: {}",
                    self.line_number,
                    e.to_string()
                ));
            }

            if read_result.unwrap() == 0 {
                // end
                match self.state {
                    ReaderState::Moves => {
                        self.state = ReaderState::Ended;

                        return self.parse(pgn);
                    }
                    ReaderState::Tags => {
                        self.state = ReaderState::Ended;
                        return ReadOutcome::Error(simple_error!(
                            "Line {}: Ended unexpectedly.",
                            self.line_number
                        ));
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
                        self.count += 1;
                        let mut new_pgn = RawPgn::new(self.prefix.as_str(), self.count);
                        new_pgn
                            .tags
                            .insert(caps[1].to_string(), caps[2].to_string());
                        new_pgn.tags_text.push_str(trimmed);
                        new_pgn.tags_text.push('\n');
                        self.last_pgn = Some(new_pgn);

                        return self.parse(pgn);
                    }
                    _ => {
                        self.state = ReaderState::Tags;
                        pgn.tags_text.push_str(trimmed);
                        pgn.tags_text.push('\n');
                        pgn.tags.insert(caps[1].to_string(), caps[2].to_string());
                        continue;
                    }
                }
            }

            match self.state {
                ReaderState::Moves => {
                    pgn.moves_text.push_str(trimmed);
                    pgn.moves_text.push('\n');
                }
                ReaderState::Tags => {
                    pgn.moves_text.push_str(trimmed);
                    pgn.moves_text.push('\n');
                    self.state = ReaderState::Moves;
                }
                _ => {
                    self.state = ReaderState::Ended;
                    return ReadOutcome::Error(simple_error!(
                        "Line {}: Unexpected line: {}",
                        self.line_number,
                        line
                    ));
                }
            }
        }
    }

    fn parse(&self, pgn: RawPgn) -> ReadOutcome {
        match pgn.tags.get("Result") {
            None => ReadOutcome::Error(simple_error!(
                "Line {}: Missing result tag",
                self.line_number
            )),
            Some(result_tag) => {
                if result_tag == "0-0" {
                    return ReadOutcome::KnownBadRecord(pgn);
                }

                let extracted = self.extractor.extract(pgn.moves_text.as_str());
                if extracted.is_none() {
                    return ReadOutcome::Error(simple_error!(
                        "Line {}: Cannot parse move text",
                        self.line_number
                    ));
                }

                let (_, result) = extracted.unwrap();
                if &result != result_tag {
                    ReadOutcome::Error(simple_error!(
                        "Line {}: Result tag ({}) != result sentinel ({})",
                        self.line_number,
                        result_tag,
                        result
                    ))
                } else {
                    ReadOutcome::Game(pgn)
                }
            }
        }
    }
}
