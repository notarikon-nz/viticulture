use bevy::prelude::*;
use crate::components::*;

// Enhanced animation components
#[derive(Component)]
pub struct WorkerAnimation {
    pub start_pos: Vec2,
    pub target_pos: Vec2,
    pub timer: Timer,
    pub animation_type: WorkerAnimationType,
}

#[derive(Clone, Copy)]
pub enum WorkerAnimationType {
    Placement,
    Return,
    Bounce,
}

#[derive(Component)]
pub struct CardAnimation {
    pub start_pos: Vec2,
    pub target_pos: Vec2,
    pub timer: Timer,
    pub animation_type: CardAnimationType,
    pub card_id: u32,
}

#[derive(Clone, Copy)]
pub enum CardAnimationType {
    Draw,
    Play,
    Discard,
    Shuffle,
}

#[derive(Component)]
pub struct SeasonTransition {
    pub timer: Timer,
    pub from_season: GameState,
    pub to_season: GameState,
    pub overlay_alpha: f32,
}

#[derive(Component, Clone)]
pub struct ParticleEffect {
    pub particles: Vec<Particle>,
    pub effect_type: ParticleType,
    pub timer: Timer,
}

#[derive(Clone)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
}

#[derive(Clone, Copy)]
pub enum ParticleType {
    HarvestSparkles,
    WinePouring,
    LiraGain,
    VictoryPoints,
    Construction,
}

#[derive(Resource)]
pub struct AnimationSettings {
    pub worker_animation_speed: f32,
    pub card_animation_speed: f32,
    pub particle_density: f32,
    pub enable_transitions: bool,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            worker_animation_speed: 1.0,
            card_animation_speed: 1.2,
            particle_density: 0.8,
            enable_transitions: true,
        }
    }
}

// Worker movement animations
pub fn animate_worker_placement(
    commands: &mut Commands,
    worker_entity: Entity,
    start_pos: Vec2,
    target_pos: Vec2,
    animation_type: WorkerAnimationType,
    settings: &AnimationSettings,
) {
    let duration = match animation_type {
        WorkerAnimationType::Placement => 0.4 / settings.worker_animation_speed,
        WorkerAnimationType::Return => 0.3 / settings.worker_animation_speed,
        WorkerAnimationType::Bounce => 0.2 / settings.worker_animation_speed,
    };
    
    commands.entity(worker_entity).insert(WorkerAnimation {
        start_pos,
        target_pos,
        timer: Timer::from_seconds(duration, TimerMode::Once),
        animation_type,
    });
}

pub fn worker_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animated_workers: Query<(Entity, &mut Transform, &mut WorkerAnimation)>,
) {
    for (entity, mut transform, mut animation) in animated_workers.iter_mut() {
        animation.timer.tick(time.delta());
        
        let progress = animation.timer.elapsed_secs() / animation.timer.duration().as_secs_f32();
        let eased_progress = match animation.animation_type {
            WorkerAnimationType::Placement => ease_out_back(progress),
            WorkerAnimationType::Return => ease_in_out_cubic(progress),
            WorkerAnimationType::Bounce => ease_out_bounce(progress),
        };
        
        let current_pos = animation.start_pos.lerp(animation.target_pos, eased_progress);
        transform.translation = current_pos.extend(transform.translation.z);
        
        if animation.timer.finished() {
            // Ensure final position is exact
            transform.translation = animation.target_pos.extend(transform.translation.z);
            commands.entity(entity).remove::<WorkerAnimation>();
        }
    }
}

