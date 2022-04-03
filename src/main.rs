
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

/// An implementation of the classic game "Breakout"
pub const RESOLUTION: f32 = 16.0/9.0;

fn main() {
    let height = 900.0;
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(WindowDescriptor {
            width: height*RESOLUTION,
            height: height,
            title: String::from("LD Stranded"),
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .insert_resource(Scoreboard { score: 0 })
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_tiles)
        .add_startup_system(spawn_resources)
        .add_system(diffusion_system)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        
        //.add_startup_system(spawn_player)
        //.add_system(scoreboard_system)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

struct Scoreboard {
    score: usize,
}
fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.top = 1.0;
    camera.orthographic_projection.bottom = -1.0;

    camera.orthographic_projection.right = 1.0 * RESOLUTION;
    camera.orthographic_projection.left = -1.0 * RESOLUTION;

    //camera.orthographic_projection.scaling_mode = ScalingMode::None;
    commands.spawn_bundle(camera);
}

fn spawn_tiles(mut commands: Commands,asset_server: Res<AssetServer>,mut map_query: MapQuery){
    let texture_handle = asset_server.load("tileset x1.png");

    let map_entitity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entitity);
    let layer_settings = LayerSettings::new(
        MapSize(8,4),
        ChunkSize(6,6),
        TileSize(32.0,32.0),
        TextureSize(1184.0,736.0),
    );
    //floor tiles 
    let (mut floor, floor_entitity) = LayerBuilder::new(
        &mut commands,
        layer_settings.clone(),
        0u16,
        0u16,
    );
    floor.set_all(TileBundle{
        tile: Tile{
            texture_index: 71,
            ..Default::default()
        },
        ..Default::default()
    });
    map.add_layer(&mut commands, 0u16, floor_entitity);
    map_query.build_layer(&mut commands, floor, texture_handle.clone());
    //bounding walls
    let (mut walls, wall_entitity) = LayerBuilder::new(
        &mut commands,
        layer_settings.clone(),
        0u16,
        2,
    );// top
    for n in 0..47 {
        let _ = walls.set_tile(TilePos(n,23), 
            TileBundle {
                tile: Tile {
                    texture_index:5, 
                    ..Default::default()
                },
                ..Default::default()
            },
        );//bottom
        let _ = walls.set_tile(TilePos(n,0), 
            TileBundle {
                tile: Tile {
                    texture_index:375, 
                    ..Default::default()
                },
                ..Default::default()
            },
        );
    }//left
    for n in 0..23 {
        let _ = walls.set_tile(TilePos(0,n), 
            TileBundle {
                tile: Tile {
                    texture_index:75, 
                    ..Default::default()
                },
                ..Default::default()
            },
        );//right
        let _ = walls.set_tile(TilePos(47,n), 
            TileBundle {
                tile: Tile {
                    texture_index:83, 
                    ..Default::default()
                },
                ..Default::default()
            },
        );
    }//corners
    let _ = walls.set_tile(TilePos(0,23), 
        TileBundle {
            tile: Tile {
                texture_index:2, 
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let _ = walls.set_tile(TilePos(47,23), 
        TileBundle {
            tile: Tile {
                texture_index:8, 
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let _ = walls.set_tile(TilePos(0,0), 
        TileBundle {
            tile: Tile {
                texture_index:372, 
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let _ = walls.set_tile(TilePos(47,0), 
        TileBundle {
            tile: Tile {
                texture_index:378, 
                ..Default::default()
            },
            ..Default::default()
        },
    );
    map_query.build_layer(&mut commands, walls, texture_handle.clone());
    map.add_layer(&mut commands, 2, wall_entitity);
    //spawn everything
    commands
        .entity(map_entitity)
        .insert(map)
        .insert(Transform::from_xyz(-768.0, -352.0, 0.0))
        .insert(GlobalTransform::default());
}

#[derive(Component, Copy, Clone)]
struct TileResources {air: f32, water: f32, fire:f32}

fn spawn_resources(mut commands: Commands) {
    for n in 0..47 {
        for m in 0..23 {
            commands.spawn()
                .insert(TileResources{air:1.0, water:0.0, fire:0.0})
                .insert(TilePos(n,m));
        }
    }
}

fn diffusion_system(mut resource_query: Query<(&mut TileResources, &TilePos)>) {
    for x0 in 0..47 {
        for y0 in 0..23 { 
            let mut tot_air = 0.0;
            let mut tot_h2o = 0.0;
            let mut tot_fir = 0.0;
            let mut tile_count = 0.0;
            for (res, tile_pos2) in resource_query.iter() {
                let mut count = 0;
                if x0 == 0||x0 == 47 {
                    count += 3;
                }
                if y0 == 0 || y0 == 24 {
                    count += 3;
                }
                count = count.min(5);
                let mut west = false;
                let mut east = false;
                let mut north = false;
                let mut south = false;
                if tile_pos2.0 < x0 {
                    if x0 - tile_pos2.0 == 1 {
                        west = true;
                    }
                } else if tile_pos2.0 > x0 {
                    if tile_pos2.0 - x0 == 1 {
                        east = true;
                    }
                } else {
                    east = true;
                    west = true;
                }
                if tile_pos2.1 < y0 {
                    if y0 - tile_pos2.1 == 1 {
                        north = true;
                    }
                } else if tile_pos2.1 > y0 {
                    if tile_pos2.1 - y0 == 1 {
                        south = true;
                    }
                } else {
                    north = true;
                    south = true;
                }
                if north && east && south && west {
                    tot_air += res.air;
                    tot_h2o += res.water;
                    tot_fir += res.fire;
                    count += 1;
                    tile_count += 1f32;
                } else if north || east || south || west {
                    //aggregate surrounding tiles
                    tot_air += res.air;
                    tot_h2o += res.water;
                    tot_fir += res.fire;
                    count += 1;
                    tile_count += 1f32;
                }
                if count >= 9 {
                    break
                }
            }
            for (mut res, tile_pos2) in resource_query.iter_mut() {
                if tile_pos2.0 == x0 && tile_pos2.1 == y0 {
                    res.air = tot_air/tile_count;
                    res.water = tot_h2o/tile_count;
                    res.fire = tot_fir/tile_count;
                }
            } 
        }
    }
}
/*
fn spawn_player(mut commands: Commands, tiles: Res<Tilemap>) {
    let sprite = TextureAtlasSprite::new(8);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: sprite,
            texture_atlas: tiles.0.clone(),
            transform: Transform {
                translation: Vec3::new(0.0,0.0,900.0), 
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Player"));
}
*/
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Add the game's entities to our world

    // cameras
    //commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    
    
    commands.spawn_bundle(UiCameraBundle::default());
    // scoreboard
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Score: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(1.0, 0.5, 0.5),
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}
/*
fn paddle_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Paddle, &mut Transform)>,
) {
    let (paddle, mut transform) = query.single_mut();
    let mut direction = 0.0;
    if keyboard_input.pressed(KeyCode::Left) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        direction += 1.0;
    }

    let translation = &mut transform.translation;
    // move the paddle horizontally
    translation.x += direction * paddle.speed * TIME_STEP;
    // bound the paddle within the walls
    translation.x = translation.x.min(380.0).max(-380.0);

    if keyboard_input.pressed(KeyCode::Up) {
        translation.y += paddle.speed * TIME_STEP;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        translation.y -= paddle.speed * TIME_STEP;
    }
    translation.y = translation.y.min(210.0).max(-190.0);


}*/
/*
fn ball_movement_system(mut ball_query: Query<(&Ball, &mut Transform)>) {
    let (ball, mut transform) = ball_query.single_mut();
    transform.translation += ball.velocity * TIME_STEP;
}
*/
fn scoreboard_system(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("{}", scoreboard.score);
}
/*
fn ball_collision_system(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<(&mut Ball, &Transform)>,
    collider_query: Query<(Entity, &Collider, &Transform)>,
) {
    let (mut ball, ball_transform) = ball_query.single_mut();
    let ball_size = ball_transform.scale.truncate();
    let velocity = &mut ball.velocity;

    // check collision with walls
    for (collider_entity, collider, transform) in collider_query.iter() {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            // scorable colliders should be despawned and increment the scoreboard on collision
            if let Collider::Scorable = *collider {
                scoreboard.score += 1;
                commands.entity(collider_entity).despawn();
            }

            // reflect the ball when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // only reflect if the ball's velocity is going in the opposite direction of the
            // collision
            match collision {
                Collision::Left => reflect_x = velocity.x > 0.0,
                Collision::Right => reflect_x = velocity.x < 0.0,
                Collision::Top => reflect_y = velocity.y < 0.0,
                Collision::Bottom => reflect_y = velocity.y > 0.0,
            }

            // reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                velocity.x = -velocity.x;
            }

            // reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                velocity.y = -velocity.y;
            }

            // break if this collide is on a solid, otherwise continue check whether a solid is
            // also in collision
            if let Collider::Solid = *collider {
                break;
            }
        }
    }
}
*/