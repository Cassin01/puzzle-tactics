use bevy::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
    GameOver,
}

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PhaseState {
    #[default]
    Idle,
    Matching,
    Cascading,
    Combating,
}

#[derive(Resource, Default)]
pub struct ComboCounter {
    pub current: u32,
    pub max_this_turn: u32,
}

impl ComboCounter {
    pub fn increment(&mut self) {
        self.current += 1;
        if self.current > self.max_this_turn {
            self.max_this_turn = self.current;
        }
    }

    pub fn reset(&mut self) {
        self.current = 0;
        self.max_this_turn = 0;
    }
}

// ============================================================
// Time Scale System (Slow Motion)
// ============================================================

/// Resource for controlling game time scale (slow motion effects)
#[derive(Resource)]
pub struct TimeScale {
    /// Current time scale (1.0 = normal, 0.3 = slow motion)
    pub scale: f32,
    /// Timer for slow motion duration
    pub duration: Timer,
    /// Target scale to return to after slow motion ends
    pub target: f32,
    /// Whether slow motion is currently active
    pub active: bool,
}

impl Default for TimeScale {
    fn default() -> Self {
        Self {
            scale: 1.0,
            duration: Timer::from_seconds(0.0, TimerMode::Once),
            target: 1.0,
            active: false,
        }
    }
}

impl TimeScale {
    /// Trigger slow motion effect
    pub fn trigger_slowmo(&mut self, scale: f32, duration_secs: f32) {
        self.scale = scale;
        self.duration = Timer::from_seconds(duration_secs, TimerMode::Once);
        self.target = 1.0;
        self.active = true;
    }

    /// Update the time scale (call every frame)
    pub fn update(&mut self, delta_secs: f32) {
        if !self.active {
            return;
        }

        self.duration.tick(std::time::Duration::from_secs_f32(delta_secs));

        if self.duration.finished() {
            self.scale = self.target;
            self.active = false;
        }
    }

    /// Get the current effective delta time
    pub fn scaled_delta(&self, delta_secs: f32) -> f32 {
        delta_secs * self.scale
    }
}

/// Event to trigger slow motion effect
#[derive(Event)]
pub struct SlowMoEvent {
    pub scale: f32,
    pub duration: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timescale_default_is_one() {
        let ts = TimeScale::default();
        assert!((ts.scale - 1.0).abs() < f32::EPSILON, "Default scale should be 1.0");
        assert!(!ts.active, "Default should not be active");
    }

    #[test]
    fn test_slowmo_sets_scale() {
        let mut ts = TimeScale::default();
        ts.trigger_slowmo(0.3, 1.0);
        assert!((ts.scale - 0.3).abs() < f32::EPSILON, "Scale should be 0.3 after trigger");
        assert!(ts.active, "Should be active after trigger");
    }

    #[test]
    fn test_slowmo_returns_to_normal() {
        let mut ts = TimeScale::default();
        ts.trigger_slowmo(0.3, 0.5);

        // Simulate time passing (more than duration)
        ts.update(0.6);

        assert!((ts.scale - 1.0).abs() < f32::EPSILON, "Scale should return to 1.0 after duration");
        assert!(!ts.active, "Should not be active after duration ends");
    }

    #[test]
    fn test_scaled_delta() {
        let mut ts = TimeScale::default();
        ts.trigger_slowmo(0.5, 1.0);

        let delta = 0.016; // ~60fps
        let scaled = ts.scaled_delta(delta);
        assert!((scaled - 0.008).abs() < f32::EPSILON, "Scaled delta should be half");
    }
}