// Card animations
pub fn animate_card_draw(
    commands: &mut Commands,
    card_type: CardType,
    target_pos: Vec2,
    settings: &AnimationSettings,
) {
    let start_pos = Vec2::new(-600.0, 0.0); // Off-screen left
    let duration = 0.6 / settings.card_animation_speed;
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: match card_type {
                    CardType::Vine => Color::from(Srgba::new(0.2, 0.8, 0.2, 0.9)),
                    CardType::WineOrder => Color::from(Srgba::new(0.6, 0.2, 0.8, 0.9)),
                },
                custom_size: Some(Vec2::new(30.0, 40.0)),
                ..default()
            },
            transform: Transform::from_translation(start_pos.extend(5.0)),
            ..default()
        },
        CardAnimation {
            start_pos,
            target_pos,
            timer: Timer::from_seconds(duration, TimerMode::Once),
            animation_type: CardAnimationType::Draw,
            card_id: 0,
        },
    ));
}

pub fn animate_card_play(
    commands: &mut Commands,
    card_entity: Entity,
    start_pos: Vec2,
    settings: &AnimationSettings,
) {
    let target_pos = Vec2::new(0.0, 200.0); // Center screen
    let duration = 0.4 / settings.card_animation_speed;
    
    commands.entity(card_entity).insert(CardAnimation {
        start_pos,
        target_pos,
        timer: Timer::from_seconds(duration, TimerMode::Once),
        animation_type: CardAnimationType::Play,
        card_id: 0,
    });
}

pub fn card_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animated_cards: Query<(Entity, &mut Transform, &mut CardAnimation, &mut Sprite),Without<MarkedForDespawn>>,
) {
    for (entity, mut transform, mut animation, mut sprite) in animated_cards.iter_mut() {
        animation.timer.tick(time.delta());
        
        let progress = animation.timer.elapsed_secs() / animation.timer.duration().as_secs_f32();
        let eased_progress = match animation.animation_type {
            CardAnimationType::Draw => ease_out_cubic(progress),
            CardAnimationType::Play => ease_in_out_back(progress),
            CardAnimationType::Discard => ease_in_cubic(progress),
            CardAnimationType::Shuffle => ease_in_out_sine(progress),
        };
        
        let current_pos = animation.start_pos.lerp(animation.target_pos, eased_progress);
        transform.translation = current_pos.extend(transform.translation.z);
        
        // Add rotation for play animation
        if matches!(animation.animation_type, CardAnimationType::Play) {
            transform.rotation = Quat::from_rotation_z(progress * 0.1);
        }
        
        // Fade out for discard
        if matches!(animation.animation_type, CardAnimationType::Discard) {
            let mut color = sprite.color.to_srgba();
            color.alpha = 1.0 - progress;
            sprite.color = Color::from(color);
        }
        
        if animation.timer.finished() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

// Seasonal transition effects
pub fn trigger_season_transition(
    commands: &mut Commands,
    from_season: GameState,
    to_season: GameState,
    settings: &AnimationSettings,
) {
    if !settings.enable_transitions {
        return;
    }

    let to_season_clone = to_season.clone();
    let from_season_clone = from_season.clone();

    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            background_color: get_season_color(&to_season_clone).into(),
            z_index: ZIndex::Global(50),
            ..default()
        },
        SeasonTransition {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            from_season,
            to_season,
            overlay_alpha: 0.8,
        },
    ));
    
    // Add season text overlay
    commands.spawn((
        TextBundle::from_section(
            get_season_text(&to_season_clone),
            TextStyle {
                font_size: 48.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(45.0),
            left: Val::Percent(50.0),
            ..default()
        }),
        SeasonTransition {
            timer: Timer::from_seconds(1.5, TimerMode::Once),
            from_season: from_season_clone,
            to_season: to_season_clone,
            overlay_alpha: 1.0,
        },
    ));
}

