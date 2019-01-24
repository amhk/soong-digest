use crate::ansi::strip_ansi_escape;
use crate::item::{Item, ItemType};
use lazy_static::lazy_static;
use regex::Regex;

pub fn parse<'a, 'b>(haystack: &'a str) -> Result<impl Iterator<Item = Item> + 'b, String> {
    lazy_static! {
        static ref re: Regex = Regex::new(
            "(?m)^FAILED: .*\n\
             Outputs: .*\n\
             Error: .*\n\
             Command: .*\n\
             Output:\n(?s)(.*?)\n\n",
        )
        .unwrap();
    }
    let mut items = vec![];
    if haystack.is_empty() {
        return Ok(items.into_iter());
    }
    re.captures_iter(haystack)
        .try_for_each(|caps| -> Result<(), String> {
            let mut iter = parse_output(caps.get(1).unwrap().as_str())?;
            items.extend(&mut iter);
            Ok(())
        })?;
    match items.len() {
        0 => Err("failed to split input into blocks".to_string()),
        _ => Ok(items.into_iter()),
    }
}

fn parse_output<'a, 'b>(haystack: &'a str) -> Result<impl Iterator<Item = Item> + 'b, String> {
    #[derive(Debug)]
    struct InternalItem<'a> {
        path: &'a str,
        line: &'a str,
        column: Option<&'a str>,
        subject: &'a str,
        body: Vec<&'a str>,
    };
    lazy_static! {
        static ref re_with_col: Regex =
            Regex::new(r"^(\S+):(\d+):(\d+): (?:fatal )?error: (.*)").unwrap();
        static ref re_without_col: Regex =
            Regex::new(r"^(\S+):(\d+): (?:fatal )?error: (.*)").unwrap();
        static ref re_errors_generated: Regex = Regex::new(r"^\d+ errors? generated\.$").unwrap();
        static ref re_errors: Regex = Regex::new(r"^\d+ errors?$").unwrap();
    }
    let mut current: Option<InternalItem> = None;
    let mut internal_items = vec![];
    let haystack = strip_ansi_escape(haystack);
    for line in haystack
        .lines()
        .filter(|line| !line.is_empty())
        .filter(|line| !re_errors_generated.is_match(line))
        .filter(|line| !re_errors.is_match(line))
    {
        let caps = re_with_col.captures(&line);
        if caps.is_some() {
            let caps = caps.unwrap();
            if current.is_some() {
                internal_items.push(current.take().unwrap());
            }
            current = Some(InternalItem {
                path: caps.get(1).unwrap().as_str(),
                line: caps.get(2).unwrap().as_str(),
                column: Some(caps.get(3).unwrap().as_str()),
                subject: caps.get(4).unwrap().as_str(),
                body: vec![],
            });
            continue;
        }

        let caps = re_without_col.captures(&line);
        if caps.is_some() {
            let caps = caps.unwrap();
            if current.is_some() {
                internal_items.push(current.take().unwrap());
            }
            current = Some(InternalItem {
                path: caps.get(1).unwrap().as_str(),
                line: caps.get(2).unwrap().as_str(),
                column: None,
                subject: caps.get(3).unwrap().as_str(),
                body: vec![],
            });
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
        internal_items.push(current.take().unwrap());
    }

    let mut out = vec![];
    for ii in internal_items {
        out.push(Item {
            path: ii.path.to_string(),
            line: ii.line.parse().unwrap(),
            column: ii.column.map(|x| x.parse().unwrap()),
            subject: ii.subject.to_string(),
            body: ii.body.join("\n"),
            type_: ItemType::Error,
        });
    }
    match out.len() {
        0 => Err(format!("failed to parse block '{}'", &haystack)),
        _ => Ok(out.into_iter()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_java_errors() {
        let haystack = include_str!("../tests/data/easter-egg-errors-java/error.log");
        let items = super::parse(&haystack).unwrap().collect::<Vec<_>>();
        assert_eq!(items.len(), 3);

        let i = &items[0];
        assert_eq!(
            i.path,
            "frameworks/base/packages/EasterEgg/src/com/android/egg/paint/PaintActivity.java"
        );
        assert_eq!(i.line, 228);
        assert_eq!(i.column, None);
        assert_eq!(i.subject, "cannot find symbol");
        assert_eq!(i.body, "            for (int i = 0; i < NUM_BRUSHES; i++) {\n                                ^\n  symbol:   variable NUM_BRUSHES\n  location: class PaintActivity");

        let i = &items[1];
        assert_eq!(
            i.path,
            "frameworks/base/packages/EasterEgg/src/com/android/egg/paint/PaintActivity.java"
        );
        assert_eq!(i.line, 233);
        assert_eq!(i.column, None);
        assert_eq!(i.subject, "cannot find symbol");
        assert_eq!(i.body, "                        (float) Math.pow((float) i / NUM_BRUSHES, 2f), minBrushWidth,\n                                                     ^\n  symbol:   variable NUM_BRUSHES\n  location: class PaintActivity");

        let i = &items[2];
        assert_eq!(
            i.path,
            "frameworks/base/packages/EasterEgg/src/com/android/egg/paint/PaintActivity.java"
        );
        assert_eq!(i.line, 311);
        assert_eq!(i.column, None);
        assert_eq!(i.subject, "cannot find symbol");
        assert_eq!(i.body, "        thisDoesNotExist();\n        ^\n  symbol:   method thisDoesNotExist()\n  location: class PaintActivity");
    }

    #[test]
    fn test_parse_kotlin_errors() {
        let haystack = include_str!("../tests/data/easter-egg-errors-kt/error.log");
        let items = super::parse(&haystack).unwrap().collect::<Vec<_>>();
        assert_eq!(items.len(), 3);

        let i = &items[0];
        assert_eq!(
            i.path,
            "frameworks/base/packages/EasterEgg/src/com/android/egg/paint/CutoutAvoidingToolbar.kt"
        );
        assert_eq!(i.line, 48);
        assert_eq!(i.column, Some(9));
        assert_eq!(i.subject, "unresolved reference: thisDoesNotExist");
        assert_eq!(i.body, "        thisDoesNotExist()\n        ^");

        let i = &items[1];
        assert_eq!(
            i.path,
            "frameworks/base/packages/EasterEgg/src/com/android/egg/paint/CutoutAvoidingToolbar.kt"
        );
        assert_eq!(i.line, 65);
        assert_eq!(i.column, Some(35));
        assert_eq!(
            i.subject,
            "type mismatch: inferred type is Int but Char was expected"
        );
        assert_eq!(
            i.body,
            "                    cutoutRight = r.width()\n                                  ^"
        );

        let i = &items[2];
        assert_eq!(
            i.path,
            "frameworks/base/packages/EasterEgg/src/com/android/egg/paint/CutoutAvoidingToolbar.kt"
        );
        assert_eq!(i.line, 79);
        assert_eq!(i.column, Some(35));
        assert_eq!(
            i.subject,
            "none of the following functions can be called with the arguments supplied: "
        );
        assert_eq!(i.body, "public constructor LayoutParams(c: Context!, attrs: AttributeSet!) defined in android.widget.LinearLayout.LayoutParams\npublic constructor LayoutParams(width: Int, height: Int) defined in android.widget.LinearLayout.LayoutParams\n                it.layoutParams = LayoutParams(cutoutRight, MATCH_PARENT)\n                                  ^");
    }

    #[test]
    fn test_parse_cpp_errors() {
        let haystack = include_str!("../tests/data/idmap-errors/error.log");
        let items = super::parse(&haystack).unwrap().collect::<Vec<_>>();
        assert_eq!(items.len(), 3);

        let i = &items[0];
        assert_eq!(i.path, "frameworks/base/cmds/idmap/idmap.cpp");
        assert_eq!(i.line, 234);
        assert_eq!(i.column, Some(5));
        assert_eq!(
            i.subject,
            "control may reach end of non-void function [-Werror,-Wreturn-type]"
        );
        assert_eq!(i.body, "    }\n    ^");

        let i = &items[1];
        assert_eq!(i.path, "frameworks/base/cmds/idmap/create.cpp");
        assert_eq!(i.line, 29);
        assert_eq!(i.column, Some(33));
        assert_eq!(i.subject, "expected ';' after expression");
        assert_eq!(i.body, "        zip->releaseEntry(entry)\n                                ^\n                                ;");

        let i = &items[2];
        assert_eq!(i.path, "frameworks/base/cmds/idmap/create.cpp");
        assert_eq!(i.line, 89);
        assert_eq!(i.column, Some(13));
        assert_eq!(i.subject, "no matching function for call to 'lseek'");
        assert_eq!(i.body, "        if (lseek(idmap_fd, 0) < 0) {\n            ^~~~~\nbionic/libc/include/unistd.h:258:7: note: candidate function not viable: requires 3 arguments, but 2 were provided\noff_t lseek(int __fd, off_t __offset, int __whence);\n      ^");
    }

    #[test]
    fn test_parse_cpp_fatal_errors() {
        let haystack = include_str!("../tests/data/idmap-fatal-errors/error.log");
        let items = super::parse(&haystack).unwrap().collect::<Vec<_>>();
        assert_eq!(items.len(), 1);

        let i = &items[0];
        assert_eq!(i.path, "frameworks/base/cmds/idmap/create.cpp");
        assert_eq!(i.line, 2);
        assert_eq!(i.column, Some(10));
        assert_eq!(i.subject, "'does-not-exist.h' file not found");
        assert_eq!(
            i.body,
            "#include \"does-not-exist.h\"\n         ^~~~~~~~~~~~~~~~~~"
        );
    }

    #[test]
    fn test_failure_to_parse_a_block() {
        let haystack = "FAILED: some path\n\
                        Outputs: some object\n\
                        Error: some return value\n\
                        Command: some command\n\
                        Output:\n\
                        some output not recognized by the parser\n\
                        \n\
                        \n";
        let result = super::parse(&haystack);
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("failed to parse block"));
    }

    #[test]
    fn test_failure_to_parse_anything() {
        let haystack = "foo";
        let result = super::parse(&haystack);
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .contains("failed to split input into blocks"));
    }

    #[test]
    fn test_empty_input_is_ok() {
        let haystack = "";
        let items = super::parse(&haystack).unwrap();
        assert_eq!(items.count(), 0);
    }
}
