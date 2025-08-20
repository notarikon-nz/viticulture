use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlock_condition: AchievementCondition,
    pub unlocked: bool,
    pub unlock_date: Option<String>,
    pub progress: u32,
    pub target: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum AchievementCondition {
    WinFirstGame,
    WinGames(u32),
    ReachVP(u8),
    PlantVines(u32),
    FulfillOrders(u32),
    BuildStructures(u32),
    EarnLira(u32),
    CompleteYear(u8),
    WinStreak(u32),
    UseAllActions,
    FastWin(f32), // Win in under X seconds
    PerfectGame, // Win without losing any resources
}

#[derive(Serialize, Deserialize, Resource, Default)]
pub struct AchievementManager {
    pub achievements: Vec<Achievement>,
    pub unlocked_this_session: Vec<String>,
}

impl AchievementManager {
    pub fn new() -> Self {
        let mut manager = Self {
            achievements: create_achievements(),
            unlocked_this_session: Vec::new(),
        };
        manager.load_progress();
        manager
    }
    
    pub fn load_progress(&mut self) {
        if let Ok(json) = std::fs::read_to_string("viticulture_achievements.json") {
            if let Ok(saved_achievements) = serde_json::from_str::<Vec<Achievement>>(&json) {
                // Merge saved progress with current achievements
                for saved in saved_achievements {
                    if let Some(achievement) = self.achievements.iter_mut().find(|a| a.id == saved.id) {
                        achievement.unlocked = saved.unlocked;
                        achievement.unlock_date = saved.unlock_date;
                        achievement.progress = saved.progress;
                    }
                }
            }
        }
    }
    
    pub fn save_progress(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.achievements) {
            let _ = std::fs::write("viticulture_achievements.json", json);
        }
    }
    
    pub fn check_achievement(&mut self, condition: &AchievementCondition, current_value: u32) -> Vec<String> {
        let mut newly_unlocked = Vec::new();
        
        for achievement in &mut self.achievements {
            if achievement.unlocked || achievement.unlock_condition != *condition {
                continue;
            }
            
            achievement.progress = current_value;
            
            let should_unlock = match &achievement.unlock_condition {
                AchievementCondition::WinFirstGame => current_value >= 1,
                AchievementCondition::WinGames(target) => current_value >= *target,
                AchievementCondition::ReachVP(target) => current_value >= *target as u32,
                AchievementCondition::PlantVines(target) => current_value >= *target,
                AchievementCondition::FulfillOrders(target) => current_value >= *target,
                AchievementCondition::BuildStructures(target) => current_value >= *target,
                AchievementCondition::EarnLira(target) => current_value >= *target,
                AchievementCondition::CompleteYear(target) => current_value >= *target as u32,
                AchievementCondition::WinStreak(target) => current_value >= *target,
                AchievementCondition::FastWin(target_time) => (current_value as f32) <= *target_time,
                AchievementCondition::UseAllActions | AchievementCondition::PerfectGame => current_value >= 1,
            };
            
            if should_unlock {
                achievement.unlocked = true;
                achievement.unlock_date = Some(chrono::Utc::now().format("%Y-%m-%d").to_string());
                newly_unlocked.push(achievement.id.clone());
                self.unlocked_this_session.push(achievement.id.clone());
                
                info!("Achievement Unlocked: {}", achievement.name);
            }
        }
        
        if !newly_unlocked.is_empty() {
            self.save_progress();
        }
        
        newly_unlocked
    }
    
    pub fn get_completion_percentage(&self) -> f32 {
        let total = self.achievements.len() as f32;
        let unlocked = self.achievements.iter().filter(|a| a.unlocked).count() as f32;
        if total > 0.0 { unlocked / total * 100.0 } else { 0.0 }
    }
}

