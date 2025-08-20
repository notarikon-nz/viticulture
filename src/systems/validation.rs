use bevy::prelude::*;
use crate::components::*;
use rand::prelude::*;

#[derive(Resource)]
pub struct GameValidation {
    pub enforce_rules: bool,
    pub prevent_illegal_moves: bool,
}

impl Default for GameValidation {
    fn default() -> Self {
        Self {
            enforce_rules: true,
            prevent_illegal_moves: true,
        }
    }
}

pub fn validate_worker_placement(
    player_id: PlayerId,
    action: ActionSpace,
    workers: &Query<&Worker>,
    action_spaces: &Query<&ActionSpaceSlot>,
    current_state: &GameState,
    validation: &Res<GameValidation>,
) -> ValidationResult {
    if !validation.enforce_rules {
        return ValidationResult::Valid;
    }
    
    // Check if player has available workers
    let has_available_worker = workers.iter()
        .any(|w| w.owner == player_id && w.placed_at.is_none());
    
    if !has_available_worker {
        return ValidationResult::Invalid("No available workers".to_string());
    }
    
    // Check season restrictions
    let is_summer_action = matches!(action, 
        ActionSpace::DrawVine | ActionSpace::PlantVine | ActionSpace::BuildStructure | 
        ActionSpace::GiveTour | ActionSpace::SellGrapes | ActionSpace::TrainWorker);
    
    let valid_season = match current_state {
        GameState::Summer => is_summer_action,
        GameState::Winter => !is_summer_action,
        _ => false,
    };
    
    if !valid_season {
        return ValidationResult::Invalid("Wrong season for this action".to_string());
    }
    
    // Check if action space is available
    if let Some(space) = action_spaces.iter().find(|s| s.action == action) {
        if space.occupied_by.is_some() {
            // Check if player has grande worker available
            let has_grande = workers.iter()
                .any(|w| w.owner == player_id && w.placed_at.is_none() && w.is_grande);
            
            if !has_grande {
                return ValidationResult::Invalid("Action space occupied and no grande worker available".to_string());
            }
        }
    }
    
    ValidationResult::Valid
}

pub fn validate_action_requirements(
    player_id: PlayerId,
    action: ActionSpace,
    players: &Query<&Player>,
    hands: &Query<&Hand>,
    vineyards: &Query<&Vineyard>,
    validation: &Res<GameValidation>,
) -> ValidationResult {
    if !validation.prevent_illegal_moves {
        return ValidationResult::Valid;
    }
    
    let player = players.iter().find(|p| p.id == player_id).unwrap();
    let hand = hands.iter().find(|h| h.owner == player_id).unwrap();
    let vineyard = vineyards.iter().find(|v| v.owner == player_id).unwrap();
    
    match action {
        ActionSpace::PlantVine => {
            if hand.vine_cards.is_empty() {
                return ValidationResult::Invalid("No vine cards to plant".to_string());
            }
            if vineyard.lira == 0 {
                return ValidationResult::Invalid("Not enough lira to plant vine".to_string());
            }
            let empty_fields = vineyard.fields.iter().filter(|f| f.is_none()).count();
            if empty_fields == 0 {
                return ValidationResult::Invalid("No empty fields to plant vine".to_string());
            }
        }
        ActionSpace::Harvest => {
            let planted_vines = vineyard.fields.iter().filter(|f| f.is_some()).count();
            if planted_vines == 0 {
                return ValidationResult::Invalid("No vines planted to harvest".to_string());
            }
        }
        ActionSpace::MakeWine => {
            let total_grapes = vineyard.red_grapes + vineyard.white_grapes;
            if total_grapes == 0 {
                return ValidationResult::Invalid("No grapes available to make wine".to_string());
            }
        }
        ActionSpace::FillOrder => {
            if hand.wine_order_cards.is_empty() {
                return ValidationResult::Invalid("No wine orders to fulfill".to_string());
            }
            let can_fulfill = hand.wine_order_cards.iter()
                .any(|order| vineyard.can_fulfill_order(order));
            if !can_fulfill {
                return ValidationResult::Invalid("Cannot fulfill any wine orders with current wine".to_string());
            }
        }
        ActionSpace::TrainWorker => {
            if vineyard.lira < 4 {
                return ValidationResult::Invalid("Need 4 lira to train a worker".to_string());
            }
        }
        ActionSpace::BuildStructure => {
            if vineyard.lira < 2 {
                return ValidationResult::Invalid("Not enough lira to build structure".to_string());
            }
        }
        ActionSpace::SellGrapes => {
            let total_grapes = vineyard.red_grapes + vineyard.white_grapes;
            if total_grapes == 0 {
                return ValidationResult::Invalid("No grapes to sell".to_string());
            }
        }
        _ => {} // Other actions don't have requirements
    }
    
    ValidationResult::Valid
}

pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }
    
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ValidationResult::Valid => None,
            ValidationResult::Invalid(msg) => Some(msg),
        }
    }
}

pub fn apply_end_game_scoring(
    mut players: Query<&mut Player>,
    vineyards: Query<&Vineyard>,
    structures: Query<&Structure>,
) {
    for mut player in players.iter_mut() {
        let vineyard = vineyards.iter().find(|v| v.owner == player.id).unwrap();
        let player_structures: Vec<_> = structures.iter()
            .filter(|s| s.owner == player.id)
            .collect();
        
        // Windmill bonus: +1 VP for every 7 lira
        if player_structures.iter().any(|s| matches!(s.structure_type, StructureType::Windmill)) {
            let windmill_bonus = vineyard.lira / 7;
            player.gain_victory_points(windmill_bonus);
        }
        
        // Cottage gives extra workers, but no VP bonus
        // Other structures provide ongoing benefits, no end-game VP
    }
}

pub fn check_tie_breaker(
    players: &Query<&Player>,
    vineyards: &Query<&Vineyard>,
) -> Option<PlayerId> {
    let mut player_scores: Vec<_> = players.iter()
        .map(|p| {
            let vineyard = vineyards.iter().find(|v| v.owner == p.id).unwrap();
            let total_wine = vineyard.red_wine + vineyard.white_wine;
            let total_grapes = vineyard.red_grapes + vineyard.white_grapes;
            
            (p.id, p.victory_points, vineyard.lira, total_wine, total_grapes)
        })
        .collect();
    
    // Sort by: VP (desc), then lira (desc), then wine (desc), then grapes (desc)
    player_scores.sort_by(|a, b| {
        b.1.cmp(&a.1) // VP
            .then(b.2.cmp(&a.2)) // Lira
            .then(b.3.cmp(&a.3)) // Wine
            .then(b.4.cmp(&a.4)) // Grapes
    });
    
    if player_scores.len() > 1 && player_scores[0].1 == player_scores[1].1 {
        // There's a tie in VP, tie-breaker applied
        Some(player_scores[0].0)
    } else {
        // Clear winner
        player_scores.get(0).map(|p| p.0)
    }
}

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
        // Add more red cards
        for i in 200..205 {
            card_decks.vine_deck.push(VineCard {
                id: i,
                vine_type: VineType::Red(2),
                cost: 1,
            });
        }
    }
    
    // Shuffle decks for randomness
    use rand::seq::SliceRandom;
    let mut rng = rand::rng();
    card_decks.vine_deck.shuffle(&mut rng);
    card_decks.wine_order_deck.shuffle(&mut rng);
}