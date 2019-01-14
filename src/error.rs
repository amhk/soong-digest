use crate::ansi::strip_ansi_escape;
use crate::item::Item;
use regex::Regex;
use std::error::Error;

pub fn parse<'a, 'b>(haystack: &'a str) -> Result<impl Iterator<Item = Item> + 'b, Box<Error>> {
    let mut items: Vec<Item> = Vec::new();
    let captures = find_captures(haystack);
    for c in captures {
        match c.try_from() {
            Ok(item) => items.push(item),
            Err(e) => return Err(e),
        }
    }
    Ok(items.into_iter())
}

struct Captures<'h> {
    _failed: &'h str,
    _outputs: &'h str,
    _error: &'h str,
    _command: &'h str,
    output: &'h str,
}

impl<'h> Captures<'h> {
    // switch to std::convert::TryFrom once that becomes stable
    fn try_from(&self) -> Result<Item, Box<Error>> {
        let re_with_column =
            Regex::new(r"(?P<path>\S+):(?P<line>\d+):(?P<column>\d+): (?P<subject>.*)").unwrap();
        let re_without_column =
            Regex::new(r"(?P<path>\S+):(?P<line>\d+): (?P<subject>.*)").unwrap();
        let caps = match re_with_column.captures(self.output) {
            Some(caps) => caps,
            None => match re_without_column.captures(self.output) {
                Some(caps) => caps,
                None => return Err(From::from(format!("failed to parse '{}'", self.output))),
            },
        };
        let line_str = caps.name("line").unwrap().as_str();
        let line = match line_str.parse() {
            Ok(line) => line,
            Err(_) => {
                return Err(From::from(format!(
                    "failed to convert '{}' to int",
                    line_str
                )))
            }
        };
        let column = match caps.name("column") {
            Some(s) => match s.as_str().parse() {
                Ok(i) => Some(i),
                Err(_) => {
                    return Err(From::from(format!(
                        "failed to convert '{}' to int",
                        s.as_str()
                    )))
                }
            },
            None => None,
        };
        let item = Item {
            path: strip_ansi_escape(caps.name("path").unwrap().as_str()),
            line,
            column,
            subject: strip_ansi_escape(caps.name("subject").unwrap().as_str()),
            body: strip_ansi_escape(self.output)
                .lines()
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n"),
        };
        Ok(item)
    }
}

fn find_captures(haystack: &str) -> Vec<Captures> {
    let re = Regex::new(
        "(?m)^FAILED: (.*)\n\
         Outputs: (.*)\n\
         Error: (.*)\n\
         Command: (.*)\n\
         Output:\n(?s)(.*?)\n\n",
    )
    .unwrap();
    let mut captures = Vec::new();
    re.captures_iter(haystack).for_each(|caps| {
        let e = Captures {
            _failed: caps.get(1).unwrap().as_str(),
            _outputs: caps.get(2).unwrap().as_str(),
            _error: caps.get(3).unwrap().as_str(),
            _command: caps.get(4).unwrap().as_str(),
            output: caps.get(5).unwrap().as_str(),
        };
        captures.push(e);
    });
    captures
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_find_captures() {
        let haystack = include_str!("../tests/data/idmap-errors/error.log");
        let captures = super::find_captures(&haystack);
        assert_eq!(captures.len(), 2);
        let c = &captures[0];
        assert!(c._failed.contains("idmap:idmap clang++ idmap.cpp"));
        assert!(c._outputs.contains("frameworks/base/cmds/idmap/idmap.o"));
        assert!(c._error.contains("exited with code: 1"));
        assert!(c._command.contains("clang++"));
        assert!(c.output.contains("idmap.cpp:234:5"));
        assert!(c.output.contains("control may reach end of non-void"));
    }

    #[test]
    fn test_parse_with_column() {
        let haystack = include_str!("../tests/data/idmap-errors/error.log");
        let items = super::parse(&haystack).unwrap().collect::<Vec<_>>();
        assert_eq!(items.len(), 2);
        let i = &items[0];
        assert_eq!(i.path, "frameworks/base/cmds/idmap/idmap.cpp");
        assert_eq!(i.line, 234);
        assert_eq!(i.column, Some(5));
        assert!(i.subject.contains("control may reach end of non-void"));
        assert_eq!(i.body, "    }\n    ^\n1 error generated.");
    }

    #[test]
    fn test_parse_without_column() {
        let haystack = include_str!("../tests/data/easter-egg-errors-java/error.log");
        let items = super::parse(&haystack).unwrap().collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        let i = &items[0];
        assert_eq!(
            i.path,
            "frameworks/base/packages/EasterEgg/src/com/android/egg/paint/PaintActivity.java"
        );
        assert_eq!(i.line, 228);
        assert_eq!(i.column, None);
        assert!(i.subject.contains("cannot find symbol"));
        assert!(i.body.contains("variable NUM_BRUSHES"));
    }
}
