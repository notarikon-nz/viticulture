use bevy::prelude::*;
use crate::components::*;
use crate::systems::*;
use crate::systems::audio::*;
use crate::systems::game_logic::execute_action;
use rand::prelude::*;

#[derive(Component)]
pub struct AIPlayer {
    pub player_id: PlayerId,
    pub difficulty: AIDifficulty,
    pub decision_timer: Timer,
}

#[derive(Clone, Copy, Debug)]
pub enum AIDifficulty {
    Beginner,
    Intermediate,
}

impl AIPlayer {
    pub fn new(player_id: PlayerId, difficulty: AIDifficulty) -> Self {
        Self {
            player_id,
            difficulty,
            decision_timer: Timer::from_seconds(1.5, TimerMode::Once),
        }
    }
}

#[derive(Resource, Default)]
pub struct AISettings {
    pub player_count: u8,
    pub ai_count: u8,
    pub ai_difficulty: AIDifficulty,
}

impl Default for AIDifficulty {
    fn default() -> Self {
        AIDifficulty::Beginner
    }
}

pub fn ai_decision_system(
    time: Res<Time>,
    mut ai_players: Query<&mut AIPlayer>,
    mut workers: Query<&mut Worker>,
    mut action_spaces: Query<&mut ActionSpaceSlot>,
    mut hands: Query<&mut Hand>,
    mut vineyards: Query<&mut Vineyard>,
    mut players: Query<&mut Player>,
    mut card_decks: ResMut<CardDecks>,
    mut commands: Commands,
    turn_order: Res<TurnOrder>,
    current_state: Res<State<GameState>>,
    audio_assets: Res<AudioAssets>,
    audio_settings: Res<AudioSettings>,
    animation_settings: Res<AnimationSettings>,
    // mut trackers: Query<&mut ResidualPaymentTracker>,
    (mut trackers, structures) : (Query<&mut ResidualPaymentTracker>, Query<&Structure>),
    // structures: Query<&Structure>, 
) {
    if !matches!(current_state.get(), GameState::Summer | GameState::Winter) {
        return;
    }
    
    if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
        let ai_player = ai_players.iter_mut().find(|ai| ai.player_id == *current_player_id);
        
        if let Some(mut ai_player) = ai_players.iter_mut().find(|ai| ai.player_id == *current_player_id) {
            ai_player.decision_timer.tick(time.delta());
            
            if ai_player.decision_timer.finished() {
                ai_player.decision_timer.reset();
                
                let action = choose_ai_action(
                    *current_player_id,
                    ai_player.difficulty,
                    &workers,
                    &action_spaces,
                    &hands,
                    &vineyards,
                    &players,
                    current_state.get(),
                );
                
                if let Some(chosen_action) = action {
                    execute_ai_action(
                        chosen_action,
                        *current_player_id,
                        &mut workers,
                        &mut action_spaces,
                        &mut hands,
                        &mut vineyards,
                        &mut players,
                        &mut card_decks,
                        &mut commands,
                        &audio_assets,
                        &audio_settings,
                        &animation_settings,
                        &mut trackers,
                        &structures,
                    );
                }
            }
        }
    }
}

fn choose_ai_action(
    player_id: PlayerId,
    difficulty: AIDifficulty,
    workers: &Query<&mut Worker>,
    action_spaces: &Query<&mut ActionSpaceSlot>,
    hands: &Query<&mut Hand>,
    vineyards: &Query<&mut Vineyard>,
    players: &Query<&mut Player>,
    current_state: &GameState,
) -> Option<ActionSpace> {
    let available_workers = workers.iter()
        .filter(|w| w.owner == player_id && w.placed_at.is_none())
        .count();
    
    if available_workers == 0 {
        return None;
    }
    
    let player = players.iter().find(|p| p.id == player_id)?;
    let hand = hands.iter().find(|h| h.owner == player_id)?;
    let vineyard = vineyards.iter().find(|v| v.owner == player_id)?;
    
    let mut valid_actions = Vec::new();
    
    for space in action_spaces.iter() {
        if space.can_place_worker(player_id, current_state) ||
           space.can_place_grande_worker(player_id, current_state) {
            valid_actions.push(space.action);
        }
    }
    
    if valid_actions.is_empty() {
        return None;
    }
    
    match difficulty {
        AIDifficulty::Beginner => choose_random_action(&valid_actions),
        AIDifficulty::Intermediate => choose_smart_action(&valid_actions, player, hand, vineyard, current_state),
    }
}

fn choose_random_action(valid_actions: &[ActionSpace]) -> Option<ActionSpace> {
    let mut rng = rand::rng();
    valid_actions.choose(&mut rng).copied()
}

