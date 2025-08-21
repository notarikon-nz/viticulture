use bevy::prelude::*;
use crate::components::*;

const YELLOW: Srgba = Srgba::new(1.0, 1.0, 0.0, 1.0);
const GOLD: Srgba = Srgba::new(1.0, 0.84, 0.0, 1.0);

pub fn main_menu_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
    text_query: Query<Entity, With<PhaseText>>, // Changed query
) {
    if text_query.is_empty() {
        commands.spawn((
            TextBundle::from_section(
                "VITICULTURE - Enhanced Edition\n\nPress SPACE to Start Game\nPress 1-4 to set player count\nPress A to cycle AI count\nPress C to view player cards in-game",
                TextStyle {
                    font_size: 28.0,
                    color: Color::WHITE,
                    ..default()
                },
            ).with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(200.0),
                left: Val::Px(50.0),
                ..default()
            }),
            PhaseText, // Mark as phase text
        ));
        
        commands.spawn((
            TextBundle::from_section(
                format!("Current Setup: {} players ({} AI)", 
                       config.player_count, config.ai_count),
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(0.8, 0.8, 0.8),
                    ..default()
                },
            ).with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(380.0),
                left: Val::Px(50.0),
                ..default()
            }),
            PhaseText, // Mark as phase text
        ));
    }
    
    // Player count selection
    if keyboard.just_pressed(KeyCode::Digit1) {
        config.player_count = 1;
        config.ai_count = config.ai_count.min(0);
        clear_menu_text(&mut commands, &text_query);
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        config.player_count = 2;
        config.ai_count = config.ai_count.min(1);
        clear_menu_text(&mut commands, &text_query);
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        config.player_count = 3;
        config.ai_count = config.ai_count.min(2);
        clear_menu_text(&mut commands, &text_query);
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        config.player_count = 4;
        config.ai_count = config.ai_count.min(3);
        clear_menu_text(&mut commands, &text_query);
    }
    
    // AI count adjustment
    if keyboard.just_pressed(KeyCode::KeyA) {
        config.ai_count = (config.ai_count + 1) % (config.player_count + 1);
        clear_menu_text(&mut commands, &text_query);
    }
    
    if keyboard.just_pressed(KeyCode::Space) {
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::Setup);
    }
}

