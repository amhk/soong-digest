use criterion::{criterion_group, criterion_main, Criterion};
use std::env;
use std::process::{Command, Output};

fn exec(arg: &str) -> Output {
    let root = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let bin = root.join("../soong-digest");
    Command::new(bin).arg(arg).output().unwrap()
}

fn bench_errors(c: &mut Criterion) {
    c.bench_function("bench-errors", |b| {
        b.iter(|| {
            exec("--errors=tests/data/idmap-both-errors-and-warnings/error.log");
        })
    });
}

fn bench_warnings(c: &mut Criterion) {
    c.bench_function("bench-warnings", |b| {
        b.iter(|| {
            exec("--warnings=tests/data/idmap-both-errors-and-warnings/verbose.log.gz");
        })
    });
}

criterion_group!(benches, bench_errors, bench_warnings);
criterion_main!(benches);
