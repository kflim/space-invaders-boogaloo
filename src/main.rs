mod components;
mod enums;
mod resources;

use bevy::{prelude::*, render::camera::ScalingMode, window::EnabledButtons};
use components::{
    Bullet, GameOverText, GameRestartButton, GameRestartText, Invader, InvaderBullet,
    InvaderBulletProbability, Player, PlayerBullet, PlayerID, PlayerLife, PlayerScore, ShieldPart,
    Shooter,
};
use enums::{InvaderBulletType, InvaderDirection};
use rand::Rng;
use resources::{
    GameState, InvaderShootTimer, InvaderSpeed, InvaderTimer, PlayerHitAnimationTimer,
    PlayerHitTimer, PlayerShootTimer, RespawningInvadersTimer,
};

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;

const SHIELD_NUMS: usize = 4;

const INVADER_SPEED: f32 = 250.0;
const INVADER_COLS: usize = 11;
const INVADER_DECOY_ROWS: usize = 2;
const INVADER_SHOOTER_ROWS: usize = 1;
const INVADER_SHOOT_PROBABILITY: f32 = 0.10;

const INVADER_BULLET_PROBABILITIES: &'static [InvaderBulletProbability] = &[
    InvaderBulletProbability {
        bullet_type: InvaderBulletType::Bullet,
        probability: 0.75,
    },
    InvaderBulletProbability {
        bullet_type: InvaderBulletType::Bolt,
        probability: 0.25,
    },
];

