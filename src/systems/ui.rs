use bevy::prelude::*;
use crate::components::*;

const YELLOW: Srgba = Srgba::new(1.0, 1.0, 0.0, 1.0);
const GOLD: Srgba = Srgba::new(1.0, 0.84, 0.0, 1.0);

pub fn main_menu_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    text_query: Query<Entity, With<Text>>,
) {
    if text_query.is_empty() {
        commands.spawn(TextBundle::from_section(
            "VITICULTURE DIGITAL\n\nPress SPACE to Start Game",
            TextStyle {
                font_size: 40.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(200.0),
            left: Val::Px(50.0),
            ..default()
        }));
    }
    
    if keyboard.just_pressed(KeyCode::Space) {
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::Setup);
    }
}

pub fn setup_ui(commands: &mut Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        },
        UIPanel,
    )).with_children(|parent| {
        // Top status bar
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.2, 0.2, 0.2, 0.9)).into(),
            ..default()
        }).with_children(|status_bar| {
            status_bar.spawn((
                TextBundle::from_section(
                    "Game Starting...",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                GameStatusText,
            ));
            
            status_bar.spawn((
                TextBundle::from_section(
                    "Player 1's Turn",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::from(Srgba::new(1.0, 1.0, 0.0, 1.0)),
                        ..default()
                    },
                ),
                TurnIndicator,
            ));
        });
        
        // Main game area
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        }).with_children(|main_area| {
            setup_action_board(main_area);
            setup_player_dashboards(main_area);
        });
    });
}

fn setup_action_board(parent: &mut ChildBuilder) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(50.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        background_color: Color::from(Srgba::new(0.1, 0.1, 0.1, 0.8)).into(),
        ..default()
    }).with_children(|action_area| {
        action_area.spawn(TextBundle::from_section(
            "SUMMER ACTIONS",
            TextStyle {
                font_size: 18.0,
                color: Color::from(Srgba::new(1.0, 1.0, 0.5, 1.0)),
                ..default()
            },
        ));
        
        // Summer actions with bonus worker spaces
        let summer_actions = [
            (ActionSpace::DrawVine, false),
            (ActionSpace::PlantVine, true), // Has bonus worker space
            (ActionSpace::BuildStructure, false),
            (ActionSpace::GiveTour, true), // Has bonus worker space
            (ActionSpace::SellGrapes, false),
            (ActionSpace::TrainWorker, false),
        ];
        
        for (action, has_bonus) in summer_actions {
            let button_text = if has_bonus {
                format!("{:?} (+1)", action)
            } else {
                format!("{:?}", action)
            };
            
            action_area.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::from(Srgba::new(0.8, 0.8, 0.2, 0.8)).into(),
                    ..default()
                },
                ActionButton { action },
            )).with_children(|button| {
                button.spawn(TextBundle::from_section(
                    button_text,
                    TextStyle {
                        font_size: 16.0,
                        color: Color::BLACK,
                        ..default()
                    },
                ));
            });
        }
        
        action_area.spawn(TextBundle::from_section(
            "WINTER ACTIONS",
            TextStyle {
                font_size: 18.0,
                color: Color::from(Srgba::new(0.5, 0.5, 1.0, 1.0)),
                ..default()
            },
        ));
        
        // Winter actions with bonus worker spaces
        let winter_actions = [
            (ActionSpace::DrawWineOrder, false),
            (ActionSpace::Harvest, true), // Has bonus worker space
            (ActionSpace::MakeWine, true), // Has bonus worker space
            (ActionSpace::FillOrder, false),
        ];
        
        for (action, has_bonus) in winter_actions {
            let button_text = if has_bonus {
                format!("{:?} (+1)", action)
            } else {
                format!("{:?}", action)
            };
            
            action_area.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::from(Srgba::new(0.2, 0.2, 0.8, 0.8)).into(),
                    ..default()
                },
                ActionButton { action },
            )).with_children(|button| {
                button.spawn(TextBundle::from_section(
                    button_text,
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        }
    });
}

fn setup_player_dashboards(parent: &mut ChildBuilder) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(50.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        ..default()
    }).with_children(|dashboard_area| {
        for i in 0..2 {
            dashboard_area.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(45.0),
                        margin: UiRect::all(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: Color::from(Srgba::new(0.15, 0.15, 0.15, 0.9)).into(),
                    border_color: Color::from(Srgba::new(0.5, 0.5, 0.5, 1.0)).into(),
                    ..default()
                },
                PlayerDashboard { player_id: PlayerId(i) },
            )).with_children(|dashboard| {
                dashboard.spawn(TextBundle::from_section(
                    format!("Player {}", i + 1),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
                
                dashboard.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                }).with_children(|resources| {
                    resources.spawn(TextBundle::from_section(
                        "VP: 0",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::from(YELLOW),
                            ..default()
                        },
                    ));
                    resources.spawn(TextBundle::from_section(
                        "Lira: 3",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::from(GOLD),
                            ..default()
                        },
                    ));
                });
                
                dashboard.spawn(TextBundle::from_section(
                    "Grapes: R:0 W:0 | Wine: R:0 W:0",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::from(Srgba::new(0.8, 0.8, 0.8, 1.0)),
                        ..default()
                    },
                ));
                
                dashboard.spawn(TextBundle::from_section(
                    "Hand: Vines:0 Orders:0 | Workers: 2+1G",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::from(Srgba::new(0.6, 0.8, 0.6, 1.0)),
                        ..default()
                    },
                ));
                
                dashboard.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(120.0),
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: Color::from(Srgba::new(0.1, 0.1, 0.1, 0.5)).into(),
                    ..default()
                });
            });
        }
    });
}

pub fn update_ui_system(
    mut status_query: Query<&mut Text, (With<GameStatusText>, Without<TurnIndicator>)>,
    mut turn_query: Query<&mut Text, (With<TurnIndicator>, Without<GameStatusText>)>,
    players: Query<&Player>,
    turn_order: Res<TurnOrder>,
    current_state: Res<State<GameState>>,
    config: Res<GameConfig>,
) {
    if let Ok(mut status_text) = status_query.get_single_mut() {
        let mut leading_player = "None";
        let mut highest_vp = 0;
        
        for player in players.iter() {
            if player.victory_points > highest_vp {
                highest_vp = player.victory_points;
                leading_player = &player.name;
            }
        }
        
        status_text.sections[0].value = format!(
            "Year {} | Leader: {} ({} VP) | Target: {} VP",
            config.current_year, leading_player, highest_vp, config.target_victory_points
        );
    }
    
    if let Ok(mut turn_text) = turn_query.get_single_mut() {
        if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
            let phase = match current_state.get() {
                GameState::Summer => "Summer",
                GameState::Winter => "Winter",
                GameState::Spring => "Spring",
                GameState::Fall => "Fall",
                _ => "Game",
            };
            turn_text.sections[0].value = format!("{} - Player {}'s Turn", phase, current_player_id.0 + 1);
        }
    }
}

pub fn ui_game_over_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut commands: Commands,
    ui_query: Query<Entity, With<UIPanel>>,
) {
    if matches!(current_state.get(), GameState::GameOver) && keyboard.just_pressed(KeyCode::Space) {
        for entity in ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        next_state.set(GameState::MainMenu);
    }
}