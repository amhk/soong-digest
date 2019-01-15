use lazy_static::lazy_static;
use regex::Regex;

pub fn strip_ansi_escape(input: &str) -> String {
    lazy_static! {
        static ref re: Regex = Regex::new(r"\u{1b}\[\d+(;\d+)*m").unwrap();
    }
    re.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_strip_ansi_escape() {
        assert_eq!(super::strip_ansi_escape(r"[1m[0m"), "");
        assert_eq!(
            super::strip_ansi_escape(
                r"[1midmap.cpp:234:5: [0m[0;1;31merror: [0m[1mfoo [-Wfoo][0m"
            ),
            "idmap.cpp:234:5: error: foo [-Wfoo]"
        );
    }
}
