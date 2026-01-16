use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

pub fn camera_zoom(
    mut scroll_events: MessageReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    for event in scroll_events.read() {
        for mut transform in query.iter_mut() {
            // Zoom in/out based on scroll direction
            let zoom_delta = event.y * 0.1;

            // Update camera scale (larger = zoomed in, smaller = zoomed out)
            let new_scale = (transform.scale.x + zoom_delta).clamp(0.3, 3.0);
            transform.scale = Vec3::splat(new_scale);
        }
    }
}

pub fn camera_pan(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut motion_events: MessageReader<CursorMoved>,
    mut query: Query<&mut Transform, With<Camera2d>>,
    mut last_pos: Local<Option<Vec2>>,
) {
    // Check if right mouse button or middle mouse button is pressed
    let is_dragging =
        mouse_button.pressed(MouseButton::Right) || mouse_button.pressed(MouseButton::Middle);

    if is_dragging {
        for event in motion_events.read() {
            if let Some(last) = *last_pos {
                for mut transform in query.iter_mut() {
                    // Calculate delta movement
                    let delta = event.position - last;

                    // Move camera in opposite direction (inverted controls feel more natural)
                    // Scale movement by camera scale so panning speed feels consistent
                    transform.translation.x -= delta.x * transform.scale.x;
                    transform.translation.y += delta.y * transform.scale.y; // Y is inverted in screen space
                }
            }
            *last_pos = Some(event.position);
        }
    } else {
        // Reset last position when not dragging
        if mouse_button.just_released(MouseButton::Right)
            || mouse_button.just_released(MouseButton::Middle)
        {
            *last_pos = None;
        }
        // Update last position even when not dragging to prevent jumps
        for event in motion_events.read() {
            *last_pos = Some(event.position);
        }
    }
}
