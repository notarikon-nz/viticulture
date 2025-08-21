// Fixed balance.rs - prevents UI text loss during automatic testing

use bevy::prelude::*;
use crate::components::*;
use crate::systems::*;
use crate::systems::ai::*;

#[derive(Resource, Default)]
pub struct BalanceTestResults {
    pub games_played: u32,
    pub ai_wins: u32,
    pub human_wins: u32,
    pub average_game_length: f32,
    pub action_usage_stats: std::collections::HashMap<u8, u32>,
}

#[derive(Resource, Default)]
pub struct AutoTestConfig {
    pub enabled: bool,
    pub target_games: u32,
    pub ai_only_mode: bool,
    pub fast_mode: bool,
    pub restart_timer: Timer, // Add timer to prevent immediate state changes
    pub ui_protected: bool,   // Flag to protect UI during testing
}

impl AutoTestConfig {
    pub fn new() -> Self {
        Self {
            enabled: false,
            target_games: 10,
            ai_only_mode: true,
            fast_mode: true,
            restart_timer: Timer::from_seconds(1.0, TimerMode::Once), // 1 second delay
            ui_protected: false,
        }
    }
}

pub fn auto_balance_test_system(
    mut test_config: ResMut<AutoTestConfig>,
    mut results: ResMut<BalanceTestResults>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    players: Query<&Player>,
    time: Res<Time>,
    mut commands: Commands,
    existing_ui: Query<Entity, With<UIPanel>>,
) {
    // Start auto-testing with F10
    if keyboard.just_pressed(KeyCode::F10) {
        test_config.enabled = !test_config.enabled;
        test_config.ai_only_mode = true;
        test_config.fast_mode = true;
        test_config.target_games = 10;
        test_config.ui_protected = true; // Protect UI during testing
        
        if test_config.enabled {
            info!("Starting balance testing - {} games", test_config.target_games);
            results.games_played = 0;
            results.ai_wins = 0;
            results.human_wins = 0;
            
            // Don't immediately restart - let current game finish naturally
            if matches!(current_state.get(), GameState::MainMenu) {
                test_config.restart_timer.reset();
                next_state.set(GameState::Setup);
            }
        } else {
            info!("Balance testing stopped");
            test_config.ui_protected = false;
        }
    }
    
    // Update restart timer
    test_config.restart_timer.tick(time.delta());
    
    // Auto-restart games for testing with delay and UI protection
    if test_config.enabled && matches!(current_state.get(), GameState::GameOver) {
        if results.games_played < test_config.target_games {
            
            // Record game result only once per game
            if test_config.restart_timer.finished() {
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
                
                // Reset timer and restart game with UI protection
                test_config.restart_timer.reset();
                
                // Instead of going to Setup (which recreates UI), go directly to Spring
                // This preserves the existing UI while resetting game state
                restart_game_preserve_ui(&mut commands, &existing_ui, &mut next_state);
            }
        } else {
            // Testing complete
            print_balance_results(&results);
            test_config.enabled = false;
            test_config.ui_protected = false;
        }
    }
}