fn create_achievements() -> Vec<Achievement> {
    vec![
        Achievement {
            id: "first_win".to_string(),
            name: "First Victory".to_string(),
            description: "Win your first game of Viticulture".to_string(),
            unlock_condition: AchievementCondition::WinFirstGame,
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 1,
        },
        Achievement {
            id: "vintner".to_string(),
            name: "Vintner".to_string(),
            description: "Win 5 games".to_string(),
            unlock_condition: AchievementCondition::WinGames(5),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 5,
        },
        Achievement {
            id: "master_vintner".to_string(),
            name: "Master Vintner".to_string(),
            description: "Win 25 games".to_string(),
            unlock_condition: AchievementCondition::WinGames(25),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 25,
        },
        Achievement {
            id: "high_scorer".to_string(),
            name: "High Scorer".to_string(),
            description: "Reach 30 victory points in a single game".to_string(),
            unlock_condition: AchievementCondition::ReachVP(30),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 30,
        },
        Achievement {
            id: "green_thumb".to_string(),
            name: "Green Thumb".to_string(),
            description: "Plant 50 vines across all games".to_string(),
            unlock_condition: AchievementCondition::PlantVines(50),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 50,
        },
        Achievement {
            id: "customer_favorite".to_string(),
            name: "Customer Favorite".to_string(),
            description: "Fulfill 100 wine orders across all games".to_string(),
            unlock_condition: AchievementCondition::FulfillOrders(100),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 100,
        },
        Achievement {
            id: "architect".to_string(),
            name: "Architect".to_string(),
            description: "Build 20 structures across all games".to_string(),
            unlock_condition: AchievementCondition::BuildStructures(20),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 20,
        },
        Achievement {
            id: "wealthy".to_string(),
            name: "Wealthy Vintner".to_string(),
            description: "Earn 500 lira across all games".to_string(),
            unlock_condition: AchievementCondition::EarnLira(500),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 500,
        },
        Achievement {
            id: "veteran".to_string(),
            name: "Veteran Player".to_string(),
            description: "Complete 10 full years (70 total years)".to_string(),
            unlock_condition: AchievementCondition::CompleteYear(70),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 70,
        },
        Achievement {
            id: "streak".to_string(),
            name: "Hot Streak".to_string(),
            description: "Win 3 games in a row".to_string(),
            unlock_condition: AchievementCondition::WinStreak(3),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 3,
        },
        Achievement {
            id: "versatile".to_string(),
            name: "Versatile Vintner".to_string(),
            description: "Use every action type in a single game".to_string(),
            unlock_condition: AchievementCondition::UseAllActions,
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 1,
        },
        Achievement {
            id: "speed_demon".to_string(),
            name: "Speed Demon".to_string(),
            description: "Win a game in under 5 minutes".to_string(),
            unlock_condition: AchievementCondition::FastWin(300.0),
            unlocked: false,
            unlock_date: None,
            progress: 0,
            target: 300,
        },
    ]
}

#[derive(Component)]
pub struct AchievementNotification {
    pub timer: Timer,
    pub achievement_name: String,
}

#[derive(Component)]
pub struct AchievementUI;

pub fn initialize_achievements_system(mut commands: Commands) {
    let manager = AchievementManager::new();
    commands.insert_resource(manager);
}

pub fn achievement_tracking_system(
    mut achievement_manager: ResMut<AchievementManager>,
    mut commands: Commands,
    players: Query<&Player>,
    vineyards: Query<&Vineyard>,
    workers: Query<&Worker>,
    game_stats: Res<crate::systems::statistics::GameStatistics>,
    current_state: Res<State<GameState>>,
    config: Res<GameConfig>,
) {
    // Track various achievements
    for player in players.iter() {
        // High VP achievement
        let unlocked = achievement_manager.check_achievement(
            &AchievementCondition::ReachVP(30),
            player.victory_points as u32,
        );
        for achievement_id in unlocked {
            show_achievement_notification(&mut commands, &achievement_manager, &achievement_id);
        }
    }
    
    // Game completion achievements
    if matches!(current_state.get(), GameState::GameOver) {
        // Win-based achievements
        let unlocked = achievement_manager.check_achievement(
            &AchievementCondition::WinGames(5),
            game_stats.total_games_won,
        );
        for achievement_id in unlocked {
            show_achievement_notification(&mut commands, &achievement_manager, &achievement_id);
        }
        
        // Win streak achievement
        let unlocked = achievement_manager.check_achievement(
            &AchievementCondition::WinStreak(3),
            game_stats.current_streak,
        );
        for achievement_id in unlocked {
            show_achievement_notification(&mut commands, &achievement_manager, &achievement_id);
        }
    }
    
    // Cumulative achievements
    let unlocked = achievement_manager.check_achievement(
        &AchievementCondition::EarnLira(500),
        game_stats.total_lira_earned,
    );
    for achievement_id in unlocked {
        show_achievement_notification(&mut commands, &achievement_manager, &achievement_id);
    }
    
    // Year completion achievement
    let total_years = game_stats.total_games_played * 7; // Approximate
    let unlocked = achievement_manager.check_achievement(
        &AchievementCondition::CompleteYear(70),
        total_years,
    );
    for achievement_id in unlocked {
        show_achievement_notification(&mut commands, &achievement_manager, &achievement_id);
    }
}

