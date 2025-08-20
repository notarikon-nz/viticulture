use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::*;

#[derive(Serialize, Deserialize, Resource, Default)]
pub struct OnboardingState {
    pub first_time_player: bool,
    pub games_played: u32,
    pub tutorial_offered: bool,
    pub help_shown: Vec<String>,
    pub tips_seen: Vec<String>,
}

impl OnboardingState {
    pub fn load_or_default() -> Self {
        match std::fs::read_to_string("viticulture_onboarding.json") {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self {
                first_time_player: true,
                games_played: 0,
                tutorial_offered: false,
                help_shown: Vec::new(),
                tips_seen: Vec::new(),
            }
        }
    }
    
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("viticulture_onboarding.json", json);
        }
    }
    
    pub fn is_new_player(&self) -> bool {
        self.games_played < 3
    }
    
    pub fn should_show_tip(&self, tip_id: &str) -> bool {
        !self.tips_seen.contains(&tip_id.to_string())
    }
    
    pub fn mark_tip_seen(&mut self, tip_id: &str) {
        if !self.tips_seen.contains(&tip_id.to_string()) {
            self.tips_seen.push(tip_id.to_string());
            self.save();
        }
    }
}

#[derive(Component)]
pub struct OnboardingUI;

#[derive(Component)]
pub struct GameplayTip {
    pub tip_id: String,
    pub display_timer: Timer,
}

#[derive(Clone)]
pub struct Tip {
    pub id: String,
    pub title: String,
    pub content: String,
    pub trigger: TipTrigger,
    pub priority: u8, // Higher = more important
}

#[derive(Clone, PartialEq)]
pub enum TipTrigger {
    GameStart,
    FirstSummer,
    FirstWinter,
    LowResources,
    HighVP,
    Phase(GameState),
    Action(ActionSpace),
}

pub fn initialize_onboarding_system(mut commands: Commands) {
    let onboarding = OnboardingState::load_or_default();
    commands.insert_resource(onboarding);
}

pub fn welcome_screen_system(
    mut onboarding: ResMut<OnboardingState>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    existing_welcome: Query<Entity, With<OnboardingUI>>,
) {
    if matches!(current_state.get(), GameState::MainMenu) && onboarding.first_time_player && existing_welcome.is_empty() {
        show_welcome_screen(&mut commands, &onboarding);
    }
    
    // Handle welcome screen input
    if !existing_welcome.is_empty() {
        if keyboard.just_pressed(KeyCode::KeyT) {
            // Start tutorial
            onboarding.tutorial_offered = true;
            onboarding.first_time_player = false;
            onboarding.save();
            
            for entity in existing_welcome.iter() {
                commands.entity(entity).despawn_recursive();
            }
            
            info!("Starting tutorial from welcome screen");
        } else if keyboard.just_pressed(KeyCode::Space) {
            // Skip tutorial, start game
            onboarding.first_time_player = false;
            onboarding.save();
            
            for entity in existing_welcome.iter() {
                commands.entity(entity).despawn_recursive();
            }
        } else if keyboard.just_pressed(KeyCode::Escape) {
            // Dismiss welcome, stay at menu
            onboarding.first_time_player = false;
            onboarding.save();
            
            for entity in existing_welcome.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn show_welcome_screen(commands: &mut Commands, onboarding: &OnboardingState) {
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
            background_color: Color::from(Srgba::new(0.0, 0.0, 0.0, 0.9)).into(),
            z_index: ZIndex::Global(1500),
            ..default()
        },
        OnboardingUI,
    )).with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(500.0),
                padding: UiRect::all(Val::Px(30.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::from(Srgba::new(0.1, 0.1, 0.1, 0.95)).into(),
            ..default()
        }).with_children(|panel| {
            panel.spawn(TextBundle::from_section(
                "🍷 Welcome to Viticulture! 🍷",
                TextStyle {
                    font_size: 28.0,
                    color: Color::from(Srgba::new(0.8, 0.6, 0.2, 1.0)),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            }));
            
            panel.spawn(TextBundle::from_section(
                "It looks like this is your first time playing!\n\n\
                 Viticulture is a worker placement game where you:\n\
                 • Plant vines and harvest grapes\n\
                 • Make wine from your grapes\n\
                 • Fulfill wine orders for victory points\n\
                 • First to 20 VP wins!\n\n\
                 Choose your experience:",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            }));
            
            // Tutorial option
            panel.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                background_color: Color::from(Srgba::new(0.0, 0.4, 0.0, 0.8)).into(),
                ..default()
            }).with_children(|option| {
                option.spawn(TextBundle::from_section(
                    "📚 TUTORIAL (Recommended)\nPress T - Learn step-by-step with guided gameplay",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
            
            // Jump in option
            panel.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(15.0)),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                background_color: Color::from(Srgba::new(0.4, 0.4, 0.0, 0.8)).into(),
                ..default()
            }).with_children(|option| {
                option.spawn(TextBundle::from_section(
                    "🚀 JUMP RIGHT IN\nPress SPACE - Start playing with helpful tips",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
            
            // Help info
            panel.spawn(TextBundle::from_section(
                "💡 You can always access help with F1 (contextual) or F2 (quick reference)\n\
                 Press ESC to dismiss this screen",
                TextStyle {
                    font_size: 12.0,
                    color: Color::from(Srgba::new(0.7, 0.7, 0.7, 1.0)),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            }));
        });
    });
}