// TODO: Refactor magic numbers and update enemy bullets, then add special enemy bullets, rare enemies, power-ups, and bosses
fn main() {
    App::new()
        .insert_resource(PlayerShootTimer(Timer::from_seconds(0.5, TimerMode::Once)))
        .insert_resource(InvaderTimer(Timer::from_seconds(
            0.85,
            TimerMode::Repeating,
        )))
        .insert_resource(InvaderShootTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .insert_resource(GameState::Playing)
        .insert_resource(PlayerHitTimer(Timer::from_seconds(1.5, TimerMode::Once)))
        .insert_resource(PlayerHitAnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .insert_resource(RespawningInvadersTimer(Timer::from_seconds(
            1.0,
            TimerMode::Once,
        )))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Space Invaders".into(),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        resizable: false,
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        enabled_buttons: EnabledButtons {
                            close: true,
                            minimize: false,
                            maximize: false,
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .build(),
        )
        .add_systems(
            Startup,
            (
                setup_camera,
                (setup_player, setup_player_score, setup_player_lives).chain(),
                setup_shields,
                setup_invaders,
            ),
        )
        .add_systems(
            Update,
            (
                players_movement,
                invaders_movement,
                spawn_player_bullets,
                invaders_shooting,
                (
                    bullets_movement,
                    bullet_collision_detection,
                    update_player_score,
                    update_player_lives,
                    check_if_invaders_defeated,
                )
                    .chain(),
            )
                .run_if(resource_exists_and_equals(GameState::Playing)),
        )
        .add_systems(
            Update,
            player_hit_animation.run_if(resource_exists_and_equals(GameState::PlayerHit)),
        )
        .add_systems(
            Update,
            ((game_over, play_again).chain())
                .run_if(resource_exists_and_equals(GameState::GameOver)),
        )
        .add_systems(
            Update,
            (
                (setup_player, setup_player_score, setup_player_lives).chain(),
                setup_shields,
                setup_invaders,
            )
                .run_if(resource_exists_and_equals(GameState::Restarting)),
        )
        .add_systems(
            Update,
            respawn_invaders.run_if(resource_exists_and_equals(GameState::RespawningInvaders)),
        )
        .add_systems(
            Update,
            pause_game.run_if(resource_exists_and_equals(GameState::Pausing)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();

    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 256.0,
        min_height: 144.0,
    };

    commands.spawn(camera);
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_texture: Handle<Image> = asset_server.load("player.png");

    commands.spawn((
        SpriteBundle {
            texture: player_texture,
            transform: Transform::from_translation(Vec3::new(-77.75, -75.0, 0.0)),
            ..Default::default()
        },
        Player {
            speed: 100.0,
            lives: 3,
            score: 0,
            id: PlayerID {
                id: rand::thread_rng().gen(),
            },
            is_hit: false,
        },
    ));
}

fn players_movement(
    mut players: Query<(&mut Transform, &Player)>,
    window: Query<&Window>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut window_width = WINDOW_WIDTH;

    for window in window.iter() {
        window_width = window.width();
    }

    for (mut transform, player) in &mut players {
        let movement_amount = player.speed * time.delta_seconds();

        if input.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= movement_amount;

            if transform.translation.x < -(window_width / 4.0) + 50.0 {
                transform.translation.x = -(window_width / 4.0) + 50.0;
            }
        }
        if input.pressed(KeyCode::ArrowRight) {
            transform.translation.x += movement_amount;

            if transform.translation.x > (window_width / 4.0) - 50.0 {
                transform.translation.x = (window_width / 4.0) - 50.0;
            }
        }
    }
}

fn setup_shields(mut commands: Commands, asset_server: Res<AssetServer>) {
    let left_shield_textures: Vec<(Vec<Handle<Image>>, Vec2)> = vec![
        (
            vec![
                asset_server.load("heavily_damaged_shield_corner_2.png"),
                asset_server.load("badly_damaged_shield_corner_2.png"),
                asset_server.load("slightly_damaged_shield_corner_2.png"),
                asset_server.load("undamaged_shield_corner_2.png"),
            ],
            Vec2::new(-83.75, -40.0),
        ),
        (
            vec![
                asset_server.load("heavily_damaged_shield.png"),
                asset_server.load("badly_damaged_shield.png"),
                asset_server.load("slightly_damaged_shield.png"),
                asset_server.load("undamaged_shield.png"),
            ],
            Vec2::new(-83.75, -46.0),
        ),
        (
            vec![
                asset_server.load("heavily_damaged_shield.png"),
                asset_server.load("badly_damaged_shield.png"),
                asset_server.load("slightly_damaged_shield.png"),
                asset_server.load("undamaged_shield.png"),
            ],
            Vec2::new(-83.75, -52.0),
        ),
    ];

    let left_center_shield_textures: Vec<(Vec<Handle<Image>>, Vec2)> = vec![
        (
            vec![
                asset_server.load("heavily_damaged_shield.png"),
                asset_server.load("badly_damaged_shield.png"),
                asset_server.load("slightly_damaged_shield.png"),
                asset_server.load("undamaged_shield.png"),
            ],
            Vec2::new(-77.75, -40.0),
        ),
        (
            vec![
                asset_server.load("heavily_damaged_shield_corner_1.png"),
                asset_server.load("badly_damaged_shield_corner_1.png"),
                asset_server.load("slightly_damaged_shield_corner_1.png"),
                asset_server.load("undamaged_shield_corner_1.png"),
            ],
            Vec2::new(-77.75, -46.0),
        ),
    ];

    let right_center_shield_textures: Vec<(Vec<Handle<Image>>, Vec2)> = vec![
        (
            vec![
                asset_server.load("heavily_damaged_shield.png"),
                asset_server.load("badly_damaged_shield.png"),
                asset_server.load("slightly_damaged_shield.png"),
                asset_server.load("undamaged_shield.png"),
            ],
            Vec2::new(-71.75, -40.0),
        ),
        (
            vec![
                asset_server.load("heavily_damaged_shield_corner_1.png"),
                asset_server.load("badly_damaged_shield_corner_1.png"),
                asset_server.load("slightly_damaged_shield_corner_1.png"),
                asset_server.load("undamaged_shield_corner_1.png"),
            ],
            Vec2::new(-71.75, -46.0),
        ),
    ];

    let right_shield_textures: Vec<(Vec<Handle<Image>>, Vec2)> = vec![
        (
            vec![
                asset_server.load("heavily_damaged_shield_corner_2.png"),
                asset_server.load("badly_damaged_shield_corner_2.png"),
                asset_server.load("slightly_damaged_shield_corner_2.png"),
                asset_server.load("undamaged_shield_corner_2.png"),
            ],
            Vec2::new(-65.75, -40.0),
        ),
        (
            vec![
                asset_server.load("heavily_damaged_shield.png"),
                asset_server.load("badly_damaged_shield.png"),
                asset_server.load("slightly_damaged_shield.png"),
                asset_server.load("undamaged_shield.png"),
            ],
            Vec2::new(-65.75, -46.0),
        ),
        (
            vec![
                asset_server.load("heavily_damaged_shield.png"),
                asset_server.load("badly_damaged_shield.png"),
                asset_server.load("slightly_damaged_shield.png"),
                asset_server.load("undamaged_shield.png"),
            ],
            Vec2::new(-65.75, -52.0),
        ),
    ];

    for i in 0..SHIELD_NUMS {
        for (texture_handles, offset) in &left_shield_textures {
            commands.spawn((
                ShieldPart {
                    health: 4,
                    textures: texture_handles.clone(),
                    x: offset.x + (i * 50) as f32,
                    y: offset.y,
                },
                SpriteBundle {
                    texture: texture_handles[3].clone(),
                    transform: Transform::from_translation(Vec3::new(
                        offset.x + (i * 50) as f32,
                        offset.y,
                        0.0,
                    )),
                    ..Default::default()
                },
            ));
        }

        for (texture_handles, offset) in &left_center_shield_textures {
            commands.spawn((
                ShieldPart {
                    health: 4,
                    textures: texture_handles.clone(),
                    x: offset.x + (i * 50) as f32,
                    y: offset.y,
                },
                SpriteBundle {
                    texture: texture_handles[3].clone(),
                    transform: Transform::from_translation(Vec3::new(
                        offset.x + (i * 50) as f32,
                        offset.y,
                        0.0,
                    )),
                    ..Default::default()
                },
            ));
        }

        for (texture_handles, offset) in &right_center_shield_textures {
            commands.spawn((
                ShieldPart {
                    health: 4,
                    textures: texture_handles.clone(),
                    x: offset.x + (i * 50) as f32,
                    y: offset.y,
                },
                SpriteBundle {
                    texture: texture_handles[3].clone(),
                    transform: Transform {
                        translation: Vec3::new(offset.x + (i * 50) as f32, offset.y, 0.0),
                        scale: Vec3::new(-1.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ));
        }

        for (texture_handles, offset) in &right_shield_textures {
            commands.spawn((
                ShieldPart {
                    health: 4,
                    textures: texture_handles.clone(),
                    x: offset.x + (i * 50) as f32,
                    y: offset.y,
                },
                SpriteBundle {
                    texture: texture_handles[3].clone(),
                    transform: Transform {
                        translation: Vec3::new(offset.x + (i * 50) as f32, offset.y, 0.0),
                        scale: Vec3::new(-1.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ));
        }
    }
}

fn setup_invaders(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bottom_invader_texture = asset_server.load("invader-1.png");
    let middle_invader_texture = asset_server.load("invader-2.png");
    let top_invader_texture = asset_server.load("invader-3.png");

    for row in 0..INVADER_DECOY_ROWS {
        for col in 0..INVADER_COLS {
            commands.spawn((
                SpriteBundle {
                    texture: bottom_invader_texture.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        -80.0 + col as f32 * 16.0,
                        6.0 + row as f32 * 16.0,
                        0.0,
                    )),
                    ..Default::default()
                },
                Invader {},
            ));

            commands.spawn((
                SpriteBundle {
                    texture: middle_invader_texture.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        -80.0 + col as f32 * 16.0,
                        36.0 + row as f32 * 16.0,
                        0.0,
                    )),
                    ..Default::default()
                },
                Invader {},
            ));
        }
    }

    for row in 0..INVADER_SHOOTER_ROWS {
        for col in 0..INVADER_COLS {
            commands.spawn((
                SpriteBundle {
                    texture: top_invader_texture.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        -80.0 + col as f32 * 16.0,
                        68.0 + row as f32 * 16.0,
                        0.0,
                    )),
                    ..Default::default()
                },
                Invader {},
                Shooter {},
            ));
        }
    }

    commands.insert_resource(InvaderDirection::Right);
    commands.insert_resource(InvaderSpeed(INVADER_SPEED));
}

fn invaders_movement(
    mut invaders: Query<&mut Transform, With<Invader>>,
    mut direction: ResMut<InvaderDirection>,
    speed: Res<InvaderSpeed>,
    time: Res<Time>,
    mut timer: ResMut<InvaderTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let (dx, _) = match *direction {
            InvaderDirection::Left => (-1.0, 0.0),
            InvaderDirection::Right => (1.0, 0.0),
        };

        let mut change_direction = false;

        for mut transform in invaders.iter_mut() {
            transform.translation.x += dx * speed.0 * time.delta_seconds();

            if transform.translation.x.abs() >= (WINDOW_WIDTH / 2.0) - 11.0 * 18.5 {
                change_direction = true;
            }
        }

        if change_direction {
            *direction = match *direction {
                InvaderDirection::Left => InvaderDirection::Right,
                InvaderDirection::Right => InvaderDirection::Left,
            };

            for mut transform in invaders.iter_mut() {
                transform.translation.x -= dx * speed.0 * time.delta_seconds();
                transform.translation.y -= 8.0;
            }
        }
    }
}

fn spawn_player_bullets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(&Transform, &Player)>,
    input: Res<ButtonInput<KeyCode>>,
    mut timer: ResMut<PlayerShootTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        if input.just_pressed(KeyCode::Space) {
            if let Some((player_transform, player)) = player_query.iter().next() {
                let texture = asset_server.load("player-bullet.png");

                commands.spawn((
                    Bullet {
                        velocity: Vec2::new(0.0, 100.0),
                    },
                    PlayerBullet {
                        player_id: player.id.clone(),
                    },
                    SpriteBundle {
                        texture,
                        transform: Transform::from_translation(player_transform.translation),
                        ..Default::default()
                    },
                ));

                timer.0.reset();
            }
        }
    }
}

