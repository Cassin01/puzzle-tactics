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