// New function to restart game without destroying UI
fn restart_game_preserve_ui(
    commands: &mut Commands,
    existing_ui: &Query<Entity, With<UIPanel>>,
    next_state: &mut ResMut<NextState<GameState>>,
) {
    // Don't despawn UI - just reset game state
    // The setup_game_system will handle resetting player data
    
    info!("Restarting game while preserving UI...");
    
    // Check if UI still exists
    if existing_ui.is_empty() {
        // UI was lost, need to recreate it
        warn!("UI was lost during testing, will recreate");
        next_state.set(GameState::Setup);
    } else {
        // UI exists, can safely restart game logic
        next_state.set(GameState::Spring);
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
    
    let ai_win_rate = results.ai_wins as f32 / results.games_played as f32;
    if ai_win_rate < 0.3 {
        warn!("AI too weak - consider buffing AI decision making");
    } else if ai_win_rate > 0.7 {
        warn!("AI too strong - consider nerfing AI decision making");
    } else {
        info!("AI balance looks good!");
    }
}

// Protected setup system that doesn't recreate UI during testing
pub fn protected_setup_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut config: ResMut<GameConfig>,
    test_config: Res<AutoTestConfig>,
    existing_ui: Query<Entity, With<UIPanel>>,
    existing_players: Query<Entity, With<Player>>,
    mut turn_order: ResMut<TurnOrder>,
    mut card_decks: ResMut<CardDecks>,
    text_query: Query<Entity, With<Text>>,
    existing_entities: Query<Entity, (Without<UIPanel>, Without<PhaseText>)>,
    current_state: Res<State<GameState>>,

) {
    // ONLY run this system during balance testing and in Setup state
    if !test_config.enabled || !matches!(current_state.get(), GameState::Setup) {
        return; // Let the normal setup system handle it
    }

    // Only run in Setup state during testing
    if !test_config.enabled {
        // Normal setup - call the regular setup system
        // Note: We need to call this differently since we have ResMut vs Res mismatch
        setup_normal_game(
            &mut commands, 
            &mut next_state, 
            &mut config, 
            &mut turn_order, 
            &mut card_decks,
            &text_query,
            &existing_entities
        );
        return;
    }
    
    // Testing mode setup - preserve UI, reset game data only
    info!("Protected setup for balance testing");
    
    // Reset game config
    config.current_year = 1;
    
    // Clean up old player entities (but not UI)
    for entity in existing_players.iter() {
        commands.entity(entity).despawn();
    }
    
    // Clean up other game entities but preserve UI
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn();
    }
    
    // Reset turn order
    turn_order.players.clear();
    turn_order.current_player = 0;
    turn_order.wake_up_order.clear();
    
    // Create new players for testing
    setup_test_players(&mut commands, &config, &mut turn_order);
    
    // If UI doesn't exist, create it
    if existing_ui.is_empty() {
        crate::systems::ui::setup_ui(&mut commands);
    }
    
    next_state.set(GameState::Spring);
}

// Helper function that mimics the regular setup without type conflicts
fn setup_normal_game(
    commands: &mut Commands,
    next_state: &mut ResMut<NextState<GameState>>,
    config: &mut ResMut<GameConfig>,
    turn_order: &mut ResMut<TurnOrder>,
    card_decks: &mut ResMut<CardDecks>,
    text_query: &Query<Entity, With<Text>>,
    existing_entities: &Query<Entity, (Without<UIPanel>, Without<PhaseText>)>,
) {
    // Clean up existing entities
    for entity in text_query.iter() {
        commands.entity(entity).despawn();
    }
    
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn();
    }
    
    // Reset game state
    config.current_year = 1;
    turn_order.players.clear();
    turn_order.current_player = 0;
    turn_order.wake_up_order.clear();
    
    // Create players
    for i in 0..config.player_count {
        let is_ai = i >= (config.player_count - config.ai_count);
        let name = if is_ai {
            format!("AI Player {}", i + 1)
        } else {
            format!("Player {}", i + 1)
        };
        
        let player = Player::new(i, name, is_ai);
        turn_order.players.push(player.id);
        commands.spawn(player);
        
        // Create vineyard for each player
        commands.spawn(Vineyard::new(PlayerId(i)));
        
        // Create hand for each player
        commands.spawn(Hand::new(PlayerId(i)));
        
        // Create residual payment tracker
        commands.spawn(ResidualPaymentTracker::new(PlayerId(i)));
    }
    
    // Create workers for each player
    for i in 0..config.player_count {
        let player_id = PlayerId(i);
        
        // Regular workers
        for j in 0..2 {
            let worker_pos = Vec2::new(-500.0 + (i as f32 * 120.0) + (j as f32 * 30.0), -200.0);
            commands.spawn((
                Worker::new(player_id, false, worker_pos),
                Clickable { size: Vec2::new(20.0, 20.0) },
            ));
        }
        
        // Grande worker
        let grande_pos = Vec2::new(-500.0 + (i as f32 * 120.0), -170.0);
        commands.spawn((
            Worker::new(player_id, true, grande_pos),
            Clickable { size: Vec2::new(25.0, 25.0) },
        ));
    }
    
    // Create action board
    commands.spawn(ActionBoard::new());
    
    next_state.set(GameState::Spring);
}

