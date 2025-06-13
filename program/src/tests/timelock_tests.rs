use crate::state::Escrow;

#[test]
fn test_timelock_ordering() {
    let base_time = 1_700_000_000u64;
    let timelocks = [
        0,      // src_exclusive_withdraw (already passed)
        0,      // src_public_withdraw (already passed)
        300,    // dst_exclusive_withdraw (5 minutes)
        600,    // dst_public_withdraw (10 minutes)
        900,    // dst_exclusive_cancel (15 minutes)
        1200,   // dst_public_cancel (20 minutes)
        86400,  // rescue (24 hours)
    ];
    
    // Verify timelocks are in ascending order (excluding already-passed ones)
    for i in 2..timelocks.len() - 1 {
        assert!(
            timelocks[i] < timelocks[i + 1],
            "Timelock {} should be less than timelock {}",
            i,
            i + 1
        );
    }
}

#[test]
fn test_timelock_calculations() {
    let deployed_at = 1_700_000_000u64;
    let timelocks = [0, 0, 300, 600, 900, 1200, 86400];
    
    // Calculate absolute times
    let exclusive_withdraw = deployed_at + timelocks[Escrow::TL_DST_EXCLUSIVE_WITHDRAW];
    let public_withdraw = deployed_at + timelocks[Escrow::TL_DST_PUBLIC_WITHDRAW];
    let exclusive_cancel = deployed_at + timelocks[Escrow::TL_DST_EXCLUSIVE_CANCEL];
    let public_cancel = deployed_at + timelocks[Escrow::TL_DST_PUBLIC_CANCEL];
    let rescue = deployed_at + timelocks[Escrow::TL_RESCUE];
    
    assert_eq!(exclusive_withdraw, deployed_at + 300);
    assert_eq!(public_withdraw, deployed_at + 600);
    assert_eq!(exclusive_cancel, deployed_at + 900);
    assert_eq!(public_cancel, deployed_at + 1200);
    assert_eq!(rescue, deployed_at + 86400);
}

#[test]
fn test_timelock_overflow_safety() {
    let deployed_at = u64::MAX - 1000;
    let timelocks = [0, 0, 300, 600, 900, 1200, 86400];
    
    // This would overflow without checked arithmetic
    let result = deployed_at.checked_add(timelocks[Escrow::TL_RESCUE]);
    assert!(result.is_none(), "Should detect overflow");
}