use bevy::prelude::*;
use crate::components::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin,DiagnosticsStore};

const GREY: Srgba = Srgba::new(0.6, 0.6, 0.6, 1.0);

#[derive(Resource)]
pub struct PerformanceSettings {
    pub enable_sprite_culling: bool,
    pub limit_animations: bool,
    pub cache_ui_updates: bool,
    pub debug_performance: bool,
}

#[derive(Resource, Default)]
pub struct FrameCache {
    pub last_ui_update: f32,
    pub last_sprite_update: f32,
    pub cached_player_data: Vec<(PlayerId, u8, u8, u8, u8)>, // (id, vp, lira, workers, cards)
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            enable_sprite_culling: true,
            limit_animations: false,
            cache_ui_updates: true,
            debug_performance: false,
        }
    }
}

// Optimized UI update system - only updates when data changes
pub fn cached_ui_update_system(
    mut cache: ResMut<FrameCache>,
    performance: Res<PerformanceSettings>,
    time: Res<Time>,
    players: Query<&Player>,
    hands: Query<&Hand>,
    vineyards: Query<&Vineyard>,
    mut status_query: Query<&mut Text, (With<GameStatusText>, Without<TurnIndicator>)>,
    turn_order: Res<TurnOrder>,
    config: Res<GameConfig>,
) {
    if !performance.cache_ui_updates {
        return;
    }
    
    // Only update UI every 0.1 seconds or when data changes
    let current_time = time.elapsed_seconds();
    if current_time - cache.last_ui_update < 0.1 {
        return;
    }
    
    // Check if player data has changed
    let current_data: Vec<_> = players.iter()
        .map(|p| {
            let hand = hands.iter().find(|h| h.owner == p.id);
            let vineyard = vineyards.iter().find(|v| v.owner == p.id);
            let total_cards = hand.map(|h| h.vine_cards.len() + h.wine_order_cards.len()).unwrap_or(0) as u8;
            let total_resources = vineyard.map(|v| v.red_grapes + v.white_grapes + v.red_wine + v.white_wine).unwrap_or(0);
            
            (p.id, p.victory_points, p.lira, p.workers, total_cards + total_resources)
        })
        .collect();
    
    // Only update if data changed
    if current_data != cache.cached_player_data {
        cache.cached_player_data = current_data;
        cache.last_ui_update = current_time;
        
        // Update status text
        if let Ok(mut status_text) = status_query.get_single_mut() {
            let mut leading_player = "None";
            let mut highest_vp = 0;
            
            for player in players.iter() {
                if player.victory_points > highest_vp {
                    highest_vp = player.victory_points;
                    leading_player = &player.name;
                }
            }
            
            status_text.sections[0].value = format!(
                "Year {} | Leader: {} ({} VP) | Target: {} VP",
                config.current_year, leading_player, highest_vp, config.target_victory_points
            );
        }
    }
}

// Optimized sprite system - only updates visible sprites
pub fn culled_sprite_system(
    mut commands: Commands,
    performance: Res<PerformanceSettings>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    workers: Query<&Worker>,
    vineyards: Query<&Vineyard>,
    hands: Query<&Hand>,
    worker_sprites: Query<Entity, With<WorkerSprite>>,
    vineyard_sprites: Query<Entity, With<VineyardSprite>>,
    card_sprites: Query<Entity, With<CardSprite>>,
    turn_order: Res<TurnOrder>,
    mut cache: ResMut<FrameCache>,
    time: Res<Time>,
) {
    if !performance.enable_sprite_culling {
        return;
    }
    
    // Limit sprite updates to 30 FPS for performance
    let current_time = time.elapsed_seconds();
    if current_time - cache.last_sprite_update < 0.033 {
        return;
    }
    cache.last_sprite_update = current_time;
    
    // Get visible area from camera
    let (camera, camera_transform) = camera_q.single();
    let window = windows.single();
    let viewport_size = Vec2::new(window.width(), window.height());
    
    // Clear existing sprites
    for entity in worker_sprites.iter() {
        commands.entity(entity).despawn();
    }
    for entity in vineyard_sprites.iter() {
        commands.entity(entity).despawn();
    }
    for entity in card_sprites.iter() {
        commands.entity(entity).despawn();
    }
    
    // Only render workers that are visible
    for worker in workers.iter() {
        if is_position_visible(worker.position, camera_transform.translation().truncate(), viewport_size) {
            spawn_worker_sprite(&mut commands, worker);
        }
    }
    
    // Only render vineyard fields for current player (optimization)
    if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
        if let Some(vineyard) = vineyards.iter().find(|v| v.owner == *current_player_id) {
            spawn_vineyard_sprites(&mut commands, vineyard);
        }
    }
    
    // Only render current player's hand
    if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
        if let Some(hand) = hands.iter().find(|h| h.owner == *current_player_id) {
            spawn_hand_sprites(&mut commands, hand);
        }
    }
}

