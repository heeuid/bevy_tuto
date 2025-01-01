use bevy::math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume};
use bevy::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Score>()
        .add_event::<Scored>()
        .add_systems(
            Startup,
            (
                spawn_ball,
                spawn_camera,
                spawn_paddles,
                spawn_gutters,
                spawn_scoreboard,
            ),
        )
        .add_systems(
            Update,
            (
                move_ai,
                move_ball,
                project_positions.after(move_ball),
                handle_collisions.after(move_ball),
                handle_player_input,
                move_paddles.after(handle_player_input),
                detect_scoring,
                reset_ball.after(detect_scoring),
                update_score.after(detect_scoring),
                update_scoreboard.after(update_score),
            ),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn project_positions(mut positionables: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut positionables {
        transform.translation = position.0.extend(0.);
    }
}

#[derive(Component)]
struct Ai;

fn move_ai(
    mut ai: Query<(&mut Velocity, &Position), With<Ai>>,
    ball: Query<&Position, With<Ball>>,
) {
    if let Ok((mut velocity, position)) = ai.get_single_mut() {
        if let Ok(ball_position) = ball.get_single() {
            let a_to_b = ball_position.0 - position.0;

            let ai_speed_factor = rand::thread_rng().gen_range(0.8..1.0);
            velocity.0.y = a_to_b.y.signum() * ai_speed_factor;
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Shape(Vec2);

const BALL_SIZE: f32 = 5.;
const BALL_SPEED: f32 = 5.;

#[derive(Component)]
struct Ball;

#[derive(Bundle)]
struct BallBundle {
    ball: Ball,
    shape: Shape,
    velocity: Velocity,
    position: Position,
}

impl BallBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            ball: Ball,
            shape: Shape(Vec2::new(BALL_SIZE, BALL_SIZE)),
            velocity: Velocity(Vec2::new(x, y)),
            position: Position(Vec2::ZERO),
        }
    }
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("Spawning ball ...");

    let shape = Circle::new(BALL_SIZE);
    let color = Color::srgb(1., 0., 0.);

    let mesh = meshes.add(shape);
    let material = materials.add(color);

    commands.spawn((
        BallBundle::new(1., 1.),
        Mesh2d(mesh),
        MeshMaterial2d(material),
    ));
}

fn move_ball(mut ball: Query<(&mut Position, &Velocity), With<Ball>>) {
    if let Ok((mut position, velocity)) = ball.get_single_mut() {
        position.0 += velocity.0 * BALL_SPEED;
    }
}

const PADDLE_WIDTH: f32 = 10.;
const PADDLE_HEIGHT: f32 = 50.;
const PADDLE_SPEED: f32 = 5.;

#[derive(Component)]
struct Paddle;

#[derive(Bundle)]
struct PaddleBundle {
    paddle: Paddle,
    shape: Shape,
    position: Position,
    velocity: Velocity,
}

impl PaddleBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            paddle: Paddle,
            shape: Shape(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
            position: Position(Vec2::new(x, y)),
            velocity: Velocity(Vec2::ZERO),
        }
    }
}

fn spawn_paddles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    println!("Spawning paddles ...");

    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let padding = 50.;
        let right_paddle_x = window_width / 2. - padding;
        let left_paddle_x = -window_width / 2. + padding;

        let shape = Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT);
        let mesh = meshes.add(shape);
        let player_color = materials.add(Color::srgb(0., 1., 0.));
        let ai_color = materials.add(Color::srgb(0., 0., 1.));

        commands.spawn((
            Player,
            PaddleBundle::new(right_paddle_x, 0.),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(player_color),
        ));

        commands.spawn((
            Ai,
            PaddleBundle::new(left_paddle_x, 0.),
            Mesh2d(mesh),
            MeshMaterial2d(ai_color),
        ));
    }
}

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle: Query<&mut Velocity, With<Player>>,
) {
    if let Ok(mut velocity) = paddle.get_single_mut() {
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            velocity.0.y = 1.;
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            velocity.0.y = -1.;
        } else {
            velocity.0.y = 0.;
        }
    }
}

