//! Camera plugin with multiple view modes, zoom, and pan

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::components::{CameraMode, PlayerMarker};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<CameraMode>()
            .init_resource::<CameraSettings>()
            .init_resource::<CameraControl>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    switch_camera_mode,
                    handle_mouse_input,
                    update_camera_projection,
                    update_camera_position,
                )
                    .chain(),
            );
    }
}

/// Marker for the main camera
#[derive(Component)]
pub struct MainCamera;

/// Camera configuration
#[derive(Resource)]
pub struct CameraSettings {
    pub follow_speed: f32,
    pub third_person_distance: f32,
    pub third_person_height: f32,
    pub isometric_distance: f32,
    pub zoom_speed: f32,
    pub pan_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            follow_speed: 8.0,
            third_person_distance: 10.0,
            third_person_height: 8.0,
            isometric_distance: 15.0,
            zoom_speed: 0.1,
            pan_speed: 0.05,
            min_zoom: 0.1,
            max_zoom: 5.0,
        }
    }
}

/// Runtime camera control state (zoom and pan)
#[derive(Resource)]
pub struct CameraControl {
    /// Zoom level (1.0 = default, smaller = zoomed in, larger = zoomed out)
    pub zoom: f32,
    /// Pan offset from target position (in world coordinates)
    pub pan_offset: Vec3,
    /// Whether we're currently panning (mouse held)
    pub is_panning: bool,
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_offset: Vec3::ZERO,
            is_panning: false,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    // Start with top-down orthographic view centered on map
    // Map is 80x21, so center at (40, 0, 10.5)
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 30.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(40.0, 40.0, 10.5).looking_at(Vec3::new(40.0, 0.0, 10.5), Vec3::Z),
    ));
}

fn switch_camera_mode(
    input: Res<ButtonInput<KeyCode>>,
    current_mode: Res<State<CameraMode>>,
    mut next_mode: ResMut<NextState<CameraMode>>,
    mut control: ResMut<CameraControl>,
) {
    let new_mode = if input.just_pressed(KeyCode::F1) {
        Some(CameraMode::TopDown)
    } else if input.just_pressed(KeyCode::F2) {
        Some(CameraMode::Isometric)
    } else if input.just_pressed(KeyCode::F3) {
        Some(CameraMode::ThirdPerson)
    } else if input.just_pressed(KeyCode::F4) {
        Some(CameraMode::FirstPerson)
    } else {
        None
    };

    if let Some(mode) = new_mode {
        if *current_mode.get() != mode {
            next_mode.set(mode);
            // Reset pan offset when switching modes
            control.pan_offset = Vec3::ZERO;
        }
    }

    // Reset view with Home key
    if input.just_pressed(KeyCode::Home) {
        control.zoom = 1.0;
        control.pan_offset = Vec3::ZERO;
    }
}

fn handle_mouse_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    camera_mode: Res<State<CameraMode>>,
    settings: Res<CameraSettings>,
    mut control: ResMut<CameraControl>,
) {
    // Handle zoom with scroll wheel
    let scroll_delta = mouse_scroll.delta.y;
    if scroll_delta != 0.0 {
        // Zoom in = smaller value (see more detail), zoom out = larger value (see more)
        let zoom_change = -scroll_delta * settings.zoom_speed;
        control.zoom = (control.zoom + zoom_change).clamp(settings.min_zoom, settings.max_zoom);
    }

    // Handle panning with middle mouse button or right mouse button
    let panning =
        mouse_button.pressed(MouseButton::Middle) || mouse_button.pressed(MouseButton::Right);
    control.is_panning = panning;

    if panning {
        let delta = mouse_motion.delta;
        if delta != Vec2::ZERO {
            // Convert screen delta to world delta based on camera mode
            let pan_multiplier = control.zoom * settings.pan_speed;

            match camera_mode.get() {
                CameraMode::TopDown => {
                    // In top-down, X is left-right, Z is up-down on screen
                    control.pan_offset.x -= delta.x * pan_multiplier;
                    control.pan_offset.z -= delta.y * pan_multiplier;
                }
                CameraMode::Isometric => {
                    // In isometric, we need to transform screen coords to world
                    // Approximate: diagonal movement
                    let dx = (-delta.x + delta.y) * pan_multiplier * 0.7;
                    let dz = (-delta.x - delta.y) * pan_multiplier * 0.7;
                    control.pan_offset.x += dx;
                    control.pan_offset.z += dz;
                }
                CameraMode::ThirdPerson => {
                    // Pan left-right and forward-back relative to view
                    control.pan_offset.x -= delta.x * pan_multiplier;
                    control.pan_offset.z += delta.y * pan_multiplier;
                }
                CameraMode::FirstPerson => {
                    // In first person, could rotate view instead, but for now just pan
                    control.pan_offset.x -= delta.x * pan_multiplier * 0.5;
                    control.pan_offset.z += delta.y * pan_multiplier * 0.5;
                }
            }
        }
    }
}