fn is_position_visible(pos: Vec2, camera_pos: Vec2, viewport_size: Vec2) -> bool {
    let half_viewport = viewport_size * 0.5;
    let min = camera_pos - half_viewport;
    let max = camera_pos + half_viewport;
    
    pos.x >= min.x && pos.x <= max.x && pos.y >= min.y && pos.y <= max.y
}

fn spawn_worker_sprite(commands: &mut Commands, worker: &Worker) {
    let player_colors = [
        Color::from(Srgba::RED),
        Color::from(Srgba::BLUE),
        Color::from(Srgba::GREEN),
        Color::from(Srgba::new(1.0, 0.0, 1.0, 1.0)),
    ];
    
    let color_grey = Color::from(GREY);
    let color = player_colors.get(worker.owner.0 as usize).unwrap_or(&color_grey);
    let final_color = if worker.is_grande {
        Color::from(Srgba::new(color.to_srgba().red * 1.2, color.to_srgba().green * 1.2, color.to_srgba().blue * 1.2, 1.0))
    } else {
        *color
    };
    
    let size = if worker.is_grande { Vec2::new(20.0, 20.0) } else { Vec2::new(16.0, 16.0) };
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: final_color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(worker.position.extend(if worker.is_grande { 1.5 } else { 1.0 })),
            ..default()
        },
        WorkerSprite { player_id: worker.owner },
    ));
}

fn spawn_vineyard_sprites(commands: &mut Commands, vineyard: &Vineyard) {
    for (field_idx, field) in vineyard.fields.iter().enumerate() {
        let field_x = -200.0 + ((field_idx % 3) as f32 * 40.0);
        let field_y = 100.0 - ((field_idx / 3) as f32 * 40.0);
        let field_pos = Vec2::new(field_x + (vineyard.owner.0 as f32 * 200.0), field_y);
        
        // FIXED: Access the vine field properly
        let field_color = match field.vine {  // Changed from field to field.vine
            Some(VineType::Red(_)) => Color::from(Srgba::new(0.8, 0.2, 0.2, 1.0)),
            Some(VineType::White(_)) => Color::from(Srgba::new(0.9, 0.9, 0.7, 1.0)),
            None => {
                // Base color depends on field type
                match field.field_type {
                    FieldType::Premium => Color::from(Srgba::new(0.5, 0.4, 0.2, 0.8)), // Rich soil
                    FieldType::Poor => Color::from(Srgba::new(0.3, 0.3, 0.3, 0.8)),    // Rocky soil
                    FieldType::Standard => Color::from(Srgba::new(0.4, 0.3, 0.2, 0.8)), // Normal soil
                }
            },
        };
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: field_color,
                    custom_size: Some(Vec2::new(35.0, 35.0)),
                    ..default()
                },
                transform: Transform::from_translation(field_pos.extend(0.5)),
                ..default()
            },
            VineyardSprite { 
                player_id: vineyard.owner,
                field_index: field_idx,
            },
        ));
    }
}

fn spawn_hand_sprites(commands: &mut Commands, hand: &Hand) {
    let hand_y = -200.0;
    let mut card_x = -300.0;
    
    // Limit card display for performance
    let max_cards_to_show = 8;
    
    for (i, _) in hand.vine_cards.iter().take(max_cards_to_show).enumerate() {
        let card_pos = Vec2::new(card_x + (i as f32 * 35.0), hand_y);
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::from(Srgba::new(0.2, 0.8, 0.2, 0.9)),
                    custom_size: Some(Vec2::new(30.0, 40.0)),
                    ..default()
                },
                transform: Transform::from_translation(card_pos.extend(2.0)),
                ..default()
            },
            CardSprite { card_type: CardType::Vine },
        ));
    }
    
    card_x += hand.vine_cards.len().min(max_cards_to_show) as f32 * 35.0 + 20.0;
    
    for (i, _) in hand.wine_order_cards.iter().take(max_cards_to_show).enumerate() {
        let card_pos = Vec2::new(card_x + (i as f32 * 35.0), hand_y);
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::from(Srgba::new(0.6, 0.2, 0.8, 0.9)),
                    custom_size: Some(Vec2::new(30.0, 40.0)),
                    ..default()
                },
                transform: Transform::from_translation(card_pos.extend(2.0)),
                ..default()
            },
            CardSprite { card_type: CardType::WineOrder },
        ));
    }
}

pub fn performance_monitor_system(
    time: Res<Time>,
    performance: Res<PerformanceSettings>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if !performance.debug_performance {
        return;
    }
    
    // Log FPS every 5 seconds
    if time.elapsed_seconds() % 5.0 < 0.1 {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                info!("Average FPS: {:.1}", average);
                if average < 30.0 {
                    warn!("Low FPS detected! Consider enabling performance optimizations.");
                }
            }
        }
    }
}

