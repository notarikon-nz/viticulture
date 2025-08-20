use bevy::prelude::*;
use crate::components::*;

// Fix edge cases and potential bugs in the game
pub fn fix_worker_state_system(
    mut workers: Query<&mut Worker>,
    action_spaces: Query<&ActionSpaceSlot>,
    turn_order: Res<TurnOrder>,
) {
    // Fix workers that are placed but space is no longer occupied
    for mut worker in workers.iter_mut() {
        if let Some(placed_action) = worker.placed_at {
            let space_still_occupied = action_spaces.iter()
                .any(|space| space.action == placed_action && 
                     (space.occupied_by == Some(worker.owner) || 
                      space.bonus_worker_slot == Some(worker.owner)));
            
            if !space_still_occupied {
                warn!("Fixed orphaned worker for player {:?}", worker.owner);
                worker.placed_at = None;
                // Reset worker position
                let player_id = worker.owner.0;
                let y_offset = if worker.is_grande { -170.0 } else { -200.0 };
                worker.position = Vec2::new(-500.0 + (player_id as f32 * 100.0), y_offset);
            }
        }
    }
}

pub fn fix_card_deck_system(
    mut card_decks: ResMut<CardDecks>,
) {

    let mut card_decks_clone = card_decks.clone();

    // Prevent empty deck issues by reshuffling discard pile
    if card_decks.vine_deck.is_empty() && !card_decks.vine_discard.is_empty() {
        info!("Reshuffling vine discard pile into deck");
        card_decks.vine_deck.append(&mut card_decks_clone.vine_discard);
        
        // Shuffle the deck
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        card_decks.vine_deck.shuffle(&mut rng);
    }
    
    let mut card_decks_clone_again = card_decks.clone();

    if card_decks.wine_order_deck.is_empty() && !card_decks.wine_order_discard.is_empty() {
        info!("Reshuffling wine order discard pile into deck");
        card_decks.wine_order_deck.append(&mut card_decks_clone_again.wine_order_discard);
        
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        card_decks.wine_order_deck.shuffle(&mut rng);
    }
}

pub fn fix_resource_overflow_system(
    mut players: Query<&mut Player>,
    mut vineyards: Query<&mut Vineyard>,
) {
    // Prevent resource overflow and underflow
    for mut player in players.iter_mut() {
        player.victory_points = player.victory_points.min(99); // Cap at 99 VP
        player.lira = player.lira.min(50); // Cap at 50 lira
        player.workers = player.workers.max(1).min(10); // Min 1, max 10 workers
    }
    
    for mut vineyard in vineyards.iter_mut() {
        // Cap grapes and wine at reasonable limits
        vineyard.red_grapes = vineyard.red_grapes.min(20);
        vineyard.white_grapes = vineyard.white_grapes.min(20);
        vineyard.red_wine = vineyard.red_wine.min(20);
        vineyard.white_wine = vineyard.white_wine.min(20);
        vineyard.lira = vineyard.lira.min(50);
    }
}

pub fn fix_turn_order_system(
    mut turn_order: ResMut<TurnOrder>,
    players: Query<&Player>,
) {
    let player_count = players.iter().count();
    
    // Fix out-of-bounds current player
    if turn_order.current_player >= player_count {
        warn!("Fixed out-of-bounds current player index");
        turn_order.current_player = 0;
    }
    
    // Ensure all players are in turn order
    let existing_players: std::collections::HashSet<_> = turn_order.players.iter().collect();
    let all_players: std::collections::HashSet<_> = players.iter().map(|p| &p.id).collect();
    
    if existing_players != all_players {
        warn!("Fixed mismatched turn order players");
        turn_order.players = players.iter().map(|p| p.id).collect();
    }
}

pub fn fix_action_space_consistency_system(
    mut action_spaces: Query<&mut ActionSpaceSlot>,
    workers: Query<&Worker>,
) {
    // Fix action spaces that claim to be occupied but have no worker
    for mut space in action_spaces.iter_mut() {
        if let Some(occupying_player) = space.occupied_by {
            let worker_present = workers.iter()
                .any(|w| w.owner == occupying_player && w.placed_at == Some(space.action));
            
            if !worker_present {
                warn!("Fixed ghost occupation on action space {:?}", space.action);
                space.occupied_by = None;
            }
        }
        
        if let Some(bonus_player) = space.bonus_worker_slot {
            let grande_worker_present = workers.iter()
                .any(|w| w.owner == bonus_player && w.is_grande && w.placed_at == Some(space.action));
            
            if !grande_worker_present {
                warn!("Fixed ghost bonus occupation on action space {:?}", space.action);
                space.bonus_worker_slot = None;
            }
        }
    }
}

