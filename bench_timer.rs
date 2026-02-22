use std::time::Instant;

fn main() {
    let start = Instant::now();
    let iter = 1_000_000;
    for _ in 0..iter {
        let _ = Instant::now();
    }
    let duration = start.elapsed();
    println!("Avg Instant::now() overhead: {:.2} ns", (duration.as_nanos() as f64) / (iter as f64));
}
