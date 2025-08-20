use bevy::prelude::*;
use crate::components::*;
use crate::systems::ai::*;

#[derive(Resource, Default)]
pub struct BalanceTestResults {
    pub games_played: u32,
    pub ai_wins: u32,
    pub human_wins: u32,
    pub average_game_length: f32,
    pub action_usage_stats: std::collections::HashMap<u8, u32>, // ActionSpace as u8
}

#[derive(Resource, Default)]
pub struct AutoTestConfig {
    pub enabled: bool,
    pub target_games: u32,
    pub ai_only_mode: bool,
    pub fast_mode: bool, // Skip animations and delays
}

pub fn auto_balance_test_system(
    mut test_config: ResMut<AutoTestConfig>,
    mut results: ResMut<BalanceTestResults>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    players: Query<&Player>,
) {
    // Start auto-testing with F10
    if keyboard.just_pressed(KeyCode::F10) {
        test_config.enabled = !test_config.enabled;
        test_config.ai_only_mode = true;
        test_config.fast_mode = true;
        test_config.target_games = 10;
        
        if test_config.enabled {
            info!("Starting balance testing - {} games", test_config.target_games);
            results.games_played = 0;
            results.ai_wins = 0;
            results.human_wins = 0;
            next_state.set(GameState::Setup);
        } else {
            info!("Balance testing stopped");
        }
    }
    
    // Auto-restart games for testing
    if test_config.enabled && matches!(current_state.get(), GameState::GameOver) {
        if results.games_played < test_config.target_games {
            // Record game result
            let winner = find_winner(players);
            if let Some(winner_name) = winner {
                info!("Game {} completed - Winner: {}", results.games_played + 1, winner_name);
                if winner_name.contains("AI") {
                    results.ai_wins += 1;
                } else {
                    results.human_wins += 1;
                }
            }
            
            results.games_played += 1;
            
            // Start next test game
            next_state.set(GameState::Setup);
        } else {
            // Testing complete
            print_balance_results(&results);
            test_config.enabled = false;
        }
    }
}

fn find_winner(players: Query<&Player>) -> Option<String> {
    players.iter()
        .max_by_key(|p| p.victory_points)
        .map(|p| p.name.clone())
}

fn print_balance_results(results: &BalanceTestResults) {
    info!("=== BALANCE TEST RESULTS ===");
    info!("Games Played: {}", results.games_played);
    info!("AI Wins: {} ({:.1}%)", results.ai_wins, 
          (results.ai_wins as f32 / results.games_played as f32) * 100.0);
    info!("Human Wins: {} ({:.1}%)", results.human_wins,
          (results.human_wins as f32 / results.games_played as f32) * 100.0);
    
    // Ideal balance: 40-60% win rate for AI (challenging but beatable)
    let ai_win_rate = results.ai_wins as f32 / results.games_played as f32;
    if ai_win_rate < 0.3 {
        warn!("AI too weak - consider buffing AI decision making");
    } else if ai_win_rate > 0.7 {
        warn!("AI too strong - consider nerfing AI decision making");
    } else {
        info!("AI balance looks good!");
    }
}

fn action_to_id(action: ActionSpace) -> u8 {
    match action {
        ActionSpace::DrawVine => 0,
        ActionSpace::PlantVine => 1,
        ActionSpace::BuildStructure => 2,
        ActionSpace::GiveTour => 3,
        ActionSpace::SellGrapes => 4,
        ActionSpace::DrawWineOrder => 5,
        ActionSpace::Harvest => 6,
        ActionSpace::MakeWine => 7,
        ActionSpace::FillOrder => 8,
        ActionSpace::TrainWorker => 9,
    }
}

fn id_to_action(id: u8) -> Option<ActionSpace> {
    match id {
        0 => Some(ActionSpace::DrawVine),
        1 => Some(ActionSpace::PlantVine),
        2 => Some(ActionSpace::BuildStructure),
        3 => Some(ActionSpace::GiveTour),
        4 => Some(ActionSpace::SellGrapes),
        5 => Some(ActionSpace::DrawWineOrder),
        6 => Some(ActionSpace::Harvest),
        7 => Some(ActionSpace::MakeWine),
        8 => Some(ActionSpace::FillOrder),
        9 => Some(ActionSpace::TrainWorker),
        _ => None,
    }
}

