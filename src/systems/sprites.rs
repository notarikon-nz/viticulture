use bevy::prelude::*;
use crate::components::*;

const GREY: Srgba = Srgba::new(0.6, 0.6, 0.6, 1.0);

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
    // Clear existing sprites to refresh
    for entity in worker_sprites.iter() {
        commands.entity(entity).despawn();
    }
    for entity in vineyard_sprites.iter() {
        commands.entity(entity).despawn();
    }
    for entity in card_sprites.iter() {
        commands.entity(entity).despawn();
    }
    
    // Update worker sprites with grande worker distinction
    for worker in workers.iter() {
        let player_colors = [
            Color::from(Srgba::RED),
            Color::from(Srgba::BLUE),
            Color::from(Srgba::GREEN),
            Color::from(Srgba::new(1.0, 0.0, 1.0, 1.0)), // Magenta
        ];
        
        let color_grey = Color::from(GREY);
        let color = player_colors.get(worker.owner.0 as usize).unwrap_or(&color_grey);
        
        // Make grande workers brighter/more prominent
        let final_color = if worker.is_grande {
            Color::from(Srgba::new(color.to_srgba().red * 1.2, color.to_srgba().green * 1.2, color.to_srgba().blue * 1.2, 1.0))
        } else {
            *color
        };
        
        let size = if worker.is_grande { 
            Vec2::new(20.0, 20.0) // Grande workers are larger
        } else { 
            Vec2::new(16.0, 16.0) 
        };
        
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
    
    // Update vineyard field sprites
    for vineyard in vineyards.iter() {
        for (field_idx, field) in vineyard.fields.iter().enumerate() {
            let field_x = -200.0 + ((field_idx % 3) as f32 * 40.0);
            let field_y = 100.0 - ((field_idx / 3) as f32 * 40.0);
            let field_pos = Vec2::new(field_x + (vineyard.owner.0 as f32 * 200.0), field_y);
            
            let field_color = match field {
                Some(VineType::Red(_)) => Color::from(Srgba::new(0.8, 0.2, 0.2, 1.0)),
                Some(VineType::White(_)) => Color::from(Srgba::new(0.9, 0.9, 0.7, 1.0)),
                None => Color::from(Srgba::new(0.4, 0.3, 0.2, 0.8)),
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
    
    // Update card sprites for current player's hand
    if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
        if let Some(hand) = hands.iter().find(|h| h.owner == *current_player_id) {
            let hand_y = -200.0;
            let mut card_x = -300.0;
            
            // Vine cards
            for (i, _vine_card) in hand.vine_cards.iter().enumerate() {
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
            
            card_x += hand.vine_cards.len() as f32 * 35.0 + 20.0;
            
            // Wine order cards
            for (i, _order_card) in hand.wine_order_cards.iter().enumerate() {
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
    }
}