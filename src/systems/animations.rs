use bevy::prelude::*;
use crate::components::*;

pub fn spawn_animated_text(commands: &mut Commands, player_id: PlayerId, text: &str, color: Color) {
    let start_pos = Vec2::new(-400.0 + (player_id.0 as f32 * 200.0), 200.0);
    let end_pos = Vec2::new(start_pos.x, start_pos.y + 50.0);
    
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font_size: 20.0,
                    color,
                    ..default()
                },
            ),
            transform: Transform::from_translation(start_pos.extend(10.0)),
            ..default()
        },
        AnimatedText {
            timer: Timer::from_seconds(1.5, TimerMode::Once),
            start_pos,
            end_pos,
        },
    ));
}

pub fn animate_text_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animated_texts: Query<(Entity, &mut Transform, &mut AnimatedText, &mut Text)>,
) {
    for (entity, mut transform, mut animated_text, mut text) in animated_texts.iter_mut() {
        animated_text.timer.tick(time.delta());
        
        let progress = animated_text.timer.elapsed_secs() / animated_text.timer.duration().as_secs_f32();
        let current_pos = animated_text.start_pos.lerp(animated_text.end_pos, progress);
        transform.translation = current_pos.extend(10.0);
        
        let alpha = (1.0 - progress).max(0.0);
        for section in text.sections.iter_mut() {
            let mut color = section.style.color;
            match &mut color {
                Color::Srgba(srgba) => srgba.alpha = alpha,
                Color::LinearRgba(linear) => linear.alpha = alpha,
                _ => {}
            }
            section.style.color = color;
        }
        
        if animated_text.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}