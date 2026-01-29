//! Combat System Tests (TDD)
//!
//! Tests for critical hit and defense damage calculations.

use puzzle_tactics::battle::*;
use puzzle_tactics::puzzle::TileType;

// ============================================================
// Critical Hit Tests
// ============================================================

/// Test: Critical hit multiplier should be 1.5x
#[test]
fn test_critical_hit_multiplier() {
    let base_damage = 100.0;
    let crit_multiplier = DamageCalculator::crit_multiplier();

    assert!((crit_multiplier - 1.5).abs() < 0.01);
}

/// Test: Critical hit damage calculation
#[test]
fn test_critical_hit_damage() {
    let base_damage = 100.0;
    let damage_with_crit = DamageCalculator::apply_crit(base_damage, true);
    let damage_without_crit = DamageCalculator::apply_crit(base_damage, false);

    // With crit: 100 * 1.5 = 150
    assert!((damage_with_crit - 150.0).abs() < 0.01);
    // Without crit: 100 * 1.0 = 100
    assert!((damage_without_crit - 100.0).abs() < 0.01);
}

// ============================================================
// Defense Reduction Tests
// ============================================================

/// Test: Defense reduces damage by percentage (defense / 100)
#[test]
fn test_defense_percentage_reduction() {
    let base_damage = 100.0;

    // defense = 10 -> 10% reduction -> 90 damage
    let damage_10_def = DamageCalculator::apply_defense(base_damage, 10.0);
    assert!((damage_10_def - 90.0).abs() < 0.01);

    // defense = 50 -> 50% reduction -> 50 damage
    let damage_50_def = DamageCalculator::apply_defense(base_damage, 50.0);
    assert!((damage_50_def - 50.0).abs() < 0.01);
}

/// Test: Defense capped at 80% reduction maximum
#[test]
fn test_defense_cap_at_80_percent() {
    let base_damage = 100.0;

    // defense = 100 -> should cap at 80% reduction -> 20 damage
    let damage_100_def = DamageCalculator::apply_defense(base_damage, 100.0);
    assert!((damage_100_def - 20.0).abs() < 0.01);

    // defense = 200 -> still capped at 80% -> 20 damage
    let damage_200_def = DamageCalculator::apply_defense(base_damage, 200.0);
    assert!((damage_200_def - 20.0).abs() < 0.01);
}

/// Test: Zero defense means no reduction
#[test]
fn test_zero_defense_no_reduction() {
    let base_damage = 100.0;
    let damage_zero_def = DamageCalculator::apply_defense(base_damage, 0.0);

    assert!((damage_zero_def - 100.0).abs() < 0.01);
}

// ============================================================
// Full Damage Calculation Tests
// ============================================================

/// Test: Full damage calculation with crit and defense
#[test]
fn test_full_damage_calculation() {
    // base_damage = 100, crit = true, defense = 20
    // crit: 100 * 1.5 = 150
    // defense: 150 * (1 - 0.2) = 150 * 0.8 = 120
    let result = DamageCalculator::calculate(100.0, true, 20.0);
    assert!((result - 120.0).abs() < 0.01);
}

/// Test: Damage calculation without crit
#[test]
fn test_damage_calculation_no_crit() {
    // base_damage = 100, crit = false, defense = 20
    // no crit: 100 * 1.0 = 100
    // defense: 100 * (1 - 0.2) = 100 * 0.8 = 80
    let result = DamageCalculator::calculate(100.0, false, 20.0);
    assert!((result - 80.0).abs() < 0.01);
}

/// Test: Minimum damage is 1.0 (never 0)
#[test]
fn test_minimum_damage_is_one() {
    // Even with max defense, damage should be at least 1
    let result = DamageCalculator::calculate(1.0, false, 100.0);
    assert!(result >= 1.0);
}

// ============================================================
// Integration Tests with UnitStats
// ============================================================

/// Test: UnitStats damage calculation uses new formula
#[test]
fn test_unit_stats_take_damage_with_defense() {
    let mut defender = UnitStats::for_type(TileType::Blue, 1);
    defender.defense = 50.0; // 50% reduction
    let initial_health = defender.health;

    // Apply 100 damage -> should reduce to 50 after defense
    defender.take_calculated_damage(100.0);

    let expected_health = initial_health - 50.0;
    assert!((defender.health - expected_health).abs() < 0.01);
}