fn setup_test_players(commands: &mut Commands, config: &GameConfig, turn_order: &mut ResMut<TurnOrder>) {
    // Create players for testing
    for i in 0..config.player_count {
        let is_ai = i >= (config.player_count - config.ai_count);
        let name = if is_ai {
            format!("AI Player {}", i + 1)
        } else {
            format!("Human Player {}", i + 1)
        };
        
        let player = Player::new(i, name, is_ai);
        turn_order.players.push(player.id);
        commands.spawn(player);
        
        // Create vineyard for each player
        commands.spawn(Vineyard::new(PlayerId(i)));
        
        // Create hand for each player
        commands.spawn(Hand::new(PlayerId(i)));
        
        // Create residual payment tracker
        commands.spawn(ResidualPaymentTracker::new(PlayerId(i)));
    }
    
    // Create workers for each player
    for i in 0..config.player_count {
        let player_id = PlayerId(i);
        
        // Regular workers
        for j in 0..2 {
            let worker_pos = Vec2::new(-500.0 + (i as f32 * 120.0) + (j as f32 * 30.0), -200.0);
            commands.spawn((
                Worker::new(player_id, false, worker_pos),
                Clickable { size: Vec2::new(20.0, 20.0) },
            ));
        }
        
        // Grande worker
        let grande_pos = Vec2::new(-500.0 + (i as f32 * 120.0), -170.0);
        commands.spawn((
            Worker::new(player_id, true, grande_pos),
            Clickable { size: Vec2::new(25.0, 25.0) },
        ));
    }
}
// Enhanced fast test mode that doesn't break UI and ensures game progression
pub fn fast_test_mode_system(
    test_config: Res<AutoTestConfig>,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut timer: Local<Timer>,
    time: Res<Time>,
    mut turn_order: ResMut<TurnOrder>,
    workers: Query<&Worker>,
    players: Query<&Player>,
    ai_players: Query<&AIPlayer>,
    mut config: ResMut<GameConfig>,
) {
    if !test_config.fast_mode || !test_config.enabled {
        return;
    }
    
    // Initialize timer on first run
    if timer.duration() == std::time::Duration::ZERO {
        *timer = Timer::from_seconds(0.5, TimerMode::Repeating);
    }
    
    timer.tick(time.delta());
    
    // Auto-advance phases quickly for testing, but only on timer
    if timer.just_finished() {
        match current_state.get() {
            GameState::Spring => {
                info!("Fast test: advancing from Spring to Summer");
                next_state.set(GameState::Summer);
            }
            GameState::Fall => {
                info!("Fast test: advancing from Fall to Winter");
                next_state.set(GameState::Winter);
            }
            GameState::Summer | GameState::Winter => {
                // Check if we should advance to next phase or next year
                if should_advance_season(&workers, &players, &ai_players, &turn_order) {
                    match current_state.get() {
                        GameState::Summer => {
                            info!("Fast test: advancing from Summer to Fall");
                            next_state.set(GameState::Fall);
                        }
                        GameState::Winter => {
                            info!("Fast test: advancing from Winter to next year");
                            advance_to_next_year(&mut config, &mut next_state);
                        }
                        _ => {}
                    }
                }
            }
            GameState::GameOver => {
                // Let the balance test system handle game over
            }
            _ => {}
        }
    }
    
    // Allow manual override with space key during testing
    if keyboard.just_pressed(KeyCode::Space) {
        match current_state.get() {
            GameState::Spring => next_state.set(GameState::Summer),
            GameState::Fall => next_state.set(GameState::Winter),
            GameState::Summer => next_state.set(GameState::Fall),
            GameState::Winter => advance_to_next_year(&mut config, &mut next_state),
            _ => {}
        }
    }
}

// Check if all players have finished their actions for the season
fn should_advance_season(
    workers: &Query<&Worker>,
    players: &Query<&Player>,
    ai_players: &Query<&AIPlayer>,
    turn_order: &TurnOrder,
) -> bool {
    // Count available workers for each player
    for player in players.iter() {
        let available_workers = workers.iter()
            .filter(|w| w.owner == player.id && w.placed_at.is_none())
            .count();
        
        // If any player has available workers, the season isn't over
        if available_workers > 0 {
            // For AI players, check if they're still thinking
            if let Some(_ai_player) = ai_players.iter().find(|ai| ai.player_id == player.id) {
                // AI has workers but might be deciding - give them a moment
                return false;
            } else {
                // Human player has workers - should wait for them
                // But in fast test mode, assume they're done
                return true;
            }
        }
    }
    
    // No players have available workers - season is done
    true
}

fn advance_to_next_year(config: &mut ResMut<GameConfig>, next_state: &mut ResMut<NextState<GameState>>) {
    config.current_year += 1;
    
    // Check for game end conditions
    if config.current_year > config.max_years {
        info!("Fast test: Year limit reached, ending game");
        next_state.set(GameState::GameOver);
    } else {
        info!("Fast test: Starting year {}", config.current_year);
        next_state.set(GameState::Spring);
    }
}

