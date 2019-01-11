use crate::ansi::strip_ansi_escape;
use crate::item::Item;
use regex::Regex;
use std::error::Error;

pub fn parse(haystack: &str) -> Result<Vec<Item>, Box<Error>> {
    let mut items: Vec<Item> = Vec::new();
    let captures = find_captures(haystack);
    for c in captures {
        match c.try_from() {
            Ok(item) => items.push(item),
            Err(e) => return Err(e),
        }
    }
    Ok(items)
}

struct Captures<'h> {
    head: &'h str,
    body: &'h str,
}

impl<'h> Captures<'h> {
    // switch to std::convert::TryFrom once that becomes stable
    fn try_from(&self) -> Result<Item, Box<Error>> {
        let re = Regex::new(r"(\S+):(\d+):(\d+): warning: (.*)").unwrap();

        let caps = match re.captures(&self.head) {
            Some(caps) => caps,
            None => return Err(From::from(format!("failed to parse '{}'", self.head))),
        };
        let line_str = caps.get(2).unwrap().as_str();
        let line = match line_str.parse() {
            Ok(line) => line,
            Err(_) => {
                return Err(From::from(format!(
                    "failed to convert '{}' to int",
                    line_str
                )))
            }
        };
        let column_str = caps.get(3).unwrap().as_str();
        let column = match column_str.parse() {
            Ok(i) => Some(i),
            Err(_) => {
                return Err(From::from(format!(
                    "failed to convert '{}' to int",
                    column_str
                )))
            }
        };
        Ok(Item {
            path: strip_ansi_escape(caps.get(1).unwrap().as_str()),
            line,
            column,
            subject: strip_ansi_escape(caps.get(4).unwrap().as_str()),
            body: strip_ansi_escape(self.body),
        })
    }
}

fn find_captures(haystack: &str) -> Vec<Captures> {
    let re_subject = Regex::new(r"(?m)^\S+:\d+:\d+: warning: .*$").unwrap();
    let re_noise = Regex::new(r"(?m)^\[\d+/\d+\]").unwrap();
    let mut captures = Vec::new();
    let mut offset = 0;
    while let Some(match_subject) = re_subject.find_at(&haystack, offset) {
        // body extends from end of head to end of file, start of noise, or
        // start of next head, whichever comes first
        offset = match_subject.end();
        let a = re_subject
            .find_at(&haystack, offset)
            .map_or(haystack.len(), |m| m.start());
        let b = re_noise
            .find_at(&haystack, offset)
            .map_or(haystack.len(), |m| m.start());
        let body_end = a.min(b);

        captures.push(Captures {
            head: &haystack[match_subject.start()..match_subject.end()],
            body: &haystack[match_subject.end()..body_end]
                .trim_start_matches(|c| c == '\n')
                .trim_end_matches(|c| c == '\n'),
        });

        offset = body_end;
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
        assert_eq!(c.body, "");
    }

    #[test]
    fn test_find_captures_multiple_lines_no_noise() {
        let captures = super::find_captures("foo.c:10:20: warning: bar\nbody line 1\nbody line 2");
        assert_eq!(captures.len(), 1);
        let c = &captures[0];
        assert_eq!(c.head, "foo.c:10:20: warning: bar");
        assert_eq!(c.body, "body line 1\nbody line 2");
    }

    #[test]
    fn test_find_captures_multiple_warnings_no_noise() {
        let captures = super::find_captures(
            "foo.c:10:20: warning: bar\nfoo 1\nbar.c:30:40: warning: foo\nbar 1",
        );
        assert_eq!(captures.len(), 2);

        let c = &captures[0];
        assert_eq!(c.head, "foo.c:10:20: warning: bar");
        assert_eq!(c.body, "foo 1");

        let c = &captures[1];
        assert_eq!(c.head, "bar.c:30:40: warning: foo");
        assert_eq!(c.body, "bar 1");
    }

    #[test]
    fn test_find_captures_single_line_surrounded_by_noise() {
        let captures = super::find_captures("[1/2] foo\nfoo.c:10:20: warning: bar\n[2/2] bar");
        assert_eq!(captures.len(), 1);
        let c = &captures[0];
        assert_eq!(c.head, "foo.c:10:20: warning: bar");
        assert_eq!(c.body, "");
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
            "    private fun init(attrs: AttributeSet?, defStyle: Int) {\n                     ^"
        );
    }

    #[test]
    fn test_parse() {
        let items = super::parse("[1/2] foo\nfoo.c:10:20: warning: bar\nbody 1\nbody 2\n[2/2] bar")
            .unwrap();
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
        let items = super::parse(&contents).unwrap();
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