fn move_paddles(
    mut paddle: Query<(&mut Position, &Velocity), With<Paddle>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();
        let max_y = window_height / 2. - GUTTER_HEIGHT - PADDLE_HEIGHT / 2.;

        for (mut position, velocity) in &mut paddle {
            let new_position = position.0 + velocity.0 * PADDLE_SPEED;
            if new_position.y.abs() < max_y {
                position.0 = new_position;
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left, // of wall
    Right,
    Top,
    Bottom,
}

fn collide_with_side(ball: BoundingCircle, wall: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None;
    }

    let closest_point = wall.closest_point(ball.center());
    let offset = ball.center() - closest_point;

    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn handle_collisions(
    mut ball: Query<(&mut Velocity, &Position, &Shape), With<Ball>>,
    other_things: Query<(&Position, &Shape), Without<Ball>>,
) {
    if let Ok((mut ball_velocity, ball_position, ball_shape)) = ball.get_single_mut() {
        for (position, shape) in &other_things {
            if let Some(collision) = collide_with_side(
                BoundingCircle::new(ball_position.0, ball_shape.0.x),
                Aabb2d::new(position.0, shape.0 / 2.),
            ) {
                match collision {
                    Collision::Left | Collision::Right => {
                        ball_velocity.0.x *= -1.;
                    }
                    Collision::Top | Collision::Bottom => {
                        ball_velocity.0.y *= -1.;
                    }
                }
            }
        }
    }
}

const GUTTER_HEIGHT: f32 = 20.;

#[derive(Component)]
struct Gutter;

#[derive(Bundle)]
struct GutterBundle {
    gutter: Gutter,
    shape: Shape,
    position: Position,
}

impl GutterBundle {
    fn new(x: f32, y: f32, width: f32) -> Self {
        Self {
            gutter: Gutter,
            shape: Shape(Vec2::new(width, GUTTER_HEIGHT)),
            position: Position(Vec2::new(x, y)),
        }
    }
}

fn spawn_gutters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let window_height = window.resolution.height();

        let top_gutter_y = window_height / 2. - GUTTER_HEIGHT / 2.;
        let bottom_gutter_y = -window_height / 2. + GUTTER_HEIGHT / 2.;

        let top_gutter = GutterBundle::new(0., top_gutter_y, window_width);
        let bottom_gutter = GutterBundle::new(0., bottom_gutter_y, window_width);

        let shape = Rectangle::from_size(top_gutter.shape.0);
        let color = Color::srgb(0., 0., 0.);

        let mesh = meshes.add(shape);
        let material = materials.add(color);

        commands.spawn((
            top_gutter,
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
        ));

        commands.spawn((bottom_gutter, Mesh2d(mesh), MeshMaterial2d(material)));
    }
}

enum Scorer {
    Ai,
    Player,
}

#[derive(Event)]
struct Scored(Scorer);

#[derive(Resource, Default)]
struct Score {
    ai: u32,
    player: u32,
}

fn detect_scoring(
    mut ball: Query<&mut Position, With<Ball>>,
    window: Query<&Window>,
    mut events: EventWriter<Scored>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();

        if let Ok(ball) = ball.get_single_mut() {
            if ball.0.x > window_width / 2. {
                events.send(Scored(Scorer::Ai));
            } else if ball.0.x < -window_width / 2. {
                events.send(Scored(Scorer::Player));
            }
        }
    }
}

fn reset_ball(
    mut ball: Query<(&mut Position, &mut Velocity), With<Ball>>,
    mut events: EventReader<Scored>,
) {
    for event in events.read() {
        if let Ok((mut position, mut velocity)) = ball.get_single_mut() {
            match event.0 {
                Scorer::Ai => {
                    position.0 = Vec2::ZERO;
                    velocity.0 = Vec2::new(-1., 1.);
                }
                Scorer::Player => {
                    position.0 = Vec2::ZERO;
                    velocity.0 = Vec2::new(1., 1.);
                }
            }
        }
    }
}

fn update_score(mut score: ResMut<Score>, mut events: EventReader<Scored>) {
    for event in events.read() {
        match event.0 {
            Scorer::Ai => score.ai += 1,
            Scorer::Player => score.player += 1,
        }
    }

    println!("Score: {} - {} (player, ai)", score.player, score.ai);
}

#[derive(Component)]
struct PlayerScore;

#[derive(Component)]
struct AiScore;

fn update_scoreboard(
    mut player_score: Query<&mut Text, With<PlayerScore>>,
    mut ai_score: Query<&mut Text, (With<AiScore>, Without<PlayerScore>)>,
    score: Res<Score>,
) {
    if score.is_changed() {
        if let Ok(mut player_score) = player_score.get_single_mut() {
            player_score.0 = score.player.to_string();
        }

        if let Ok(mut ai_score) = ai_score.get_single_mut() {
            ai_score.0 = score.ai.to_string();
        }
    }
}

fn spawn_scoreboard(mut commands: Commands) {
    let padding = 10.;

    commands.spawn((
        Text::new("0"),
        TextFont::from_font_size(72.),
        TextColor::from(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Left),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(GUTTER_HEIGHT),
            right: Val::Px(padding),
            ..default()
        },
        PlayerScore,
    ));
    commands.spawn((
        Text::new("0"),
        TextFont::from_font_size(72.),
        TextColor::from(Color::srgb(1., 0., 0.)),
        TextLayout::new_with_justify(JustifyText::Right),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(GUTTER_HEIGHT),
            left: Val::Px(padding),
            ..default()
        },
        AiScore,
    ));
}
