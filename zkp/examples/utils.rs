pub fn time_spend(comment: &str, f: fn()) {
    let start = std::time::Instant::now();
    f();
    let duration = start.elapsed();
    println!("{}. Total time elapsed  {:?}", comment, duration);
}