pub fn validate_game_state_system(
    players: Query<&Player>,
    vineyards: Query<&Vineyard>,
    hands: Query<&Hand>,
    workers: Query<&Worker>,
    config: Res<GameConfig>,
) {
    // Basic sanity checks
    let player_count = players.iter().count();
    let vineyard_count = vineyards.iter().count();
    let hand_count = hands.iter().count();
    
    if player_count != vineyard_count || player_count != hand_count {
        error!("Mismatched component counts: {} players, {} vineyards, {} hands", 
               player_count, vineyard_count, hand_count);
    }
    
    if player_count != config.player_count as usize {
        warn!("Player count mismatch: expected {}, found {}", config.player_count, player_count);
    }
    
    // Check worker consistency
    for player in players.iter() {
        let player_workers = workers.iter().filter(|w| w.owner == player.id).count();
        let expected_workers = player.workers as usize + 1; // +1 for grande worker
        
        if player_workers != expected_workers {
            warn!("Worker count mismatch for player {:?}: expected {}, found {}", 
                  player.id, expected_workers, player_workers);
        }
    }
}

pub fn emergency_recovery_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
) {
    // Emergency recovery: F12 to return to main menu
    if keyboard.just_pressed(KeyCode::F12) {
        warn!("Emergency recovery triggered - returning to main menu");
        
        // Clear all entities except camera
        let entities_to_clear = commands.spawn_empty().id();
        commands.entity(entities_to_clear).despawn();
        
        next_state.set(GameState::MainMenu);
    }
    
    // Auto-recovery for stuck states
    if matches!(current_state.get(), GameState::Setup | GameState::Spring | GameState::Fall) {
        // Add timeout logic here if needed
    }
}

// Improved validation with better error handling
pub fn enhanced_action_validation(
    player_id: PlayerId,
    action: ActionSpace,
    workers: &Query<&Worker>,
    vineyards: &Query<&Vineyard>,
    hands: &Query<&Hand>,
    players: &Query<&Player>,
) -> Result<(), String> {
    // Check if player exists
    let player = players.iter().find(|p| p.id == player_id)
        .ok_or("Player not found")?;
    
    let vineyard = vineyards.iter().find(|v| v.owner == player_id)
        .ok_or("Vineyard not found")?;
    
    let hand = hands.iter().find(|h| h.owner == player_id)
        .ok_or("Hand not found")?;
    
    // Check worker availability
    let available_workers = workers.iter()
        .filter(|w| w.owner == player_id && w.placed_at.is_none())
        .count();
    
    if available_workers == 0 {
        return Err("No available workers".to_string());
    }
    
    // Action-specific validation
    match action {
        ActionSpace::PlantVine => {
            if hand.vine_cards.is_empty() {
                return Err("No vine cards to plant".to_string());
            }
            if vineyard.lira == 0 {
                return Err("Not enough lira".to_string());
            }
            let empty_fields = vineyard.fields.iter().filter(|f| f.is_none()).count();
            if empty_fields == 0 {
                return Err("No empty fields".to_string());
            }
        }
        ActionSpace::FillOrder => {
            if hand.wine_order_cards.is_empty() {
                return Err("No wine orders".to_string());
            }
            let can_fulfill = hand.wine_order_cards.iter()
                .any(|order| vineyard.can_fulfill_order(order));
            if !can_fulfill {
                return Err("Cannot fulfill any orders".to_string());
            }
        }
        ActionSpace::TrainWorker => {
            if vineyard.lira < 4 {
                return Err("Need 4 lira to train worker".to_string());
            }
        }
        ActionSpace::MakeWine => {
            let total_grapes = vineyard.red_grapes + vineyard.white_grapes;
            if total_grapes == 0 {
                return Err("No grapes to make wine".to_string());
            }
        }
        ActionSpace::Harvest => {
            let planted_vines = vineyard.fields.iter().filter(|f| f.is_some()).count();
            if planted_vines == 0 {
                return Err("No vines planted".to_string());
            }
        }
        ActionSpace::SellGrapes => {
            let total_grapes = vineyard.red_grapes + vineyard.white_grapes;
            if total_grapes == 0 {
                return Err("No grapes to sell".to_string());
            }
        }
        ActionSpace::BuildStructure => {
            if vineyard.lira < 2 {
                return Err("Need at least 2 lira".to_string());
            }
        }
        _ => {} // Other actions have no prerequisites
    }
    
    Ok(())
}