fn bullets_movement(mut bullets: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut transform, bullet) in bullets.iter_mut() {
        transform.translation +=
            Vec3::new(bullet.velocity.x, bullet.velocity.y, 0.0) * time.delta_seconds();
    }
}

fn invaders_shooting(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    shooter_invaders: Query<&Transform, With<Shooter>>,
    time: Res<Time>,
    mut timer: ResMut<InvaderShootTimer>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let mut rng = rand::thread_rng();

        for transform in shooter_invaders.iter() {
            let mut roll = rng.gen_range(0.0..1.0);

            if roll > INVADER_SHOOT_PROBABILITY {
                continue;
            }

            roll = rng.gen_range(0.0..1.0);
            let mut cumulative_probability = 0.0;

            for prob in INVADER_BULLET_PROBABILITIES {
                cumulative_probability += prob.probability;

                if roll < cumulative_probability {
                    let texture = match prob.bullet_type {
                        InvaderBulletType::Bullet => asset_server.load("invader-bullet.png"),
                        InvaderBulletType::Bolt => asset_server.load("invader-bolt.png"),
                    };

                    commands.spawn((
                        Bullet {
                            velocity: Vec2::new(0.0, -100.0),
                        },
                        InvaderBullet {
                            bullet_type: match prob.bullet_type {
                                InvaderBulletType::Bullet => InvaderBulletType::Bullet,
                                InvaderBulletType::Bolt => InvaderBulletType::Bolt,
                            },
                        },
                        SpriteBundle {
                            texture,
                            transform: Transform::from_translation(transform.translation),
                            ..Default::default()
                        },
                    ));
                    break;
                }
            }
        }
    }
}

