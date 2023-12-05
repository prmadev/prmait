use criterion::{black_box, criterion_group, criterion_main, Criterion};

// use prmait::;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("bench time", |b| b.iter(time::OffsetDateTime::now_utc));
    c.bench_function("bench time with local", |b| {
        b.iter(|| {
            time::OffsetDateTime::now_utc().to_offset(
                time::UtcOffset::from_hms(black_box(3), black_box(30), black_box(0)).unwrap(),
            )
        })
    });

    // c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