pub fn season_transition_system(
    mut commands: Commands,
    time: Res<Time>,
    mut transitions: Query<(Entity, &mut SeasonTransition, &mut BackgroundColor), (With<Node>,Without<MarkedForDespawn>)>,
    mut text_transitions: Query<(Entity, &mut SeasonTransition, &mut Text), (With<Text>, Without<Node>, Without<MarkedForDespawn>)>,
) {
    // Handle background transitions
    for (entity, mut transition, mut bg_color) in transitions.iter_mut() {
        transition.timer.tick(time.delta());
        
        let progress = transition.timer.elapsed_secs() / transition.timer.duration().as_secs_f32();
        let alpha = transition.overlay_alpha * (1.0 - ease_in_out_cubic(progress));
        
        let mut color = bg_color.0.to_srgba();
        color.alpha = alpha;
        bg_color.0 = Color::from(color);
        
        if transition.timer.finished() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
    
    // Handle text transitions
    for (entity, mut transition, mut text) in text_transitions.iter_mut() {
        transition.timer.tick(time.delta());
        
        let progress = transition.timer.elapsed_secs() / transition.timer.duration().as_secs_f32();
        
        if progress < 0.3 {
            // Fade in
            let alpha = ease_out_cubic(progress / 0.3);
            let mut color = text.sections[0].style.color.to_srgba();
            color.alpha = alpha;
            text.sections[0].style.color = Color::from(color);
        } else if progress > 0.7 {
            // Fade out
            let alpha = 1.0 - ease_in_cubic((progress - 0.7) / 0.3);
            let mut color = text.sections[0].style.color.to_srgba();
            color.alpha = alpha;
            text.sections[0].style.color = Color::from(color);
        }
        
        if transition.timer.finished() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

// Particle effects
pub fn spawn_harvest_particles(
    commands: &mut Commands,
    position: Vec2,
    grape_count: u8,
    settings: &AnimationSettings,
) {
    let particle_count = (grape_count as f32 * settings.particle_density) as usize;
    let particles = create_harvest_particles(position, particle_count);
    
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(position.extend(3.0)),
            ..default()
        },
        ParticleEffect {
            particles,
            effect_type: ParticleType::HarvestSparkles,
            timer: Timer::from_seconds(2.0, TimerMode::Once),
        },
    ));
}

pub fn spawn_wine_pouring_effect(
    commands: &mut Commands,
    position: Vec2,
    settings: &AnimationSettings,
) {
    let particle_count = (20.0 * settings.particle_density) as usize;
    let particles = create_pouring_particles(position, particle_count);
    
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(position.extend(3.0)),
            ..default()
        },
        ParticleEffect {
            particles,
            effect_type: ParticleType::WinePouring,
            timer: Timer::from_seconds(1.5, TimerMode::Once),
        },
    ));
}

pub fn spawn_lira_particles(
    commands: &mut Commands,
    position: Vec2,
    amount: u8,
    settings: &AnimationSettings,
) {
    let particle_count = (amount as f32 * 2.0 * settings.particle_density) as usize;
    let particles = create_lira_particles(position, particle_count);
    
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(position.extend(3.0)),
            ..default()
        },
        ParticleEffect {
            particles,
            effect_type: ParticleType::LiraGain,
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        },
    ));
}

