use super::*;

#[test]
fn statistics_preserve_the_tail_and_count_each_over_budget_tick() {
    let values = (1..=100).map(|ms| Duration::from_millis(ms)).collect();
    let summary = Timing::from_samples(values);
    assert_eq!(
        (
            summary.p50_us,
            summary.p95_us,
            summary.p99_us,
            summary.max_us
        ),
        (50_000, 95_000, 99_000, 100_000)
    );
    assert_eq!(summary.over_tick_budget, 84);
}

#[test]
fn measured_window_excludes_warmup_and_exercises_eight_fighters() {
    let result = profile(180, 300);
    assert_eq!(result.total.samples, 300);
    assert_eq!(result.publication.samples, 150);
    assert_eq!(result.playing_ticks, 300);
    assert_eq!(result.inputs, 50 * 8);
    assert!(result.snapshot_bytes.iter().all(|b| *b > 0));
    assert_eq!(result.snapshot_bytes, [result.snapshot_bytes[0]; 8]);
    assert!(result.grip_ticks > 0 || result.hits > 0);
}
