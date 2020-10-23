use benchmark_sampledata;
use criterion::*;
use diskspace_insight::scan;
use env_logger;
use log::*;
use std::fs::File;

fn test_scan(c: &mut Criterion) {
    std::env::set_var("RUST_LOG", "INFO");
    let _ = env_logger::builder().try_init();
    // One-time setup code goes here

    let sd = benchmark_sampledata::linux_kernel().unwrap();
    info!("{:?}", sd);

    c.bench_function("scan kernel sources", |b| {
        b.iter(|| {
            // Measured code goes here
            scan(&sd.root)
        });
    });

    sd.remove();
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default()
    .sample_size(10)
    .warm_up_time(std::time::Duration::from_secs(2))
    .measurement_time(std::time::Duration::from_secs(3))
    ;
    targets = test_scan
}
criterion_main!(benches);