pub fn particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut particle_effects: Query<(Entity, &mut ParticleEffect, &Transform),Without<MarkedForDespawn>>,
    mut gizmos: Gizmos,
) {
    for (entity, mut effect, transform) in particle_effects.iter_mut() {
        effect.timer.tick(time.delta());
        
        let dt = time.delta_seconds();
        let effect_clone = effect.clone();

        // Update particles
        for particle in &mut effect.particles {
            particle.position += particle.velocity * dt;
            particle.velocity.y -= 150.0 * dt; // Gravity
            particle.life -= dt;

            // Apply effect-specific behaviors
            match effect_clone.effect_type {
                ParticleType::HarvestSparkles => {
                    particle.velocity *= 0.98; // Slow down
                }
                ParticleType::WinePouring => {
                    particle.velocity.x *= 0.95;
                }
                ParticleType::LiraGain => {
                    particle.velocity.y += 100.0 * dt; // Float upward
                }
                _ => {}
            }
        }
        
        // Remove dead particles
        effect.particles.retain(|p| p.life > 0.0);
        
        // Draw particles using gizmos
        for particle in &effect.particles {
            let alpha = particle.life / particle.max_life;
            let mut color = particle.color.to_srgba();
            color.alpha = alpha;
            let final_color = Color::from(color);
            
            let world_pos = transform.translation.truncate() + particle.position;
            gizmos.circle_2d(world_pos, particle.size, final_color);
        }
        
        // Clean up finished effects
        if effect.timer.finished() || effect.particles.is_empty() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

// Utility functions
fn create_harvest_particles(center: Vec2, count: usize) -> Vec<Particle> {
    use rand::Rng;
    let mut rng = rand::rng();
    
    (0..count)
        .map(|_| {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let speed = rng.random_range(50.0..150.0);
            let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
            
            Particle {
                position: Vec2::new(
                    rng.random_range(-10.0..10.0),
                    rng.random_range(-10.0..10.0),
                ),
                velocity,
                life: rng.random_range(1.0..2.0),
                max_life: 2.0,
                size: rng.random_range(2.0..4.0),
                color: Color::from(Srgba::new(1.0, 1.0, 0.0, 1.0)), // Gold sparkles
            }
        })
        .collect()
}

fn create_pouring_particles(center: Vec2, count: usize) -> Vec<Particle> {
    use rand::Rng;
    let mut rng = rand::rng();
    
    (0..count)
        .map(|_| {
            let velocity = Vec2::new(
                rng.random_range(-20.0..20.0),
                rng.random_range(-100.0..-50.0),
            );
            
            Particle {
                position: Vec2::new(rng.random_range(-5.0..5.0), 0.0),
                velocity,
                life: rng.random_range(0.8..1.5),
                max_life: 1.5,
                size: rng.random_range(1.0..3.0),
                color: Color::from(Srgba::new(0.5, 0.0, 0.5, 1.0)), // Purple wine
            }
        })
        .collect()
}

fn create_lira_particles(center: Vec2, count: usize) -> Vec<Particle> {
    use rand::Rng;
    let mut rng = rand::rng();
    
    (0..count)
        .map(|_| {
            let velocity = Vec2::new(
                rng.random_range(-30.0..30.0),
                rng.random_range(20.0..80.0),
            );
            
            Particle {
                position: Vec2::new(rng.random_range(-8.0..8.0), 0.0),
                velocity,
                life: rng.random_range(0.8..1.2),
                max_life: 1.2,
                size: rng.random_range(1.5..3.0),
                color: Color::from(Srgba::new(1.0, 0.84, 0.0, 1.0)), // Gold
            }
        })
        .collect()
}

fn get_season_color(season: &GameState) -> Color {
    match season {
        GameState::Spring => Color::from(Srgba::new(0.4, 0.8, 0.4, 0.8)),
        GameState::Summer => Color::from(Srgba::new(1.0, 1.0, 0.3, 0.8)),
        GameState::Fall => Color::from(Srgba::new(0.8, 0.4, 0.2, 0.8)),
        GameState::Winter => Color::from(Srgba::new(0.3, 0.3, 0.8, 0.8)),
        _ => Color::from(Srgba::new(0.2, 0.2, 0.2, 0.8)),
    }
}

fn get_season_text(season: &GameState) -> String {
    match season {
        GameState::Spring => "🌸 SPRING 🌸".to_string(),
        GameState::Summer => "☀️ SUMMER ☀️".to_string(),
        GameState::Fall => "🍂 FALL 🍂".to_string(),
        GameState::Winter => "❄️ WINTER ❄️".to_string(),
        _ => "GAME PHASE".to_string(),
    }
}

// Easing functions
fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

fn ease_out_bounce(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;

    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

fn ease_in_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c2 = c1 * 1.525;

    if t < 0.5 {
        ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
    } else {
        ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
    }
}

fn ease_in_out_sine(t: f32) -> f32 {
    (-(std::f32::consts::PI * t).cos() - 1.0) / 2.0
}

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
    mut animated_texts: Query<(Entity, &mut Transform, &mut AnimatedText, &mut Text),Without<MarkedForDespawn>>,
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
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}