// Enhanced AI system for testing - much more aggressive
pub fn fast_ai_decision_system(
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
    (mut trackers, structures): (Query<&mut ResidualPaymentTracker>, Query<&Structure>),
    test_config: Res<AutoTestConfig>,
) {
    if !matches!(current_state.get(), GameState::Summer | GameState::Winter) {
        return;
    }
    
    // Use faster decision making during testing
    let decision_time = if test_config.enabled && test_config.fast_mode {
        0.1 // Very fast decisions during testing
    } else {
        1.5 // Normal decision time
    };
    
    // Process all AI players
    for mut ai_player in ai_players.iter_mut() {
        // Update timer duration for testing
        if test_config.enabled && test_config.fast_mode {
            ai_player.decision_timer = Timer::from_seconds(decision_time, TimerMode::Once);
        }
        
        ai_player.decision_timer.tick(time.delta());
        
        if ai_player.decision_timer.finished() {
            ai_player.decision_timer.reset();
            
            // Check if this AI has available workers
            let available_workers = workers.iter()
                .filter(|w| w.owner == ai_player.player_id && w.placed_at.is_none())
                .count();
            
            if available_workers > 0 {
                let action = choose_ai_action(
                    ai_player.player_id,
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
                        ai_player.player_id,
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
                    
                    if test_config.enabled {
                        info!("Fast AI: Player {:?} executed {:?}", ai_player.player_id, chosen_action);
                    }
                } else {
                    // AI can't find a valid action - this shouldn't happen
                    if test_config.enabled {
                        warn!("Fast AI: Player {:?} has workers but no valid actions!", ai_player.player_id);
                    }
                }
            }
        }
    }
}

// Force advance system - if game gets completely stuck, force progression
pub fn unstuck_system(
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut stuck_timer: Local<Timer>,
    time: Res<Time>,
    test_config: Res<AutoTestConfig>,
    workers: Query<&Worker>,
    players: Query<&Player>,
    mut config: ResMut<GameConfig>,
) {
    if !test_config.enabled {
        return;
    }
    
    // Initialize stuck timer
    if stuck_timer.duration() == std::time::Duration::ZERO {
        *stuck_timer = Timer::from_seconds(5.0, TimerMode::Once); // 5 second timeout
    }
    
    match current_state.get() {
        GameState::Summer | GameState::Winter => {
            // Check if game is progressing
            let any_workers_available = players.iter().any(|player| {
                workers.iter().any(|w| w.owner == player.id && w.placed_at.is_none())
            });
            
            if any_workers_available {
                stuck_timer.tick(time.delta());
                
                if stuck_timer.just_finished() {
                    warn!("Game stuck in {:?}, forcing advancement", current_state.get());
                    
                    // Force advance
                    match current_state.get() {
                        GameState::Summer => {
                            next_state.set(GameState::Fall);
                        }
                        GameState::Winter => {
                            config.current_year += 1;
                            if config.current_year > config.max_years {
                                next_state.set(GameState::GameOver);
                            } else {
                                next_state.set(GameState::Spring);
                            }
                        }
                        _ => {}
                    }
                    
                    stuck_timer.reset();
                }
            } else {
                // Game is progressing normally
                stuck_timer.reset();
            }
        }
        _ => {
            stuck_timer.reset();
        }
    }
}

// UI protection system - monitors and restores button text during testing
pub fn ui_protection_system(
    test_config: Res<AutoTestConfig>,
    mut commands: Commands,
    buttons_without_text: Query<(Entity, &ActionButton), (With<Button>, Without<Children>)>,
    buttons_with_empty_children: Query<(Entity, &ActionButton, &Children), With<Button>>,
    text_query: Query<&Text>,
) {
    // Only run protection during testing
    if !test_config.ui_protected || !test_config.enabled {
        return;
    }
    
    let mut repairs_made = 0;
    
    // Repair buttons that have no children at all
    for (button_entity, action_button) in buttons_without_text.iter() {
        let button_text = get_action_display_name(action_button.action);
        
        commands.entity(button_entity).with_children(|button| {
            button.spawn(TextBundle {
                text: Text::from_section(
                    button_text,
                    TextStyle {
                        font_size: 16.0,
                        color: get_action_text_color(action_button.action),
                        ..default()
                    },
                ),
                visibility: Visibility::Inherited,
                ..default()
            });
        });
        
        repairs_made += 1;
    }
    
    // Repair buttons that have children but no text children
    for (button_entity, action_button, children) in buttons_with_empty_children.iter() {
        let has_text_child = children.iter().any(|&child| text_query.contains(child));
        
        if !has_text_child {
            let button_text = get_action_display_name(action_button.action);
            
            commands.entity(button_entity).with_children(|button| {
                button.spawn(TextBundle {
                    text: Text::from_section(
                        button_text,
                        TextStyle {
                            font_size: 16.0,
                            color: get_action_text_color(action_button.action),
                            ..default()
                        },
                    ),
                    visibility: Visibility::Inherited,
                    ..default()
                });
            });
            
            repairs_made += 1;
        }
    }
    
    if repairs_made > 0 {
        info!("UI Protection: Repaired {} button texts during testing", repairs_made);
    }
}