fn bullet_collision_detection(
    mut commands: Commands,
    mut shields: Query<(Entity, &Transform, &mut ShieldPart)>,
    mut player_bullets: Query<(Entity, &Transform, &PlayerBullet)>,
    mut invader_bullets: Query<(Entity, &Transform, &InvaderBullet)>,
    mut players: Query<(Entity, &Transform, &mut Player)>,
    mut invaders: Query<(Entity, &Transform, &Invader)>,
    mut game_state: ResMut<GameState>,
) {
    for (player_bullet_entity, player_bullet_transform, player_bullet) in player_bullets.iter_mut()
    {
        let mut has_despawned = false;

        for (shield_entity, shield_transform, mut shield) in shields.iter_mut() {
            let dist = player_bullet_transform.translation - Vec3::new(shield.x, shield.y, 0.0);

            if dist.x.abs() < 4.0 && dist.y.abs() < 4.0 {
                shield.health -= 1;

                if shield.health <= 0 {
                    commands.entity(shield_entity).despawn();
                } else {
                    let new_texture = shield.current_texture();

                    commands.entity(shield_entity).insert((
                        ShieldPart {
                            health: shield.health,
                            textures: shield.textures.clone(),
                            x: shield.x,
                            y: shield.y,
                        },
                        SpriteBundle {
                            texture: new_texture,
                            transform: shield_transform.clone(),
                            ..Default::default()
                        },
                    ));
                }

                commands.entity(player_bullet_entity).despawn();
                has_despawned = true;
                break;
            }
        }

        for (invader_entity, invader_transform, _) in invaders.iter_mut() {
            let invader_transform =
                player_bullet_transform.translation - invader_transform.translation;

            if invader_transform.x.abs() < 4.0 && invader_transform.y.abs() < 4.0 {
                for (_, _, mut player) in players.iter_mut() {
                    if player_bullet.player_id.id == player.id.id {
                        player.score += 1;
                        break;
                    }
                }

                commands.entity(player_bullet_entity).despawn();
                commands.entity(invader_entity).despawn();
                has_despawned = true;
                break;
            }
        }

        if !has_despawned {
            if player_bullet_transform.translation.y > WINDOW_HEIGHT / 4.0 - 25.0 {
                commands.entity(player_bullet_entity).despawn();
            }
        }
    }

    for (invader_bullet_entity, invader_bullet_transform, invader_bullet) in
        invader_bullets.iter_mut()
    {
        let mut has_despawned = false;

        match invader_bullet.bullet_type {
            InvaderBulletType::Bullet => {
                for (shield_entity, shield_transform, mut shield) in shields.iter_mut() {
                    let dist =
                        invader_bullet_transform.translation - Vec3::new(shield.x, shield.y, 0.0);

                    if dist.x.abs() < 8.0 && dist.y.abs() < 8.0 {
                        shield.health -= 1;

                        if shield.health <= 0 {
                            commands.entity(shield_entity).despawn();
                        } else {
                            let new_texture = shield.current_texture();

                            commands.entity(shield_entity).insert((
                                ShieldPart {
                                    health: shield.health,
                                    textures: shield.textures.clone(),
                                    x: shield.x,
                                    y: shield.y,
                                },
                                SpriteBundle {
                                    texture: new_texture,
                                    transform: shield_transform.clone(),
                                    ..Default::default()
                                },
                            ));
                        }

                        commands.entity(invader_bullet_entity).despawn();
                        has_despawned = true;
                        break;
                    }
                }

                for (_, player_transform, mut player) in players.iter_mut() {
                    let dist = invader_bullet_transform.translation - player_transform.translation;

                    if dist.x.abs() < 8.0 && dist.y.abs() < 8.0 {
                        player.lives -= 1;
                        player.is_hit = true;
                        if player.lives == 0 {
                            *game_state = GameState::GameOver;
                        } else {
                            *game_state = GameState::PlayerHit;
                        }
                        commands.entity(invader_bullet_entity).despawn();
                        has_despawned = true;
                        break;
                    }
                }
            }
            InvaderBulletType::Bolt => {
                for (_, player_transform, mut player) in players.iter_mut() {
                    let dist = invader_bullet_transform.translation - player_transform.translation;

                    if dist.x.abs() < 8.0 && dist.y.abs() < 8.0 {
                        player.lives -= 1;
                        player.is_hit = true;
                        if player.lives == 0 {
                            *game_state = GameState::GameOver;
                        } else {
                            *game_state = GameState::PlayerHit;
                        }
                        commands.entity(invader_bullet_entity).despawn();
                        has_despawned = true;
                        break;
                    }
                }
            }
        }

        if !has_despawned {
            if invader_bullet_transform.translation.y < -WINDOW_HEIGHT / 4.0 + 25.0 {
                commands.entity(invader_bullet_entity).despawn();
            }
        }
    }
}

