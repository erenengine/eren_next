use bevy::{prelude::*, text::FontSmoothing};
use bevy_dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use rand::Rng;

const NUM_SPRITES: usize = 100_000;
const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
const SPRITE_SPEED: f32 = 2000.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Sprite Movement".into(),
                    resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                    ..default()
                }),
                ..default()
            }),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 42.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::srgb(0.0, 1.0, 0.0),
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                },
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, move_sprites)
        .run();
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let texture_handle = asset_server.load("test_sprite.png");

    let mut rng = rand::rng();

    for _ in 0..NUM_SPRITES {
        let x = rng.random_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
        let y = rng.random_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.0);

        let scale = rng.random_range(0.5..2.0);
        let rotation = rng.random_range(0.0..2.0 * std::f32::consts::PI);
        let alpha = rng.random_range(0.0..1.0);

        let vx = rng.random_range(-SPRITE_SPEED..SPRITE_SPEED);
        let vy = rng.random_range(-SPRITE_SPEED..SPRITE_SPEED);

        let mut sprite = Sprite::from_image(texture_handle.clone());
        sprite.color = Color::linear_rgba(1.0, 1.0, 1.0, alpha);

        let mut transform = Transform::from_xyz(x, y, 0.0);
        transform.scale = Vec3::splat(scale);
        transform.rotation = Quat::from_rotation_z(rotation);

        commands.spawn((sprite, transform, Velocity { x: vx, y: vy }));
    }
}

fn move_sprites(time: Res<Time>, mut query: Query<(&mut Transform, &mut Velocity)>) {
    let half_screen = Vec2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0);
    let half_size = Vec2::new(32.0, 32.0);

    for (mut transform, mut velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();

        let sprite_screen_x = transform.translation.x + half_screen.x;
        let sprite_screen_y = transform.translation.y + half_screen.y;

        if sprite_screen_x < half_size.x || sprite_screen_x > WINDOW_WIDTH - half_size.x {
            velocity.x *= -1.0;
        }
        if sprite_screen_y < half_size.y || sprite_screen_y > WINDOW_HEIGHT - half_size.y {
            velocity.y *= -1.0;
        }
    }
}