pub fn gameplay_tips_system(
    mut onboarding: ResMut<OnboardingState>,
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    players: Query<&Player>,
    vineyards: Query<&Vineyard>,
    turn_order: Res<TurnOrder>,
    workers: Query<&Worker>,
) {
    if !onboarding.is_new_player() {
        return;
    }
    
    let tips = get_contextual_tips();
    
    for tip in tips {
        if !onboarding.should_show_tip(&tip.id) {
            continue;
        }
        
        let should_show = match &tip.trigger {
            TipTrigger::GameStart => matches!(current_state.get(), GameState::Setup),
            TipTrigger::FirstSummer => matches!(current_state.get(), GameState::Summer),
            TipTrigger::FirstWinter => matches!(current_state.get(), GameState::Winter),
            TipTrigger::Phase(phase) => current_state.get() == phase,
            TipTrigger::LowResources => {
                if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
                    if let Some(vineyard) = vineyards.iter().find(|v| v.owner == *current_player_id) {
                        vineyard.lira < 2
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            TipTrigger::HighVP => {
                players.iter().any(|p| p.victory_points >= 15)
            }
            _ => false,
        };
        
        if should_show {
            show_gameplay_tip(&mut commands, &tip);
            onboarding.mark_tip_seen(&tip.id);
            break; // Show only one tip at a time
        }
    }
}

fn get_contextual_tips() -> Vec<Tip> {
    vec![
        Tip {
            id: "game_start".to_string(),
            title: "Getting Started".to_string(),
            content: "Welcome to your first game! You start with 2 workers, 1 grande worker, and 3 lira. The goal is to reach 20 victory points by fulfilling wine orders.".to_string(),
            trigger: TipTrigger::GameStart,
            priority: 10,
        },
        Tip {
            id: "first_summer".to_string(),
            title: "Summer Phase".to_string(),
            content: "Summer is for preparation! Focus on drawing vine cards, planting vines, and building your economy. You'll need vines planted to harvest grapes later.".to_string(),
            trigger: TipTrigger::FirstSummer,
            priority: 9,
        },
        Tip {
            id: "first_winter".to_string(),
            title: "Winter Phase".to_string(),
            content: "Winter is for production and scoring! Harvest grapes, make wine, and fulfill orders for victory points. This is how you win the game.".to_string(),
            trigger: TipTrigger::FirstWinter,
            priority: 9,
        },
        Tip {
            id: "low_resources".to_string(),
            title: "Low on Lira".to_string(),
            content: "Running low on money? Try giving tours for quick lira, or sell your grapes if you have any. Managing your economy is crucial!".to_string(),
            trigger: TipTrigger::LowResources,
            priority: 8,
        },
        Tip {
            id: "high_vp".to_string(),
            title: "Victory in Sight".to_string(),
            content: "Someone is close to winning! Focus on fulfilling wine orders for victory points. Consider using your grande worker to secure key actions.".to_string(),
            trigger: TipTrigger::HighVP,
            priority: 7,
        },
        Tip {
            id: "spring_wakeup".to_string(),
            title: "Wake-up Order".to_string(),
            content: "Choose your wake-up time wisely! Earlier positions give better bonuses but you'll act later in turn order. Balance risk and reward.".to_string(),
            trigger: TipTrigger::Phase(GameState::Spring),
            priority: 6,
        },
        Tip {
            id: "fall_harvest".to_string(),
            title: "Automatic Harvest".to_string(),
            content: "Fall automatically harvests ALL your planted vines. No actions needed! Make sure you have space to store the grapes.".to_string(),
            trigger: TipTrigger::Phase(GameState::Fall),
            priority: 5,
        },
    ]
}

fn show_gameplay_tip(commands: &mut Commands, tip: &Tip) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(350.0),
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.0, 0.0, 0.4, 0.9)).into(),
            z_index: ZIndex::Global(700),
            ..default()
        },
        GameplayTip {
            tip_id: tip.id.clone(),
            display_timer: Timer::from_seconds(6.0, TimerMode::Once),
        },
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            format!("💡 TIP: {}\n\n{}", tip.title, tip.content),
            TextStyle {
                font_size: 13.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

pub fn tip_cleanup_system(
    mut commands: Commands,
    time: Res<Time>,
    mut tips: Query<(Entity, &mut GameplayTip)>,
) {
    for (entity, mut tip) in tips.iter_mut() {
        tip.display_timer.tick(time.delta());
        
        if tip.display_timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn new_player_encouragement_system(
    mut onboarding: ResMut<OnboardingState>,
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    players: Query<&Player>,
    game_stats: Res<crate::systems::statistics::GameStatistics>,
) {
    if !onboarding.is_new_player() {
        return;
    }
    
    // Show encouragement when game ends
    if current_state.is_changed() && matches!(current_state.get(), GameState::GameOver) {
        onboarding.games_played += 1;
        onboarding.save();
        
        let encouragement_text = match onboarding.games_played {
            1 => {
                "🎉 First Game Complete!\n\n\
                 Great job finishing your first game! Every game is a learning experience.\n\
                 Try focusing on fulfilling wine orders for victory points next time.\n\n\
                 Press SPACE to continue"
            }
            2 => {
                "📈 Second Game Done!\n\n\
                 You're getting the hang of it! Consider trying different strategies:\n\
                 • Build structures for ongoing benefits\n\
                 • Train extra workers for more actions\n\
                 • Use your grande worker strategically\n\n\
                 Press SPACE to continue"
            }
            3 => {
                "🏆 Third Game Complete!\n\n\
                 You're no longer a beginner! You now have access to all features.\n\
                 Try the expansions (F6-F8) or check out achievements (A key).\n\n\
                 Press SPACE to continue"
            }
            _ => return,
        };
        
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
                background_color: Color::from(Srgba::new(0.0, 0.3, 0.0, 0.9)).into(),
                z_index: ZIndex::Global(1100),
                ..default()
            },
            OnboardingUI,
        )).with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(450.0),
                    padding: UiRect::all(Val::Px(25.0)),
                    ..default()
                },
                background_color: Color::from(Srgba::new(0.1, 0.1, 0.1, 0.95)).into(),
                ..default()
            }).with_children(|panel| {
                panel.spawn(TextBundle::from_section(
                    encouragement_text,
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        });
    }
}

pub fn onboarding_cleanup_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    onboarding_ui: Query<Entity, With<OnboardingUI>>,
) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Escape) {
        for entity in onboarding_ui.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}