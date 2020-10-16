use criterion::{black_box, criterion_group, criterion_main, Criterion, Benchmark};
use diskspace_insight::scan;
use dirs;




fn scan_home(c: &mut Criterion) {
    std::env::set_var("RUST_LOG", "INFO");
    let _ = env_logger::builder().try_init();

    let home = dirs::home_dir().unwrap();
    //let home = "/home/woelper/Downloads";
    c.bench_function("Scan home folder", |b| {
        // Per-sample (note that a sample can be many iterations) setup goes here
        b.iter(|| {
            // Measured code goes here
            scan(&home)
        });
    });


}


criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default()
    .sample_size(10)
    .warm_up_time(std::time::Duration::from_secs(2))
    .measurement_time(std::time::Duration::from_secs(3))
    ;
    targets = scan_home,
}
criterion_main!(benches);
