use bevy::prelude::*;

use crate::enums::InvaderBulletType;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub lives: u32,
    pub score: u32,
    pub id: PlayerID,
    pub is_hit: bool,
}

#[derive(Component)]
pub struct PlayerID {
    pub id: u32,
}

impl Clone for PlayerID {
    fn clone(&self) -> Self {
        PlayerID { id: self.id }
    }
}

#[derive(Component)]
pub struct ShieldPart {
    pub health: u32,
    pub textures: Vec<Handle<Image>>,
    pub x: f32,
    pub y: f32,
}

impl ShieldPart {
    pub fn current_texture(&self) -> Handle<Image> {
        self.textures[self.health as usize - 1].clone()
    }
}

#[derive(Component)]
pub struct Invader {}

#[derive(Component)]
pub struct Shooter {}

#[derive(Component)]
pub struct Bullet {
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct PlayerBullet {
    pub player_id: PlayerID,
}

#[derive(Component)]
pub struct InvaderBullet {
    pub bullet_type: InvaderBulletType,
}

pub struct InvaderBulletProbability {
    pub bullet_type: InvaderBulletType,
    pub probability: f32,
}

#[derive(Component)]
pub struct PlayerScore {
    pub player_id: PlayerID,
}

#[derive(Component)]
pub struct PlayerLife {
    pub player_id: PlayerID,
}

#[derive(Component)]
pub struct GameOverText {}

#[derive(Component)]
pub struct GameRestartButton {}

#[derive(Component)]
pub struct GameRestartText {}
