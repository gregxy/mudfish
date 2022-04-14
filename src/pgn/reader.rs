use bzip2::read::BzDecoder;
use regex::Regex;
use seahash::SeaHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use super::extractor::Extractor;
use super::Pgn;

#[derive(PartialEq, Debug)]
enum ReaderState {
    Start,
    Tags,
    Moves,
    Ended,
}

pub struct Reader {
    prefix: String,
    buf: Box<dyn BufRead>,
    state: ReaderState,
    re_tag: Regex,
    line_number: usize,
    count: usize,
    last_pgn: Option<Pgn>,
    extractor: Extractor,
}

#[derive(Debug)]
pub enum ReadOutcome {
    Game(Pgn),
    BadPgn(String),
    Ended,
    Error(String),
}

impl Reader {
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let f = File::open(path)?;

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
            extractor: Extractor::default(),
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
            Pgn::new(self.prefix.as_str(), self.count)
        };

        loop {
            let mut line = String::new();
            self.line_number += 1;

            let read_result = self.buf.read_line(&mut line);

            if let Err(e) = read_result {
                self.state = ReaderState::Ended;
                return ReadOutcome::Error(format!("Line {}: {}", self.line_number, e));
            }

            if read_result.unwrap() == 0 {
                // end
                match self.state {
                    ReaderState::Moves => {
                        self.state = ReaderState::Ended;

                        return self.postprocess(pgn);
                    }
                    ReaderState::Tags => {
                        self.state = ReaderState::Ended;
                        return ReadOutcome::Error(format!(
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
                        let mut new_pgn = Pgn::new(self.prefix.as_str(), self.count);
                        new_pgn
                            .tags
                            .insert(caps[1].to_string(), caps[2].to_string());
                        new_pgn.tags_text.push_str(trimmed);
                        new_pgn.tags_text.push('\n');
                        self.last_pgn = Some(new_pgn);

                        return self.postprocess(pgn);
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
                    return ReadOutcome::Error(format!(
                        "Line {}: Unexpected line: {}",
                        self.line_number, line
                    ));
                }
            }
        }
    }

    fn badpgn(&self, pgn: &Pgn, message: String) -> ReadOutcome {
        return ReadOutcome::BadPgn(format!(
            "Line {}: invalid pgn: {}\n{}\n{}\n",
            self.line_number, message, pgn.tags_text, pgn.moves_text
        ));
    }

    fn postprocess(&self, mut pgn: Pgn) -> ReadOutcome {
        let result_tag_opt = pgn.tags.get("Result");
        if result_tag_opt.is_none() {
            return self.badpgn(&pgn, "missing result tag".to_string());
        }

        let result_tag = result_tag_opt.unwrap();
        if result_tag != "1-0"
            && result_tag != "0-1"
            && result_tag != "1/2-1/2"
            && result_tag != "*"
        {
            return self.badpgn(&pgn, format!("bad result tag ({})", result_tag));
        }

        let extracted = self.extractor.extract(pgn.moves_text.as_str());
        if extracted.is_none() {
            return self.badpgn(&pgn, "cannot extract move list".to_string());
        }

        let (moves, last_index, result) = extracted.unwrap();
        if &result != result_tag {
            return self.badpgn(
                &pgn,
                format!(
                    "result tag ({}) != result sentinel ({})",
                    result_tag, result
                ),
            );
        }

        if last_index * 2 != moves.len() && last_index * 2 - 1 != moves.len() {
            return self.badpgn(
                &pgn,
                format!(
                    "last move index == {}, but # of moves (white + black) == {}",
                    last_index,
                    moves.len()
                ),
            );
        }

        pgn.moves = moves;
        pgn.moves_fingerprint = moves_fingerprint(&pgn.moves);

        ReadOutcome::Game(pgn)
    }
}

fn moves_fingerprint(moves: &Vec<String>) -> u64 {
    let mut hasher = SeaHasher::new();

    for m in moves.iter() {
        hasher.write(m.as_bytes());
    }

    return hasher.finish();
}
