use bevy::prelude::*;

#[derive(Resource)]
pub enum InvaderDirection {
    Left,
    Right,
}

pub enum InvaderBulletType {
    Bullet,
    Bolt,
}
