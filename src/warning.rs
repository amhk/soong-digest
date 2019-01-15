use crate::ansi::strip_ansi_escape;
use crate::item::Item;
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::From;

pub fn parse<'a, 'b>(haystack: &'a str) -> impl Iterator<Item = Item> + 'b {
    let mut items: Vec<Item> = Vec::new();
    let haystack = strip_ansi_escape(haystack);
    let captures = find_captures(&haystack);
    for c in captures {
        items.push(Item::from(c));
    }
    items.into_iter()
}

struct Captures<'h> {
    head: &'h str,
    body: Vec<&'h str>,
}

impl<'h> From<Captures<'h>> for Item {
    fn from(captures: Captures<'h>) -> Self {
        lazy_static! {
            static ref re: Regex = Regex::new(r"(\S+):(\d+):(\d+): warning: (.*)").unwrap();
        }
        let caps = re.captures(&captures.head).unwrap();
        Item {
            path: caps.get(1).unwrap().as_str().to_string(),
            line: caps.get(2).unwrap().as_str().parse().unwrap(),
            column: Some(caps.get(3).unwrap().as_str().parse().unwrap()),
            subject: caps.get(4).unwrap().as_str().to_string(),
            body: captures.body.join("\n"),
        }
    }
}

fn find_captures(haystack: &str) -> Vec<Captures> {
    lazy_static! {
        static ref re_subject: Regex = Regex::new(r"^\S+:\d+:\d+: warning: .*$").unwrap();
        static ref re_noise: Regex = Regex::new(r"^\[\d+/\d+\]").unwrap();
    }
    let mut captures = Vec::new();
    let mut current: Option<Captures> = None;
    for line in haystack.lines() {
        if re_subject.is_match(line) {
            if current.is_some() {
                captures.push(current.take().unwrap());
            }
            current = Some(Captures {
                head: line,
                body: vec![],
            });
            continue;
        }
        if re_noise.is_match(line) {
            if current.is_some() {
                captures.push(current.take().unwrap());
            }
            continue;
        }
        if current.is_some() {
            let mut c = current.unwrap();
            c.body.push(line);
            current = Some(c);
            continue;
        }
    }
    if current.is_some() {
        captures.push(current.unwrap());
    }
    captures
}

#[cfg(test)]
mod tests {
    use flate2::read::GzDecoder;
    use std::io::Read;

    fn uncompress_test_data() -> String {
        let raw: &[u8] = include_bytes!("../tests/data/easter-egg-errors-java/verbose.log.gz");
        let mut decoder = GzDecoder::new(raw);
        let mut contents = String::new();
        decoder.read_to_string(&mut contents).unwrap();
        contents
    }

    #[test]
    fn test_find_captures_empty() {
        let captures = super::find_captures("");
        assert_eq!(captures.len(), 0);
    }

    #[test]
    fn test_find_captures_single_line_no_noise() {
        let captures = super::find_captures("foo.c:10:20: warning: bar");
        assert_eq!(captures.len(), 1);
        let c = &captures[0];
        assert_eq!(c.head, "foo.c:10:20: warning: bar");
        assert_eq!(c.body, Vec::<&str>::new());
    }

    #[test]
    fn test_find_captures_multiple_lines_no_noise() {
        let captures = super::find_captures("foo.c:10:20: warning: bar\nbody line 1\nbody line 2");
        assert_eq!(captures.len(), 1);
        let c = &captures[0];
        assert_eq!(c.head, "foo.c:10:20: warning: bar");
        assert_eq!(c.body, vec!["body line 1", "body line 2"]);
    }

    #[test]
    fn test_find_captures_multiple_warnings_no_noise() {
        let captures = super::find_captures(
            "foo.c:10:20: warning: bar\nfoo 1\nbar.c:30:40: warning: foo\nbar 1",
        );
        assert_eq!(captures.len(), 2);

        let c = &captures[0];
        assert_eq!(c.head, "foo.c:10:20: warning: bar");
        assert_eq!(c.body, vec!["foo 1"]);

        let c = &captures[1];
        assert_eq!(c.head, "bar.c:30:40: warning: foo");
        assert_eq!(c.body, vec!["bar 1"]);
    }

    #[test]
    fn test_find_captures_single_line_surrounded_by_noise() {
        let captures = super::find_captures("[1/2] foo\nfoo.c:10:20: warning: bar\n[2/2] bar");
        assert_eq!(captures.len(), 1);
        let c = &captures[0];
        assert_eq!(c.head, "foo.c:10:20: warning: bar");
        assert_eq!(c.body, Vec::<&str>::new());
    }

    #[test]
    fn test_find_captures_actual_soong_output() {
        let contents = uncompress_test_data();
        let captures = super::find_captures(&contents);
        assert_eq!(captures.len(), 9);

        let c = &captures[0];
        assert_eq!(c.head, "frameworks/base/packages/EasterEgg/src/com/android/egg/paint/CutoutAvoidingToolbar.kt:85:22: warning: parameter 'attrs' is never used");
        assert_eq!(
            c.body,
            vec![
                "    private fun init(attrs: AttributeSet?, defStyle: Int) {",
                "                     ^"
            ]
        );
    }

    #[test]
    fn test_parse() {
        let items = super::parse("[1/2] foo\nfoo.c:10:20: warning: bar\nbody 1\nbody 2\n[2/2] bar")
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.path, "foo.c");
        assert_eq!(item.line, 10);
        assert_eq!(item.column, Some(20));
        assert_eq!(item.subject, "bar");
        assert_eq!(item.body, "body 1\nbody 2");
    }

    #[test]
    fn test_parse_actual_soong_output() {
        let contents = uncompress_test_data();
        let items = super::parse(&contents).collect::<Vec<_>>();
        assert_eq!(items.len(), 9);

        let item = &items[0];
        assert_eq!(
            item.path,
            "frameworks/base/packages/EasterEgg/src/com/android/egg/paint/CutoutAvoidingToolbar.kt"
        );
        assert_eq!(item.line, 85);
        assert_eq!(item.column, Some(22));
        assert_eq!(item.subject, "parameter 'attrs' is never used");
        assert_eq!(
            item.body,
            "    private fun init(attrs: AttributeSet?, defStyle: Int) {\n                     ^"
        );
    }
}