fn show_achievement_notification(
    commands: &mut Commands,
    achievement_manager: &AchievementManager,
    achievement_id: &str,
) {
    if let Some(achievement) = achievement_manager.achievements.iter().find(|a| a.id == achievement_id) {
        commands.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    right: Val::Px(20.0),
                    width: Val::Px(300.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    ..default()
                },
                background_color: Color::from(Srgba::new(0.8, 0.6, 0.0, 0.95)).into(),
                z_index: ZIndex::Global(1200),
                ..default()
            },
            AchievementNotification {
                timer: Timer::from_seconds(4.0, TimerMode::Once),
                achievement_name: achievement.name.clone(),
            },
        )).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                format!("🏆 ACHIEVEMENT UNLOCKED!\n\n{}\n{}", achievement.name, achievement.description),
                TextStyle {
                    font_size: 14.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
    }
}

pub fn achievement_notification_system(
    mut commands: Commands,
    time: Res<Time>,
    mut notifications: Query<(Entity, &mut AchievementNotification), Without<MarkedForDespawn>>,
) {
    for (entity, mut notification) in notifications.iter_mut() {
        notification.timer.tick(time.delta());
        
        if notification.timer.finished() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

pub fn achievement_menu_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    achievement_manager: Res<AchievementManager>,
    existing_ui: Query<Entity, (With<AchievementUI>,Without<MarkedForDespawn>)>,
) {
    if keyboard.just_pressed(KeyCode::KeyA) {
        if existing_ui.is_empty() {
            show_achievement_menu(&mut commands, &achievement_manager);
        } else {
            for entity in existing_ui.iter() {
                commands.entity(entity).insert(MarkedForDespawn);
            }
        }
    }
}

fn show_achievement_menu(commands: &mut Commands, achievement_manager: &AchievementManager) {
    let completion_percent = achievement_manager.get_completion_percentage();
    let unlocked_count = achievement_manager.achievements.iter().filter(|a| a.unlocked).count();
    let total_count = achievement_manager.achievements.len();
    
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::from(Srgba::new(0.0, 0.0, 0.0, 0.8)).into(),
            z_index: ZIndex::Global(1000),
            ..default()
        },
        AchievementUI,
    )).with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(600.0),
                height: Val::Px(500.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.1, 0.1, 0.1, 0.95)).into(),
            ..default()
        }).with_children(|panel| {
            // Header
            panel.spawn(TextBundle::from_section(
                format!("🏆 ACHIEVEMENTS ({}/{})\nCompletion: {:.1}%\n\nPress A to close", 
                        unlocked_count, total_count, completion_percent),
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            }));
            
            // Achievement list
            for achievement in &achievement_manager.achievements {
                let status_icon = if achievement.unlocked { "✅" } else { "⬜" };
                let text_color = if achievement.unlocked { 
                    Color::from(Srgba::new(0.8, 1.0, 0.8, 1.0))
                } else { 
                    Color::from(Srgba::new(0.6, 0.6, 0.6, 1.0))
                };
                
                let progress_text = if achievement.unlocked {
                    format!("Unlocked on {}", achievement.unlock_date.as_ref().unwrap_or(&"Unknown".to_string()))
                } else if achievement.target > 1 {
                    format!("Progress: {}/{}", achievement.progress, achievement.target)
                } else {
                    "Not unlocked".to_string()
                };
                
                panel.spawn(NodeBundle {
                    style: Style {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: Color::from(Srgba::new(0.2, 0.2, 0.2, 0.5)).into(),
                    ..default()
                }).with_children(|achievement_item| {
                    achievement_item.spawn(TextBundle::from_section(
                        format!("{} {}\n{}\n{}", 
                                status_icon, achievement.name, achievement.description, progress_text),
                        TextStyle {
                            font_size: 14.0,
                            color: text_color,
                            ..default()
                        },
                    ));
                });
            }
        });
    });
}