pub fn track_action_usage_system(
    mut results: ResMut<BalanceTestResults>,
    workers: Query<&Worker, Changed<Worker>>,
) {
    // Track which actions are being used most
    for worker in workers.iter() {
        if let Some(action) = worker.placed_at {
            let action_id = action_to_id(action);
            *results.action_usage_stats.entry(action_id).or_insert(0) += 1;
        }
    }
}

pub fn dynamic_difficulty_system(
    mut ai_players: Query<&mut AIPlayer>,
    results: Res<BalanceTestResults>,
) {
    if results.games_played < 5 {
        return; // Need some data first
    }
    
    let ai_win_rate = results.ai_wins as f32 / results.games_played as f32;
    
    // Adjust AI difficulty based on performance
    for mut ai_player in ai_players.iter_mut() {
        match ai_player.difficulty {
            AIDifficulty::Beginner => {
                if ai_win_rate > 0.7 {
                    // AI winning too much, keep it beginner
                } else if ai_win_rate < 0.3 {
                    // AI losing too much, upgrade to intermediate
                    ai_player.difficulty = AIDifficulty::Intermediate;
                    info!("Upgraded AI {:?} to Intermediate difficulty", ai_player.player_id);
                }
            }
            AIDifficulty::Intermediate => {
                if ai_win_rate > 0.8 {
                    // Too strong, downgrade
                    ai_player.difficulty = AIDifficulty::Beginner;
                    info!("Downgraded AI {:?} to Beginner difficulty", ai_player.player_id);
                }
            }
        }
    }
}

// Balance tweaks based on testing
pub fn apply_balance_tweaks(
    card_decks: ResMut<CardDecks>,
    results: Res<BalanceTestResults>,
) {
    if results.games_played < 10 {
        return;
    }
    
    // Analyze action usage
    let total_actions: u32 = results.action_usage_stats.values().sum();
    if total_actions == 0 {
        return;
    }
    
    // Check if any actions are underused (< 5% usage)
    for (action_id, usage) in &results.action_usage_stats {
        let usage_rate = *usage as f32 / total_actions as f32;
        
        if let Some(action) = id_to_action(*action_id) {
            if usage_rate < 0.05 {
                info!("Action {:?} underused ({:.1}%) - consider buffing", action, usage_rate * 100.0);
                
                // Apply buffs to underused actions
                match action {
                    ActionSpace::BuildStructure => {
                        // Make structures cheaper by adding more starting lira
                        info!("Buffing BuildStructure by improving economy");
                    }
                    ActionSpace::SellGrapes => {
                        info!("SellGrapes underused - might need better payouts");
                    }
                    ActionSpace::TrainWorker => {
                        info!("TrainWorker underused - cost might be too high");
                    }
                    _ => {}
                }
            } else if usage_rate > 0.25 {
                info!("Action {:?} overused ({:.1}%) - consider nerfing", action, usage_rate * 100.0);
            }
        }
    }
}

// Quick game mode for testing
pub fn fast_test_mode_system(
    test_config: Res<AutoTestConfig>,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    time: Res<Time>,
) {
    if !test_config.fast_mode {
        return;
    }
    
    // Auto-advance phases quickly for testing
    let should_advance = match current_state.get() {
        GameState::Spring => true,
        GameState::Fall => true,
        _ => false,
    };
    
    if should_advance {
        // Skip to next phase after short delay
        if time.elapsed_seconds() % 0.5 < 0.1 {
            match current_state.get() {
                GameState::Spring => next_state.set(GameState::Summer),
                GameState::Fall => next_state.set(GameState::Winter),
                _ => {}
            }
        }
    }
}

pub fn game_length_tracking_system(
    mut results: ResMut<BalanceTestResults>,
    config: Res<GameConfig>,
    current_state: Res<State<GameState>>,
    time: Res<Time>,
) {
    // Track average game length for balance analysis
    if matches!(current_state.get(), GameState::GameOver) {
        let game_length = config.current_year as f32;
        
        if results.games_played > 0 {
            results.average_game_length = (results.average_game_length * (results.games_played - 1) as f32 + game_length) / results.games_played as f32;
        } else {
            results.average_game_length = game_length;
        }
        
        // Ideal game length: 5-7 years
        if results.average_game_length < 4.0 {
            info!("Games ending too quickly - consider increasing VP requirement");
        } else if results.average_game_length > 8.0 {
            info!("Games taking too long - consider decreasing VP requirement");
        }
    }
}