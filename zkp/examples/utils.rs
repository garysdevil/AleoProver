pub fn time_spend(f: fn()) {
    let start = std::time::Instant::now();
    f();
    let duration = start.elapsed();
    println!("{}. Total time elapsed  {:?}", "-", duration);
}
