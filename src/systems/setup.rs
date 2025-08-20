use bevy::prelude::*;
use crate::components::*;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = GameAssets {
        worker_texture: asset_server.load("worker.png"),
        vine_card_texture: asset_server.load("vine_card.png"),
        wine_order_card_texture: asset_server.load("wine_order.png"),
        field_texture: asset_server.load("field.png"),
    };
    commands.insert_resource(assets);
}

pub fn setup_game_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    config: Res<GameConfig>,
    mut turn_order: ResMut<TurnOrder>,
    mut card_decks: ResMut<CardDecks>,
    text_query: Query<Entity, With<Text>>,
    existing_entities: Query<Entity, (With<PlayerId>, Without<Camera>)>,
) {
    // Clean up existing entities
    for entity in text_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn();
    }
    
    turn_order.players.clear();
    
    // Prepare Mama & Papa cards
    let mut mama_cards = card_decks.mama_cards.clone();
    let mut papa_cards = card_decks.papa_cards.clone();
    use rand::seq::SliceRandom;
    let mut rng = rand::rng();
    mama_cards.shuffle(&mut rng);
    papa_cards.shuffle(&mut rng);
    
    // Create players with Mama & Papa cards
    for i in 0..config.player_count {
        let is_ai = i >= (config.player_count - config.ai_count);
        let name = if is_ai {
            format!("AI Player {}", i + 1)
        } else {
            format!("Player {}", i + 1)
        };
        
        // Assign cards
        let mama_card = mama_cards.get(i as usize).cloned()
            .unwrap_or_else(|| mama_cards[0].clone());
        let papa_card = papa_cards.get(i as usize).cloned()
            .unwrap_or_else(|| papa_cards[0].clone());
        
        // Create player with bonuses
        let mut player = Player::new(i, name, is_ai);
        player.lira += mama_card.bonus_lira;
        player.workers += mama_card.bonus_workers;
        player.victory_points += papa_card.bonus_vp;
        
        let mut vineyard = Vineyard::new(PlayerId(i));
        vineyard.lira += mama_card.bonus_lira;
        
        // Add bonus fields if any
        if papa_card.bonus_fields > 0 {
            vineyard.fields[8] = VineyardField::new(FieldType::Premium);
        }
        
        let mut hand = Hand::new(PlayerId(i));
        
        // Add bonus vine cards from Mama
        for _ in 0..mama_card.bonus_vine_cards {
            if let Some(vine_card) = card_decks.draw_vine_card() {
                hand.vine_cards.push(vine_card);
            }
        }
        
        let mama_card_clone = mama_card.clone();
        let papa_card_clone = papa_card.clone();

        commands.spawn(player);
        commands.spawn(vineyard);
        commands.spawn(hand);
        commands.spawn(mama_card);
        commands.spawn(papa_card);
        
        // Create starting structures from Papa card
        for structure_type in papa_card_clone.starting_structures {
            commands.spawn(Structure {
                structure_type,
                owner: PlayerId(i),
            });
        }
        
        // Create workers (exactly 2 regular workers per player)
        for w in 0..2 {
            let worker_pos = Vec2::new(-500.0 + (i as f32 * 120.0), -200.0 + (w as f32 * 30.0));
            commands.spawn((
                Worker::new(PlayerId(i), false, worker_pos),
                Clickable { size: Vec2::new(20.0, 20.0) },
            ));
        }
        
        // Create bonus workers from Mama card
        for w in 0..mama_card_clone.bonus_workers {
            let worker_pos = Vec2::new(-500.0 + (i as f32 * 120.0), -140.0 + (w as f32 * 30.0));
            commands.spawn((
                Worker::new(PlayerId(i), false, worker_pos),
                Clickable { size: Vec2::new(20.0, 20.0) },
            ));
        }
        
        // Create exactly 1 grande worker per player
        let grande_pos = Vec2::new(-500.0 + (i as f32 * 120.0), -170.0);
        commands.spawn((
            Worker::new(PlayerId(i), true, grande_pos),
            Clickable { size: Vec2::new(25.0, 25.0) },
        ));
        
        turn_order.players.push(PlayerId(i));
    }
    
    // Create action board
    let action_board = ActionBoard::new();
    for space in action_board.spaces.clone() {
        commands.spawn((
            space,
            Clickable { size: Vec2::new(60.0, 30.0) },
        ));
    }
    commands.spawn(action_board);
    
    info!("Enhanced game setup complete: {} players ({} AI) with Mama & Papa cards", 
          config.player_count, config.ai_count);
    next_state.set(GameState::Spring);
}

// Update setup to include residual payment trackers
pub fn setup_residual_payment_system(
    mut commands: Commands,
    players: Query<&Player>,
) {
    for player in players.iter() {
        commands.spawn(ResidualPaymentTracker::new(player.id));
    }
}