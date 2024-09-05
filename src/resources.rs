use bevy::prelude::*;

#[derive(Resource)]
pub struct InvaderSpeed(pub f32);

#[derive(Resource)]
pub struct InvaderTimer(pub Timer);

#[derive(Resource)]
pub struct PlayerShootTimer(pub Timer);

#[derive(Resource)]
pub struct InvaderShootTimer(pub Timer);

#[derive(Resource)]
pub enum GameState {
    Playing,
    PlayerHit,
    GameOver,
    Restarting,
}

impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (GameState::Playing, GameState::Playing) => true,
            (GameState::PlayerHit, GameState::PlayerHit) => true,
            (GameState::GameOver, GameState::GameOver) => true,
            (GameState::Restarting, GameState::Restarting) => true,
            _ => false,
        }
    }
}

#[derive(Resource)]
pub struct PlayerHitTimer(pub Timer);

#[derive(Resource)]
pub struct PlayerHitAnimationTimer(pub Timer);