// Helper functions for better button text
fn get_action_display_name(action: ActionSpace) -> String {
    match action {
        ActionSpace::DrawVine => "Draw Vine".to_string(),
        ActionSpace::PlantVine => "Plant Vine (+1)".to_string(),
        ActionSpace::BuildStructure => "Build Structure".to_string(),
        ActionSpace::GiveTour => "Give Tour (+1)".to_string(),
        ActionSpace::SellGrapes => "Sell Grapes".to_string(),
        ActionSpace::DrawWineOrder => "Draw Wine Order".to_string(),
        ActionSpace::Harvest => "Harvest (+1)".to_string(),
        ActionSpace::MakeWine => "Make Wine (+1)".to_string(),
        ActionSpace::FillOrder => "Fill Order".to_string(),
        ActionSpace::TrainWorker => "Train Worker".to_string(),
    }
}

fn get_action_text_color(action: ActionSpace) -> Color {
    match action {
        ActionSpace::DrawVine | ActionSpace::PlantVine | ActionSpace::BuildStructure | 
        ActionSpace::GiveTour | ActionSpace::SellGrapes | ActionSpace::TrainWorker => Color::BLACK,
        ActionSpace::DrawWineOrder | ActionSpace::Harvest | ActionSpace::MakeWine | 
        ActionSpace::FillOrder => Color::WHITE,
    }
}

// Keep the rest of the existing functions unchanged
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
        return;
    }
    
    let ai_win_rate = results.ai_wins as f32 / results.games_played as f32;
    
    for mut ai_player in ai_players.iter_mut() {
        match ai_player.difficulty {
            AIDifficulty::Beginner => {
                if ai_win_rate < 0.3 {
                    ai_player.difficulty = AIDifficulty::Intermediate;
                    info!("Upgraded AI {:?} to Intermediate difficulty", ai_player.player_id);
                }
            }
            AIDifficulty::Intermediate => {
                if ai_win_rate > 0.8 {
                    ai_player.difficulty = AIDifficulty::Beginner;
                    info!("Downgraded AI {:?} to Beginner difficulty", ai_player.player_id);
                }
            }
        }
    }
}

pub fn apply_balance_tweaks(
    card_decks: ResMut<CardDecks>,
    results: Res<BalanceTestResults>,
) {
    if results.games_played < 10 {
        return;
    }
    
    let total_actions: u32 = results.action_usage_stats.values().sum();
    if total_actions == 0 {
        return;
    }
    
    for (action_id, usage) in &results.action_usage_stats {
        let usage_rate = *usage as f32 / total_actions as f32;
        
        if let Some(action) = id_to_action(*action_id) {
            if usage_rate < 0.05 {
                info!("Action {:?} underused ({:.1}%) - consider buffing", action, usage_rate * 100.0);
            } else if usage_rate > 0.25 {
                info!("Action {:?} overused ({:.1}%) - consider nerfing", action, usage_rate * 100.0);
            }
        }
    }
}

pub fn game_length_tracking_system(
    mut results: ResMut<BalanceTestResults>,
    config: Res<GameConfig>,
    current_state: Res<State<GameState>>,
) {
    if matches!(current_state.get(), GameState::GameOver) {
        let game_length = config.current_year as f32;
        
        if results.games_played > 0 {
            results.average_game_length = (results.average_game_length * (results.games_played - 1) as f32 + game_length) / results.games_played as f32;
        } else {
            results.average_game_length = game_length;
        }
        
        if results.average_game_length < 4.0 {
            info!("Games ending too quickly - consider increasing VP requirement");
        } else if results.average_game_length > 8.0 {
            info!("Games taking too long - consider decreasing VP requirement");
        }
    }
}