fn update_camera_projection(
    camera_mode: Res<State<CameraMode>>,
    control: Res<CameraControl>,
    mut camera_query: Query<&mut Projection, With<MainCamera>>,
) {
    // Update when mode changes or zoom changes
    if !camera_mode.is_changed() && !control.is_changed() {
        return;
    }

    let Ok(mut projection) = camera_query.get_single_mut() else {
        return;
    };

    match camera_mode.get() {
        CameraMode::TopDown => {
            // Apply zoom to viewport height (larger = zoomed out)
            let height = 30.0 * control.zoom;
            *projection = Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: height,
                },
                ..OrthographicProjection::default_3d()
            });
        }
        CameraMode::Isometric => {
            let height = 25.0 * control.zoom;
            *projection = Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: height,
                },
                ..OrthographicProjection::default_3d()
            });
        }
        CameraMode::ThirdPerson | CameraMode::FirstPerson => {
            // For perspective, zoom affects FOV
            let base_fov = 60.0_f32.to_radians();
            let fov =
                (base_fov * control.zoom).clamp(20.0_f32.to_radians(), 120.0_f32.to_radians());
            *projection = Projection::Perspective(PerspectiveProjection { fov, ..default() });
        }
    }
}

fn update_camera_position(
    camera_mode: Res<State<CameraMode>>,
    settings: Res<CameraSettings>,
    control: Res<CameraControl>,
    time: Res<Time>,
    player_query: Query<&Transform, (With<PlayerMarker>, Without<MainCamera>)>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    // Get player position or use map center as default
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::new(40.0, 0.0, 10.5));

    // Apply pan offset to base position
    let base_pos = player_pos + control.pan_offset;

    let (target_pos, target_look) = match camera_mode.get() {
        CameraMode::TopDown => {
            // Directly above, looking down
            // Height scales with zoom for orthographic
            let height = 40.0;
            let pos = Vec3::new(base_pos.x, height, base_pos.z);
            let look = Vec3::new(base_pos.x, 0.0, base_pos.z);
            (pos, look)
        }
        CameraMode::Isometric => {
            // 45-degree angle from southeast
            let dist = settings.isometric_distance * control.zoom;
            let offset = Vec3::new(dist, dist, dist);
            let pos = base_pos + offset;
            (pos, base_pos)
        }
        CameraMode::ThirdPerson => {
            // Behind and above player (looking north by default)
            // Distance scales with zoom
            let dist = settings.third_person_distance * control.zoom;
            let height = settings.third_person_height * control.zoom;
            let pos = base_pos + Vec3::new(0.0, height, -dist);
            let look = base_pos + Vec3::Y * 0.5;
            (pos, look)
        }
        CameraMode::FirstPerson => {
            // At player eye level
            let pos = base_pos + Vec3::Y * 0.8;
            // Look forward (north in this case)
            let look = pos + Vec3::Z * 10.0;
            (pos, look)
        }
    };

    // Smooth interpolation (faster when panning for responsiveness)
    let speed = if control.is_panning {
        15.0 * time.delta_secs()
    } else {
        settings.follow_speed * time.delta_secs()
    };
    camera_transform.translation = camera_transform.translation.lerp(target_pos, speed);

    // Update look direction
    let direction = (target_look - camera_transform.translation).normalize_or_zero();
    if direction.length_squared() > 0.001 {
        // Choose appropriate up vector based on mode
        let up = match camera_mode.get() {
            CameraMode::TopDown => Vec3::Z, // Use Z as up for top-down to avoid gimbal lock
            _ => Vec3::Y,
        };
        camera_transform.look_to(direction, up);
    }
}