fn player_hit_animation(
    mut commands: Commands,
    mut players: Query<(Entity, &Transform, &mut Player)>,
    bullets: Query<(Entity, &Transform, &Bullet)>,
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    mut player_hit_timer: ResMut<PlayerHitTimer>,
    mut player_hit_animation_timer: ResMut<PlayerHitAnimationTimer>,
    time: Res<Time>,
) {
    player_hit_timer.0.tick(time.delta());
    player_hit_animation_timer.0.tick(time.delta());

    if player_hit_timer.0.finished() {
        let player_texture: Handle<Image> = asset_server.load("player.png");

        *game_state = GameState::Playing;

        for (player_entity, player_transform, _) in players.iter_mut() {
            commands.entity(player_entity).insert(SpriteBundle {
                texture: player_texture.clone(),
                transform: player_transform.clone(),
                ..Default::default()
            });
        }

        player_hit_timer.0.reset();
        return;
    }

    let player_hit_1_texture: Handle<Image> = asset_server.load("player-hit-1.png");
    let player_hit_2_texture: Handle<Image> = asset_server.load("player-hit-2.png");

    if player_hit_animation_timer.0.finished() {
        for (player_entity, player_transform, _) in players.iter_mut() {
            if time.elapsed().as_secs_f32() % 0.2 < 0.1 {
                commands.entity(player_entity).insert(SpriteBundle {
                    texture: player_hit_2_texture.clone(),
                    transform: player_transform.clone(),
                    ..Default::default()
                });
            } else {
                commands.entity(player_entity).insert(SpriteBundle {
                    texture: player_hit_1_texture.clone(),
                    transform: player_transform.clone(),
                    ..Default::default()
                });
            }
        }
        player_hit_animation_timer.0.reset();
    }

    for (bullet_entity, _, _) in bullets.iter() {
        commands.entity(bullet_entity).despawn();
    }
}

