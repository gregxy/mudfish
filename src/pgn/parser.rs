use regex::Regex;

const RE_SAN: &str = r#"(?:[PNBRQK]?[a-h]?[1-8]?x?[a-h][1-8](?:=[PNBRQK])?[\+\#]?)"#;
const RE_INDEX: &str = r#"(?:\d+\.+)"#;
const RE_CASTLE: &str = r#"(?:O-O(?:-O)?[\+\#]?)"#;
const RE_SUFFIX: &str = r#"(?:[!?][!?]?)"#;
const RE_ANNOTATION: &str = r#"(?:\{[^{]+\})"#;
const RE_RESULT: &str = r#"((?:0-1)|(?:1-0)|(?:1/2-1/2)|\*)"#;

pub struct ExtractMove {
    full_re: Regex,
    move_re: Regex,
}

impl Default for ExtractMove {
    fn default() -> Self {
        let m = format!(
            r#"(?:({san}|{castle}){suffix}?(?:\s*{annotation})?)"#,
            san = RE_SAN,
            castle = RE_CASTLE,
            suffix = RE_SUFFIX,
            annotation = RE_ANNOTATION
        );

        let item = format!(r#"(?:{index}\s+{m}(?:\s+{m})?)"#, index = RE_INDEX, m = m);

        let full = format!(
            r#"{item}(?:\s+{item})*\s+(?P<result>{result})"#,
            item = item,
            result = RE_RESULT
        );

        Self {
            full_re: Regex::new(full.as_str()).unwrap(),
            move_re: Regex::new(item.as_str()).unwrap(),
        }
    }
}

impl ExtractMove {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extract(&self, text: &str) -> Option<(Vec<String>, String)> {
        let fullcap = self.full_re.captures(text);

        let result = fullcap?["result"].to_string();

        let mut moves: Vec<String> = Vec::new();
        for cap in self.move_re.captures_iter(text) {
            moves.push(cap[1].to_string());
            if cap.len() == 3 {
                moves.push(cap[2].to_string());
            }
        }

        Some((moves, result))
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    const RE_SAN_DETAIL: &str = r#"(?P<m>[PNBRQK]?)(?P<d>[a-h]?[1-8]?)(?P<x>x?)(?P<t>[a-h][1-8])(?P<p>(?:=[PNBRQK])?)(?P<c>[\+\#]?)"#;

    use super::*;

    #[derive(Debug)]
    struct San {
        m: &'static str,
        d: &'static str,
        x: &'static str,
        t: &'static str,
        p: &'static str,
        c: &'static str,
    }

    #[derive(Debug)]
    struct SanTest {
        text: &'static str,
        san: San,
    }

    #[test]
    fn match_san() {
        let re_san = Regex::new(RE_SAN_DETAIL).unwrap();

        let tests = [
            SanTest {
                text: "e4",
                san: San {
                    m: "",
                    d: "",
                    x: "",
                    t: "e4",
                    p: "",
                    c: "",
                },
            },
            SanTest {
                text: "Nxe5",
                san: San {
                    m: "N",
                    d: "",
                    x: "x",
                    t: "e5",
                    p: "",
                    c: "",
                },
            },
            SanTest {
                text: "Rad1+",
                san: San {
                    m: "R",
                    d: "a",
                    x: "",
                    t: "d1",
                    p: "",
                    c: "+",
                },
            },
        ];

        for test in tests.into_iter() {
            let caps = re_san.captures(test.text).unwrap();
            assert_eq!(&caps["m"], test.san.m);
            assert_eq!(&caps["d"], test.san.d);
            assert_eq!(&caps["x"], test.san.x);
            assert_eq!(&caps["t"], test.san.t);
            assert_eq!(&caps["p"], test.san.p);
            assert_eq!(&caps["c"], test.san.c);
        }
    }

    #[test]
    fn extract_move() {
        let pgn = r#"1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. c3 Nf6 5. d3 d6 6. Bg5 h6 7. Bh4 Qe7 8. O-O a6
9. b4 Ba7 10. Nbd2 g5 11. Bg3 Nh7 12. a4 h5 13. h4 g4 14. Ne1 Nf8 15. Nc2 Ng6
16. Ne3 Be6 17. b5 Nd8 18. d4 Nxh4 19. dxe5 dxe5 20. Bxe5 O-O 21. Bxe6 fxe6 22.
Bd4 c5 23. bxc6 Nxc6 24. Bxa7 Rxa7 25. Ndc4 Rd8 26. Qb3 Raa8 27. Rfd1 Kh8 28.
Rab1 Rxd1+ 29. Qxd1 Rd8 30. Qb3 Ng6 31. Rb2 Rd7 32. Nb6 Rd8 33. Nbc4 Rd7 34. Nb6
Rd8 35. Nbc4 Rd7 1/2-1/2"#;

        let ex = ExtractMove::new();
        let opt = ex.extract(pgn);
        assert!(opt.is_some());

        let (m, r) = opt.unwrap();
        assert_eq!(r, "1/2-1/2");
        assert_eq!(m.len(), 35 * 2);
    }
}
