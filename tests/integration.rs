use std::env;
use std::process::{Command, Output};

fn exec(arg: &str) -> Output {
    let root = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let bin = root.join("../soong-digest");
    Command::new(bin).arg(arg).output().unwrap()
}

#[test]
fn test_parse_all_error_data() {
    let o = exec("--errors=this-does-not-exist");
    assert!(!o.status.success());

    let o = exec("--errors=tests/data/easter-egg-errors-kt/error.log");
    assert!(o.status.success());

    let o = exec("--errors=tests/data/easter-egg-errors-java/error.log");
    assert!(o.status.success());

    let o = exec("--errors=tests/data/idmap-warnings/error.log");
    assert!(o.status.success());

    let o = exec("--errors=tests/data/idmap-both-errors-and-warnings/error.log");
    assert!(o.status.success());

    let o = exec("--errors=tests/data/idmap-errors/error.log");
    assert!(o.status.success());

    let o = exec("--errors=tests/data/idmap-identical-errors/error.log");
    assert!(o.status.success());

    let o = exec("--errors=tests/data/idmap-fatal-errors/error.log");
    assert!(o.status.success());
}

#[test]
fn test_parse_quick_warning_data() {
    let o = exec("--warnings=this-does-not-exist");
    assert!(!o.status.success());

    let o = exec("--warnings=tests/data/easter-egg-errors-kt/verbose.log.gz");
    assert!(o.status.success());

    let o = exec("--warnings=tests/data/easter-egg-errors-java/verbose.log.gz");
    assert!(o.status.success());

    let o = exec("--warnings=tests/data/idmap-identical-errors/verbose.log.gz");
    assert!(o.status.success());

    let o = exec("--warnings=tests/data/idmap-fatal-errors/verbose.log.gz");
    assert!(o.status.success());
}

#[test]
#[ignore]
fn test_parse_slow_warning_data() {
    let o = exec("--warnings=tests/data/idmap-warnings/verbose.log.gz");
    assert!(o.status.success());

    let o = exec("--warnings=tests/data/idmap-both-errors-and-warnings/verbose.log.gz");
    assert!(o.status.success());

    let o = exec("--warnings=tests/data/idmap-errors/verbose.log.gz");
    assert!(o.status.success());
}