fn setup_player_score(mut commands: Commands, player: Query<&Player>) {
    commands
        .spawn(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: format!("Score: {}", player.iter().next().unwrap().score),
                    style: TextStyle {
                        font: Default::default(),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                }],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(PlayerScore {
            player_id: player.iter().next().unwrap().id.clone(),
        });
}

fn update_player_score(mut texts: Query<(&mut Text, &PlayerScore)>, players: Query<&Player>) {
    for (mut text, player_score) in texts.iter_mut() {
        for player in players.iter() {
            if player.id.id != player_score.player_id.id {
                continue;
            }
            text.sections[0].value = format!("Score: {}", player.score);
        }
    }
}

fn setup_player_lives(
    mut commands: Commands,
    player: Query<&Player>,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
) {
    let player = player.iter().next().unwrap();
    let player_texture = asset_server.load("player.png");

    commands.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Lives".to_string(),
                style: TextStyle {
                    font: Default::default(),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            }],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(200.0),
            ..Default::default()
        },
        ..Default::default()
    });

    for i in 0..player.lives {
        commands.spawn((
            SpriteBundle {
                texture: player_texture.clone(),
                transform: Transform::from_translation(Vec3::new(
                    WINDOW_WIDTH / 10.0 + i as f32 * 20.0,
                    WINDOW_HEIGHT / 8.0 + 24.0,
                    0.0,
                )),
                ..Default::default()
            },
            PlayerLife {
                player_id: player.id.clone(),
            },
        ));
    }

    match *game_state {
        GameState::Restarting => *game_state = GameState::Playing,
        _ => {}
    }
}

fn update_player_lives(
    mut commands: Commands,
    player: Query<&Player>,
    mut player_lives: Query<(Entity, &PlayerLife)>,
) {
    let player = player.iter().next().unwrap();

    for (i, (player_life_entity, player_life)) in player_lives.iter_mut().enumerate() {
        if i >= player.lives as usize && player_life.player_id.id == player.id.id {
            commands.entity(player_life_entity).despawn();
        }
    }
}

