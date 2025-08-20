use bevy::prelude::*;
use crate::components::*;

const GREY: Srgba = Srgba::new(0.6, 0.6, 0.6, 1.0);

// =============================================================================
// FIX IN: src/systems/validation.rs around line 235
// =============================================================================

// Find the balance_card_distribution function and update VineCard creation:
pub fn balance_card_distribution(card_decks: &mut ResMut<CardDecks>) {
    // Ensure balanced vine card distribution
    let mut red_count = 0;
    let mut white_count = 0;
    
    for card in &card_decks.vine_deck {
        match card.vine_type {
            VineType::Red(_) => red_count += 1,
            VineType::White(_) => white_count += 1,
        }
    }
    
    // Rebalance if needed (should be roughly 50/50)
    let total = red_count + white_count;
    if total > 0 && (red_count as f32 / total as f32) < 0.4 {
        // Add more red cards - FIXED: Include all required fields
        for i in 200..205 {
            card_decks.vine_deck.push(VineCard {
                id: i,
                vine_type: VineType::Red(2),
                cost: 1,
                art_style: CardArt::BasicRed,  // ADDED: Missing field
                special_ability: None,         // ADDED: Missing field
            });
        }
    }
    
    // Shuffle decks for randomness
    use rand::seq::SliceRandom;
    let mut rng = rand::rng();
    card_decks.vine_deck.shuffle(&mut rng);
    card_decks.wine_order_deck.shuffle(&mut rng);
}

// =============================================================================
// FIX IN: src/systems/performance.rs around lines 204-206
// =============================================================================

// Find the spawn_vineyard_sprites function and fix the field assignment:
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

// =============================================================================
// ALSO UPDATE IN: src/systems/sprites.rs (if it has similar vineyard rendering)
// =============================================================================

