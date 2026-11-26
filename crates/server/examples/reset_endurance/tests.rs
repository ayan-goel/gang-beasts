use super::*;

#[test]
fn eight_players_can_fight_score_and_rebuild_a_clean_round() {
    let report = run(1);
    assert_eq!(report.round_resets, 1);
    assert_eq!(report.scored_rounds, 1);
    assert!(report.grip_ticks > 0);
    assert!(report.hits > 0);
    assert!(report.verified_snapshots > 0);
}