fn game_over(
    mut commands: Commands,
    bullets: Query<(Entity, &Bullet)>,
    invaders: Query<(Entity, &Invader)>,
    mut players: Query<(Entity, &Transform, &mut Player)>,
    mut player_hit_timer: ResMut<PlayerHitTimer>,
    mut player_hit_animation_timer: ResMut<PlayerHitAnimationTimer>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    windows: Query<&Window>,
    existing_restart_buttons: Query<Entity, With<GameRestartButton>>,
) {
    if !player_hit_timer.0.finished() {
        player_hit_timer.0.tick(time.delta());

        let player_hit_1_texture: Handle<Image> = asset_server.load("player-hit-1.png");
        let player_hit_2_texture: Handle<Image> = asset_server.load("player-hit-2.png");

        player_hit_animation_timer.0.tick(time.delta());

        if player_hit_animation_timer.0.finished() {
            for (player_entity, player_transform, _) in players.iter_mut() {
                if time.elapsed().as_secs_f32() % 0.2 < 0.1 {
                    commands.entity(player_entity).insert(SpriteBundle {
                        texture: player_hit_2_texture.clone(),
                        transform: player_transform.clone(),
                        ..Default::default()
                    });
                } else {
                    commands.entity(player_entity).insert(SpriteBundle {
                        texture: player_hit_1_texture.clone(),
                        transform: player_transform.clone(),
                        ..Default::default()
                    });
                }
            }
            player_hit_animation_timer.0.reset();
        }
    }

    for (bullet_entity, _) in bullets.iter() {
        commands.entity(bullet_entity).despawn();
    }

    for (invader_entity, _) in invaders.iter() {
        commands.entity(invader_entity).despawn();
    }

    let mut window_width = WINDOW_WIDTH;
    let mut window_height = WINDOW_HEIGHT;

    if !existing_restart_buttons.is_empty() {
        return;
    }

    for window in windows.iter() {
        window_width = window.width();
        window_height = window.height();
    }

    commands.spawn((
        TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Game Over".to_string(),
                    style: TextStyle {
                        font: Default::default(),
                        font_size: 75.0,
                        color: Color::WHITE,
                    },
                }],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(window_height / 4.0),
                left: Val::Px((window_width / 4.0) - 12.5),
                ..Default::default()
            },
            ..Default::default()
        },
        GameOverText {},
    ));

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px((window_height / 2.0) - 10.0),
                    left: Val::Px((window_width / 2.0) - 82.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            GameRestartButton {},
        ))
        .with_children(|button| {
            button.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: "Play Again?".to_string(),
                            style: TextStyle {
                                font: Default::default(),
                                font_size: 30.0,
                                color: Color::WHITE,
                            },
                        }],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                GameRestartText {},
            ));
        });
}

fn play_again(
    interactions: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<GameRestartButton>),
    >,
    mut windows: Query<&mut Window>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    mut players: Query<(Entity, &Transform, &mut Player)>,
    game_over_texts: Query<Entity, With<GameOverText>>,
    game_restart_buttons: Query<Entity, With<GameRestartButton>>,
    game_restart_texts: Query<Entity, With<GameRestartText>>,
    scores: Query<Entity, With<PlayerScore>>,
    lives: Query<Entity, With<PlayerLife>>,
    shield_parts: Query<Entity, With<ShieldPart>>,
    mut player_hit_timer: ResMut<PlayerHitTimer>,
) {
    for mut window in windows.iter_mut() {
        for interaction in interactions.iter() {
            match *interaction {
                Interaction::Pressed => {
                    for shield_part_entity in shield_parts.iter() {
                        if shield_parts.get(shield_part_entity).is_ok() {
                            commands.entity(shield_part_entity).despawn();
                        }
                    }

                    for (player_entity, _, _) in players.iter_mut() {
                        commands.entity(player_entity).despawn();
                    }

                    for game_over_text_entity in game_over_texts.iter() {
                        commands.entity(game_over_text_entity).despawn();
                    }

                    for game_restart_button_entity in game_restart_buttons.iter() {
                        commands.entity(game_restart_button_entity).despawn();
                    }

                    for game_restart_text_entity in game_restart_texts.iter() {
                        commands.entity(game_restart_text_entity).despawn();
                    }

                    for score_entity in scores.iter() {
                        commands.entity(score_entity).despawn();
                    }

                    for life_entity in lives.iter() {
                        commands.entity(life_entity).despawn();
                    }

                    player_hit_timer.0.reset();

                    *game_state = GameState::Restarting;
                }
                Interaction::Hovered => {
                    window.cursor.icon = CursorIcon::Pointer;
                }
                Interaction::None => {
                    window.cursor.icon = CursorIcon::Default;
                }
            }
        }
    }
}

fn check_if_invaders_defeated(
    invaders: Query<(Entity, &Invader)>,
    mut game_state: ResMut<GameState>,
) {
    if invaders.iter().count() == 0 {
        *game_state = GameState::RespawningInvaders;
    }
}

fn respawn_invaders(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
) {
    setup_invaders(commands, asset_server);
    *game_state = GameState::Pausing;
}

fn pause_game(
    mut game_state: ResMut<GameState>,
    mut respawn_timer: ResMut<RespawningInvadersTimer>,
    time: Res<Time>,
) {
    respawn_timer.0.tick(time.delta());

    if respawn_timer.0.finished() {
        *game_state = GameState::Playing;
    }
}
