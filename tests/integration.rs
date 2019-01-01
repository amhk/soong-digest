use std::env;
use std::process::{Command, Output};

fn exec(arg: &str) -> Output {
    let root = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let bin = root.join("../soong-digest");
    Command::new(bin).arg(arg).output().unwrap()
}

#[test]
fn test_parse_all_error_data() {
    let o0 = exec("this-does-not-exist");
    assert!(!o0.status.success());

    let o1 = exec("tests/data/easter-egg-errors-kt/error.log");
    assert!(o1.status.success());

    let o2 = exec("tests/data/easter-egg-errors-java/error.log");
    assert!(o2.status.success());

    let o3 = exec("tests/data/idmap-warnings/error.log");
    assert!(o3.status.success());

    let o4 = exec("tests/data/idmap-both-errors-and-warnings/error.log");
    assert!(o4.status.success());

    let o5 = exec("tests/data/idmap-errors/error.log");
    assert!(o5.status.success());
}