fn clear_menu_text(commands: &mut Commands, text_query: &Query<Entity, With<PhaseText>>) {
    for entity in text_query.iter() {
        commands.entity(entity).despawn();
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

// Add a marker component for action board elements
#[derive(Component)]
pub struct ActionBoardElement;

fn setup_action_board(parent: &mut ChildBuilder) {
    parent.spawn((
        NodeBundle {
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
        },
        ActionBoardElement,
    )).with_children(|action_area| {
        // Summer Actions Header
        action_area.spawn(TextBundle::from_section(
            "SUMMER ACTIONS",
            TextStyle {
                font_size: 18.0,
                color: Color::from(Srgba::new(1.0, 1.0, 0.5, 1.0)),
                ..default()
            },
        ));
        
        // Summer actions
        let summer_actions = [
            ("Draw Vine", ActionSpace::DrawVine, false),
            ("Plant Vine (+1)", ActionSpace::PlantVine, true),
            ("Build Structure", ActionSpace::BuildStructure, false),
            ("Give Tour (+1)", ActionSpace::GiveTour, true),
            ("Sell Grapes", ActionSpace::SellGrapes, false),
            ("Train Worker", ActionSpace::TrainWorker, false),
        ];
        
        for (label, action, _has_bonus) in summer_actions {
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
                // Store text directly in the button - this should persist
                button.spawn(TextBundle {
                    text: Text::from_section(
                        label.to_string(),
                        TextStyle {
                            font_size: 16.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    ),
                    // Force the text to stay visible
                    visibility: Visibility::Inherited,
                    ..default()
                });
            });
        }
        
        // Winter Actions Header
        action_area.spawn(TextBundle::from_section(
            "WINTER ACTIONS",
            TextStyle {
                font_size: 18.0,
                color: Color::from(Srgba::new(0.5, 0.5, 1.0, 1.0)),
                ..default()
            },
        ));
        
        // Winter actions
        let winter_actions = [
            ("Draw Wine Order", ActionSpace::DrawWineOrder, false),
            ("Harvest (+1)", ActionSpace::Harvest, true),
            ("Make Wine (+1)", ActionSpace::MakeWine, true),
            ("Fill Order", ActionSpace::FillOrder, false),
        ];
        
        for (label, action, _has_bonus) in winter_actions {
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
                // Store text directly in the button - this should persist
                button.spawn(TextBundle {
                    text: Text::from_section(
                        label.to_string(),
                        TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    // Force the text to stay visible
                    visibility: Visibility::Inherited,
                    ..default()
                });
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

// Display player cards UI (Mama & Papa cards info)
pub fn display_player_cards_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mama_cards: Query<&MamaCard>,
    papa_cards: Query<&PapaCard>,
    players: Query<&Player>,
    existing_ui: Query<Entity, With<PlayerCardsUI>>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        if existing_ui.is_empty() {
            // Show player cards info panel
            let mut card_text = "🎴 PLAYER CARDS (Press C to close)\n\n".to_string();
            
            for player in players.iter() {
                card_text.push_str(&format!("🎯 Player {}: {}\n", player.id.0 + 1, player.name));
                
                // Show Mama card info
                if let Some(mama) = mama_cards.iter().find(|m| m.id == player.id.0) {
                    card_text.push_str(&format!("👵 Mama: {}\n", mama.name));
                    card_text.push_str(&format!("   Bonuses: +{} lira, +{} workers, +{} vine cards\n", 
                        mama.bonus_lira, mama.bonus_workers, mama.bonus_vine_cards));
                    
                    if let Some(ability) = &mama.special_ability {
                        let ability_desc = match ability {
                            MamaAbility::ExtraBonusAction => "Can take extra action per year",
                            MamaAbility::DiscountedStructures => "All structures cost 1 less lira",
                            MamaAbility::BonusHarvest => "+1 grape when harvesting",
                            MamaAbility::FreeVinePlanting => "Plant first vine each year for free",
                        };
                        card_text.push_str(&format!("   Special: {}\n", ability_desc));
                    }
                }
                
                // Show Papa card info
                if let Some(papa) = papa_cards.iter().find(|p| p.id == player.id.0) {
                    card_text.push_str(&format!("👴 Papa: {}\n", papa.name));
                    card_text.push_str(&format!("   Bonuses: +{} VP", papa.bonus_vp));
                    
                    if !papa.starting_structures.is_empty() {
                        let structures_text = papa.starting_structures.iter()
                            .map(|s| format!("{:?}", s))
                            .collect::<Vec<_>>()
                            .join(", ");
                        card_text.push_str(&format!(", structures: {}", structures_text));
                    }
                    
                    if papa.bonus_fields > 0 {
                        card_text.push_str(&format!(", +{} fields", papa.bonus_fields));
                    }
                    card_text.push_str("\n");
                    
                    if let Some(ability) = &papa.special_ability {
                        let ability_desc = match ability {
                            PapaAbility::ExtraVineyardField => "Start with extra vineyard field",
                            PapaAbility::AdvancedCellar => "Can store extra wine",
                            PapaAbility::TradingConnections => "Better wine order prices",
                            PapaAbility::WineExpertise => "Make blush wine more efficiently",
                        };
                        card_text.push_str(&format!("   Special: {}\n", ability_desc));
                    }
                }
                card_text.push_str("\n");
            }
            
            // Spawn the UI panel
            commands.spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(50.0),
                        left: Val::Px(20.0),
                        width: Val::Px(450.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    background_color: Color::srgb(0.1, 0.1, 0.1).with_alpha(0.95).into(),
                    z_index: ZIndex::Global(800),
                    ..default()
                },
                PlayerCardsUI,
            )).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    card_text,
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        } else {
            // Hide player cards panel
            for entity in existing_ui.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}