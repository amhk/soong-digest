use std::env;
use std::process::{Command, Output};

fn exec(arg: &str) -> Output {
    let root = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let bin = root.join("../soong-digest");
    Command::new(bin).arg(arg).output().unwrap()
}

#[test]
fn test_parse_all_error_data() {
    let o0 = exec("--errors=this-does-not-exist");
    assert!(!o0.status.success());

    let o1 = exec("--errors=tests/data/easter-egg-errors-kt/error.log");
    assert!(o1.status.success());

    let o2 = exec("--errors=tests/data/easter-egg-errors-java/error.log");
    assert!(o2.status.success());

    let o3 = exec("--errors=tests/data/idmap-warnings/error.log");
    assert!(o3.status.success());

    let o4 = exec("--errors=tests/data/idmap-both-errors-and-warnings/error.log");
    assert!(o4.status.success());

    let o5 = exec("--errors=tests/data/idmap-errors/error.log");
    assert!(o5.status.success());
}

#[test]
#[ignore]
fn test_parse_all_warning_data() {
    let o0 = exec("--warnings=this-does-not-exist");
    assert!(!o0.status.success());

    let o1 = exec("--warnings=tests/data/easter-egg-errors-kt/verbose.log.gz");
    assert!(o1.status.success());

    let o2 = exec("--warnings=tests/data/easter-egg-errors-java/verbose.log.gz");
    assert!(o2.status.success());

    let o3 = exec("--warnings=tests/data/idmap-warnings/verbose.log.gz");
    assert!(o3.status.success());

    let o4 = exec("--warnings=tests/data/idmap-both-errors-and-warnings/verbose.log.gz");
    assert!(o4.status.success());

    let o5 = exec("--warnings=tests/data/idmap-errors/verbose.log.gz");
    assert!(o5.status.success());
}