fn choose_smart_action(
    valid_actions: &[ActionSpace],
    player: &Player,
    hand: &Hand,
    vineyard: &Vineyard,
    current_state: &GameState,
) -> Option<ActionSpace> {
    let mut scored_actions = Vec::new();
    
    for &action in valid_actions {
        let score = evaluate_action(action, player, hand, vineyard, current_state);
        scored_actions.push((action, score));
    }
    
    scored_actions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Add some randomness to prevent predictable play
    let mut rng = rand::rng();
    let top_actions: Vec<_> = scored_actions.iter()
        .take(3)
        .map(|(action, _)| *action)
        .collect();
    
    top_actions.choose(&mut rng).copied()
}

fn evaluate_action(
    action: ActionSpace,
    player: &Player,
    hand: &Hand,
    vineyard: &Vineyard,
    current_state: &GameState,
) -> f32 {
    match action {
        ActionSpace::DrawVine => {
            if hand.vine_cards.len() < 3 { 0.7 } else { 0.2 }
        }
        ActionSpace::DrawWineOrder => {
            if hand.wine_order_cards.len() < 2 { 0.8 } else { 0.1 }
        }
        ActionSpace::PlantVine => {
            if !hand.vine_cards.is_empty() && vineyard.lira >= 1 { 1.0 } else { 0.0 }
        }
        ActionSpace::Harvest => {
            // FIXED: Check if any fields have vines planted
            let planted_vines = vineyard.fields.iter().filter(|f| f.vine.is_some()).count();
            if planted_vines > 0 { 0.9 } else { 0.0 }
        }
        ActionSpace::MakeWine => {
            let total_grapes = vineyard.red_grapes + vineyard.white_grapes;
            if total_grapes > 0 { 0.8 } else { 0.0 }
        }
        ActionSpace::FillOrder => {
            let can_fill = hand.wine_order_cards.iter()
                .any(|order| vineyard.can_fulfill_order(order));
            if can_fill { 1.2 } else { 0.0 }
        }
        ActionSpace::GiveTour => {
            if vineyard.lira < 5 { 0.6 } else { 0.3 }
        }
        ActionSpace::TrainWorker => {
            if player.workers < 4 && vineyard.lira >= 4 { 0.5 } else { 0.0 }
        }
        ActionSpace::BuildStructure => {
            if vineyard.lira >= 2 { 0.4 } else { 0.0 }
        }
        ActionSpace::SellGrapes => {
            let total_grapes = vineyard.red_grapes + vineyard.white_grapes;
            if total_grapes > 3 && vineyard.lira < 3 { 0.7 } else { 0.2 }
        }
    }
}

fn execute_ai_action(
    action: ActionSpace,
    player_id: PlayerId,
    workers: &mut Query<&mut Worker>,
    action_spaces: &mut Query<&mut ActionSpaceSlot>,
    hands: &mut Query<&mut Hand>,
    vineyards: &mut Query<&mut Vineyard>,
    players: &mut Query<&mut Player>,
    card_decks: &mut ResMut<CardDecks>,
    commands: &mut Commands,
    audio_assets: &Res<AudioAssets>,    // Fixed: removed mut
    audio_settings: &Res<AudioSettings>, // Fixed: removed mut
    animation_settings: &Res<AnimationSettings>,
    trackers: &mut Query<&mut ResidualPaymentTracker>,
    structures: &Query<&Structure>, 
) {
    // Find and place a worker
    let mut worker_placed = false;
    
    // Try to place regular worker first
    for mut worker in workers.iter_mut() {
        if worker.owner == player_id && worker.placed_at.is_none() && !worker.is_grande {
            // Find the action space
            for mut space in action_spaces.iter_mut() {
                if space.action == action && space.occupied_by.is_none() {
                    worker.placed_at = Some(action);
                    worker.position = space.position;
                    space.occupied_by = Some(player_id);
                    worker_placed = true;
                    break;
                }
            }
            if worker_placed { break; }
        }
    }
    
    // If no regular worker could be placed, try grande worker
    if !worker_placed {
        for mut worker in workers.iter_mut() {
            if worker.owner == player_id && worker.placed_at.is_none() && worker.is_grande {
                for mut space in action_spaces.iter_mut() {
                    if space.action == action {
                        worker.placed_at = Some(action);
                        worker.position = space.position;
                        if space.occupied_by.is_some() {
                            space.bonus_worker_slot = Some(player_id);
                        } else {
                            space.occupied_by = Some(player_id);
                        }
                        worker_placed = true;
                        break;
                    }
                }
                if worker_placed { break; }
            }
        }
    }
    
    if worker_placed {
        execute_action(action, player_id, hands, vineyards, players, card_decks, commands, trackers, structures, audio_assets, audio_settings, animation_settings);
        info!("AI Player {:?} executed action {:?}", player_id, action);
    }
}

pub fn setup_ai_players(
    mut commands: Commands,
    ai_settings: Res<AISettings>,
    players: Query<&Player>,
) {
    for player in players.iter() {
        if player.id.0 >= ai_settings.player_count - ai_settings.ai_count {
            commands.spawn(AIPlayer::new(player.id, ai_settings.ai_difficulty));
        }
    }
}