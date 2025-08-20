use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::*;

#[derive(Serialize, Deserialize, Resource, Default)]
pub struct GameStatistics {
    pub total_games_played: u32,
    pub total_games_won: u32,
    pub total_time_played: f32, // in seconds
    pub highest_victory_points: u8,
    pub fastest_win_time: f32, // in seconds
    pub favorite_actions: std::collections::HashMap<u8, u32>, // ActionSpace -> usage count
    pub win_streak: u32,
    pub current_streak: u32,
    pub average_game_length: f32,
    pub total_vp_earned: u32,
    pub total_lira_earned: u32,
}

#[derive(Resource, Default)]
pub struct SessionStats {
    pub session_start_time: f32,
    pub current_game_start: f32,
    pub actions_this_game: Vec<ActionSpace>,
    pub vp_this_game: u8,
    pub lira_this_game: u8,
}

impl GameStatistics {
    pub fn load_or_default() -> Self {
        match std::fs::read_to_string("viticulture_stats.json") {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
    
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("viticulture_stats.json", json);
        }
    }
    
    pub fn games_win_rate(&self) -> f32 {
        if self.total_games_played == 0 {
            0.0
        } else {
            self.total_games_won as f32 / self.total_games_played as f32 * 100.0
        }
    }
    
    pub fn average_vp_per_game(&self) -> f32 {
        if self.total_games_played == 0 {
            0.0
        } else {
            self.total_vp_earned as f32 / self.total_games_played as f32
        }
    }
    
    pub fn most_used_action(&self) -> Option<ActionSpace> {
        self.favorite_actions.iter()
            .max_by_key(|(_, count)| *count)
            .and_then(|(action_id, _)| u8_to_action(*action_id))
    }
}

pub fn initialize_session_system(
    mut commands: Commands,
    time: Res<Time>,
) {
    let stats = GameStatistics::load_or_default();
    commands.insert_resource(stats);
    commands.insert_resource(SessionStats {
        session_start_time: time.elapsed_seconds(),
        current_game_start: time.elapsed_seconds(),
        actions_this_game: Vec::new(),
        vp_this_game: 0,
        lira_this_game: 0,
    });
}

pub fn track_session_system(
    mut session_stats: ResMut<SessionStats>,
    time: Res<Time>,
    current_state: Res<State<GameState>>,
    players: Query<&Player>,
    turn_order: Res<TurnOrder>,
) {
    // Reset game timer when new game starts
    if current_state.is_changed() && matches!(current_state.get(), GameState::Setup) {
        session_stats.current_game_start = time.elapsed_seconds();
        session_stats.actions_this_game.clear();
        session_stats.vp_this_game = 0;
        session_stats.lira_this_game = 0;
    }
    
    // Track current player's progress
    if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
        if let Some(player) = players.iter().find(|p| p.id == *current_player_id) {
            session_stats.vp_this_game = session_stats.vp_this_game.max(player.victory_points);
            session_stats.lira_this_game = session_stats.lira_this_game.max(player.lira);
        }
    }
}

pub fn track_action_usage_system(
    mut session_stats: ResMut<SessionStats>,
    workers: Query<&Worker, Changed<Worker>>,
) {
    for worker in workers.iter() {
        if let Some(action) = worker.placed_at {
            session_stats.actions_this_game.push(action);
        }
    }
}

pub fn update_statistics_on_game_end_system(
    mut stats: ResMut<GameStatistics>,
    session_stats: ResMut<SessionStats>,
    time: Res<Time>,
    current_state: Res<State<GameState>>,
    players: Query<&Player>,
    turn_order: Res<TurnOrder>,
) {
    if current_state.is_changed() && matches!(current_state.get(), GameState::GameOver) {
        let game_duration = time.elapsed_seconds() - session_stats.current_game_start;
        
        // Find if current player won
        let current_player_won = if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
            players.iter()
                .max_by_key(|p| p.victory_points)
                .map(|winner| winner.id == *current_player_id)
                .unwrap_or(false)
        } else {
            false
        };
        
        // Update statistics
        stats.total_games_played += 1;
        if current_player_won {
            stats.total_games_won += 1;
            stats.current_streak += 1;
            stats.win_streak = stats.win_streak.max(stats.current_streak);
            
            // Track fastest win
            if stats.fastest_win_time == 0.0 || game_duration < stats.fastest_win_time {
                stats.fastest_win_time = game_duration;
            }
        } else {
            stats.current_streak = 0;
        }
        
        // Update general stats
        stats.total_time_played += game_duration;
        stats.total_vp_earned += session_stats.vp_this_game as u32;
        stats.total_lira_earned += session_stats.lira_this_game as u32;
        
        // Update highest VP
        stats.highest_victory_points = stats.highest_victory_points.max(session_stats.vp_this_game);
        
        // Update average game length
        stats.average_game_length = (stats.average_game_length * (stats.total_games_played - 1) as f32 + game_duration) / stats.total_games_played as f32;
        
        // Track favorite actions
        for action in &session_stats.actions_this_game {
            let action_id = action_to_u8(*action);
            *stats.favorite_actions.entry(action_id).or_insert(0) += 1;
        }
        
        // Save to file
        stats.save();
        
        info!("Game statistics updated - Games: {}, Win Rate: {:.1}%, Streak: {}", 
              stats.total_games_played, stats.games_win_rate(), stats.current_streak);
    }
}

pub fn display_statistics_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    stats: Res<GameStatistics>,
    session_stats: Res<SessionStats>,
    time: Res<Time>,
    existing_stats_ui: Query<Entity, With<StatsPanel>>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        if existing_stats_ui.is_empty() {
            // Show statistics panel
            let session_time = time.elapsed_seconds() - session_stats.session_start_time;
            
            let stats_text = format!(
                "📊 GAME STATISTICS 📊\n\
                 \n\
                 🎮 CAREER STATS:\n\
                 Games Played: {}\n\
                 Games Won: {} ({:.1}%)\n\
                 Current Win Streak: {}\n\
                 Best Win Streak: {}\n\
                 \n\
                 🏆 RECORDS:\n\
                 Highest VP: {}\n\
                 Fastest Win: {:.1}s\n\
                 Avg Game Length: {:.1}s\n\
                 \n\
                 💰 TOTALS:\n\
                 Total VP Earned: {}\n\
                 Total Lira Earned: {}\n\
                 Total Time Played: {:.1}h\n\
                 \n\
                 ⚡ SESSION:\n\
                 Session Time: {:.1}m\n\
                 Actions This Game: {}\n\
                 \n\
                 Press TAB to close",
                stats.total_games_played,
                stats.total_games_won, stats.games_win_rate(),
                stats.current_streak,
                stats.win_streak,
                stats.highest_victory_points,
                stats.fastest_win_time,
                stats.average_game_length,
                stats.total_vp_earned,
                stats.total_lira_earned,
                stats.total_time_played / 3600.0,
                session_time / 60.0,
                session_stats.actions_this_game.len()
            );
            
            commands.spawn((
                TextBundle::from_section(
                    stats_text,
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ).with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    left: Val::Px(50.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                }),
                StatsPanel,
            ));
            
            // Semi-transparent background
            commands.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: Color::from(Srgba::new(0.0, 0.0, 0.0, 0.7)).into(),
                    z_index: ZIndex::Global(100),
                    ..default()
                },
                StatsPanel,
            ));
        } else {
            // Hide statistics panel
            for entity in existing_stats_ui.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

#[derive(Component)]
pub struct StatsPanel;

fn action_to_u8(action: ActionSpace) -> u8 {
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

fn u8_to_action(value: u8) -> Option<ActionSpace> {
    match value {
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