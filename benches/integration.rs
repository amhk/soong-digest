use criterion::{criterion_group, criterion_main, Criterion};
use std::env;
use std::process::{Command, Output};
use std::time::Duration;

fn exec(arg: &str) -> Output {
    let root = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let bin = root.join("../soong-digest");
    Command::new(bin).arg(arg).output().unwrap()
}

fn bench_errors(c: &mut Criterion) {
    c.bench_function("bench-errors", |b| {
        b.iter(|| {
            let o = exec("--errors=tests/data/idmap-both-errors-and-warnings/error.log");
            assert!(o.status.success());
        })
    });
}

fn bench_warnings(c: &mut Criterion) {
    c.bench_function("bench-warnings", |b| {
        b.iter(|| {
            let o = exec("--warnings=tests/data/idmap-both-errors-and-warnings/verbose.log.gz");
            assert!(o.status.success());
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::new(10, 0));
    targets = bench_errors, bench_warnings
}
criterion_main!(benches);