// Update the vineyard field rendering in update_sprites_system:
pub fn update_sprites_system(
    mut commands: Commands,
    workers: Query<&Worker>,
    vineyards: Query<&Vineyard>,
    hands: Query<&Hand>,
    worker_sprites: Query<Entity, With<WorkerSprite>>,
    vineyard_sprites: Query<Entity, With<VineyardSprite>>,
    card_sprites: Query<Entity, With<CardSprite>>,
    turn_order: Res<TurnOrder>,
) {
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
    
    // Enhanced worker sprites with better distinction
    for worker in workers.iter() {
        let player_colors = [
            Color::srgb(0.8, 0.2, 0.2), // Red
            Color::srgb(0.2, 0.2, 0.8), // Blue  
            Color::srgb(0.2, 0.8, 0.2), // Green
            Color::srgb(0.8, 0.8, 0.2), // Yellow
        ];
        
        let color_grey = Color::srgb(0.6, 0.6, 0.6);
        let base_color = player_colors.get(worker.owner.0 as usize)
            .unwrap_or(&color_grey);
        
        // Enhanced visual distinction for grande workers
        let (final_color, size, z_index) = if worker.is_grande {
            let bright_color = Color::srgb(
                (base_color.to_srgba().red * 1.3).min(1.0),
                (base_color.to_srgba().green * 1.3).min(1.0),
                (base_color.to_srgba().blue * 1.3).min(1.0)
            );
            (bright_color, Vec2::new(24.0, 24.0), 2.0)
        } else {
            (*base_color, Vec2::new(18.0, 18.0), 1.0)
        };
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: final_color,
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform::from_translation(worker.position.extend(z_index)),
                ..default()
            },
            WorkerSprite { player_id: worker.owner },
        ));
        
        // Add border for grande workers
        if worker.is_grande {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(1.0, 1.0, 0.8),
                        custom_size: Some(Vec2::new(28.0, 28.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(worker.position.extend(z_index - 0.1)),
                    ..default()
                },
                WorkerSprite { player_id: worker.owner },
            ));
        }
    }
    
    // Enhanced vineyard visualization - FIXED field access
    for vineyard in vineyards.iter() {
        for (field_idx, field) in vineyard.fields.iter().enumerate() {
            let field_x = -200.0 + ((field_idx % 3) as f32 * 45.0);
            let field_y = 100.0 - ((field_idx / 3) as f32 * 45.0);
            let field_pos = Vec2::new(field_x + (vineyard.owner.0 as f32 * 220.0), field_y);
            
            // Base field color based on field type
            let base_color = match field.field_type {
                FieldType::Premium => Color::srgb(0.5, 0.4, 0.2), // Rich soil
                FieldType::Poor => Color::srgb(0.3, 0.3, 0.3),    // Rocky soil
                FieldType::Standard => Color::srgb(0.4, 0.3, 0.2), // Normal soil
            };
            
            // Field background
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: base_color,
                        custom_size: Some(Vec2::new(40.0, 40.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(field_pos.extend(0.1)),
                    ..default()
                },
                VineyardSprite { 
                    player_id: vineyard.owner,
                    field_index: field_idx,
                },
            ));
            
            // Vine visualization if planted - FIXED: Check field.vine instead of field
            if let Some(vine) = field.vine {
                let vine_color = match vine {
                    VineType::Red(_) => Color::srgb(0.7, 0.1, 0.1),
                    VineType::White(_) => Color::srgb(0.9, 0.9, 0.6),
                };
                
                // Vine sprite
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: vine_color,
                            custom_size: Some(Vec2::new(30.0, 30.0)),
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
                
                // Value indicator
                let value = field.get_harvest_value();
                if value > 0 {
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::srgb(1.0, 1.0, 1.0),
                                custom_size: Some(Vec2::new(8.0, 8.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(field_pos.extend(0.8) + Vec3::new(12.0, 12.0, 0.0)),
                            ..default()
                        },
                        VineyardSprite { 
                            player_id: vineyard.owner,
                            field_index: field_idx,
                        },
                    ));
                }
            }
            
            // Field type indicator
            let indicator_color = match field.field_type {
                FieldType::Premium => Some(Color::srgb(1.0, 0.8, 0.0)),
                FieldType::Poor => Some(Color::srgb(0.6, 0.6, 0.6)),
                FieldType::Standard => None,
            };
            
            if let Some(color) = indicator_color {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color,
                            custom_size: Some(Vec2::new(6.0, 6.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(field_pos.extend(0.3) + Vec3::new(-15.0, -15.0, 0.0)),
                        ..default()
                    },
                    VineyardSprite { 
                        player_id: vineyard.owner,
                        field_index: field_idx,
                    },
                ));
            }
        }
    }
    
    // Enhanced card sprites with better art (rest remains the same...)
    if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
        if let Some(hand) = hands.iter().find(|h| h.owner == *current_player_id) {
            let hand_y = -200.0;
            let mut card_x = -350.0;
            
            // Vine cards with enhanced visuals
            for (i, vine_card) in hand.vine_cards.iter().enumerate() {
                let card_pos = Vec2::new(card_x + (i as f32 * 38.0), hand_y);
                
                // Card background
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: vine_card.art_style.get_color(),
                            custom_size: Some(Vec2::new(32.0, 42.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(card_pos.extend(2.0)),
                        ..default()
                    },
                    CardSprite { card_type: CardType::Vine },
                ));
                
                // Card border
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: vine_card.art_style.get_border_color(),
                            custom_size: Some(Vec2::new(36.0, 46.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(card_pos.extend(1.9)),
                        ..default()
                    },
                    CardSprite { card_type: CardType::Vine },
                ));
                
                // Cost indicator
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgb(1.0, 1.0, 1.0),
                            custom_size: Some(Vec2::new(8.0, 8.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(card_pos.extend(2.1) + Vec3::new(-12.0, 15.0, 0.0)),
                        ..default()
                    },
                    CardSprite { card_type: CardType::Vine },
                ));
            }
            
            card_x += hand.vine_cards.len() as f32 * 38.0 + 25.0;
            
            // Wine order cards with enhanced visuals
            for (i, order_card) in hand.wine_order_cards.iter().enumerate() {
                let card_pos = Vec2::new(card_x + (i as f32 * 38.0), hand_y);
                
                // Card background
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: order_card.art_style.get_color(),
                            custom_size: Some(Vec2::new(32.0, 42.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(card_pos.extend(2.0)),
                        ..default()
                    },
                    CardSprite { card_type: CardType::WineOrder },
                ));
                
                // Card border
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: order_card.art_style.get_border_color(),
                            custom_size: Some(Vec2::new(36.0, 46.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(card_pos.extend(1.9)),
                        ..default()
                    },
                    CardSprite { card_type: CardType::WineOrder },
                ));
                
                // VP indicator
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgb(1.0, 1.0, 0.0),
                            custom_size: Some(Vec2::new(10.0, 10.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(card_pos.extend(2.1) + Vec3::new(12.0, 15.0, 0.0)),
                        ..default()
                    },
                    CardSprite { card_type: CardType::WineOrder },
                ));
            }
        }
    }
}