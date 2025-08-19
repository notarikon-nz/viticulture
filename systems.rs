use bevy::prelude::*;
use crate::components::*;

// Custom color constants
const YELLOW: Srgba = Srgba::new(1.0, 1.0, 0.0, 1.0);
const GOLD: Srgba = Srgba::new(1.0, 0.84, 0.0, 1.0);
const WHITE: Srgba = Srgba::new(1.0, 1.0, 1.0, 1.0);
const GREY: Srgba = Srgba::new(0.6, 0.6, 0.6, 1.0);

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = GameAssets {
        worker_texture: asset_server.load("worker.png"),
        vine_card_texture: asset_server.load("vine_card.png"),
        wine_order_card_texture: asset_server.load("wine_order.png"),
        field_texture: asset_server.load("field.png"),
    };
    commands.insert_resource(assets);
}

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

pub fn cleanup_entities_system(
    mut commands: Commands,
    entities: Query<Entity, (Without<Camera>, Without<Window>)>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn setup_game_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    config: Res<GameConfig>,
    mut turn_order: ResMut<TurnOrder>,
    text_query: Query<Entity, With<Text>>,
) {
    for entity in text_query.iter() {
        commands.entity(entity).despawn();
    }
    
    for i in 0..config.player_count {
        let player = Player::new(i, format!("Player {}", i + 1));
        let vineyard = Vineyard::new(PlayerId(i));
        let hand = Hand::new(PlayerId(i));
        
        commands.spawn(player);
        commands.spawn(vineyard);
        commands.spawn(hand);
        
        for w in 0..2 {
            let worker_pos = Vec2::new(-500.0 + (i as f32 * 100.0), -200.0 + (w as f32 * 30.0));
            commands.spawn((
                Worker::new(PlayerId(i), false, worker_pos),
                Clickable { size: Vec2::new(20.0, 20.0) },
            ));
        }
        
        turn_order.players.push(PlayerId(i));
    }
    
    let action_board = ActionBoard::new();
    for space in action_board.spaces.clone() {
        commands.spawn((
            space,
            Clickable { size: Vec2::new(60.0, 30.0) },
        ));
    }
    commands.spawn(action_board);
    
    next_state.set(GameState::Spring);
}

fn setup_ui(commands: &mut Commands) {
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
            // Left side - Action board
            main_area.spawn(NodeBundle {
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
                
                for action in [ActionSpace::DrawVine, ActionSpace::PlantVine, ActionSpace::BuildStructure, ActionSpace::GiveTour] {
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
                            format!("{:?}", action),
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
                
                for action in [ActionSpace::DrawWineOrder, ActionSpace::Harvest, ActionSpace::MakeWine, ActionSpace::FillOrder] {
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
                            format!("{:?}", action),
                            TextStyle {
                                font_size: 16.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
                }
            });
            
            // Right side - Player dashboards
            main_area.spawn(NodeBundle {
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
                                    color: Color::from(Srgba::new(1.0, 1.0, 0.0, 1.0)),
                                    ..default()
                                },
                            ));
                            resources.spawn(TextBundle::from_section(
                                "Lira: 3",
                                TextStyle {
                                    font_size: 16.0,
                                    color: Color::from(Srgba::new(1.0, 0.84, 0.0, 1.0)),
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
                            "Hand: Vines:0 Orders:0",
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
        });
    });
}

pub fn spring_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut turn_order: ResMut<TurnOrder>,
    mut workers: Query<&mut Worker>,
    mut action_spaces: Query<&mut ActionSpaceSlot>,
    mut config: ResMut<GameConfig>,
    mut commands: Commands,
    text_query: Query<Entity, (With<Text>, Without<UIPanel>)>,
    ui_query: Query<Entity, With<UIPanel>>,
    mut hands: Query<&mut Hand>,
    mut players: Query<&mut Player>,
    mut card_decks: ResMut<CardDecks>,
) {
    if ui_query.is_empty() {
        setup_ui(&mut commands);
    }
    
    if text_query.is_empty() {
        commands.spawn(Text2dBundle {
            text: Text::from_section(
                format!("SPRING PHASE - YEAR {}\nChoose wake-up times (1-7)\nPress SPACE to auto-assign and continue", config.current_year),
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 200.0, 1000.0)),
            ..default()
        });
    }
    
    if keyboard.just_pressed(KeyCode::Space) {
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        
        let mut wake_up_assignments = Vec::new();
        for (i, player_id) in turn_order.players.iter().enumerate() {
            wake_up_assignments.push((*player_id, (i + 1) as u8));
        }
        turn_order.set_wake_up_order(wake_up_assignments);
        
        for (player_id, _) in &turn_order.wake_up_order {
            if let Some(bonus) = turn_order.get_wake_up_bonus(*player_id) {
                apply_wake_up_bonus(*player_id, bonus, &mut hands, &mut players, &mut card_decks, &mut commands);
            }
        }
        
        for mut worker in workers.iter_mut() {
            worker.placed_at = None;
            let player_id = worker.owner.0;
            worker.position = Vec2::new(-500.0 + (player_id as f32 * 100.0), -200.0);
        }
        
        for mut space in action_spaces.iter_mut() {
            space.occupied_by = None;
            space.bonus_worker_slot = None;
        }
        
        turn_order.current_player = 0;
        next_state.set(GameState::Summer);
    }
}

fn apply_wake_up_bonus(
    player_id: PlayerId,
    bonus: WakeUpBonus,
    hands: &mut Query<&mut Hand>,
    players: &mut Query<&mut Player>,
    card_decks: &mut ResMut<CardDecks>,
    commands: &mut Commands,
) {
    match bonus {
        WakeUpBonus::DrawVineCard => {
            if let Some(mut hand) = hands.iter_mut().find(|h| h.owner == player_id) {
                if let Some(card) = card_decks.draw_vine_card() {
                    hand.vine_cards.push(card);
                    spawn_animated_text(commands, player_id, "Wake-up: +Vine", Color::from(Srgba::new(0.2, 0.8, 0.2, 1.0)));
                }
            }
        }
        WakeUpBonus::GainLira(amount) => {
            if let Some(mut player) = players.iter_mut().find(|p| p.id == player_id) {
                player.gain_lira(amount);
                spawn_animated_text(commands, player_id, &format!("Wake-up: +{} Lira", amount), Color::from(GOLD));
            }
        }
        WakeUpBonus::GainVictoryPoint => {
            if let Some(mut player) = players.iter_mut().find(|p| p.id == player_id) {
                player.gain_victory_points(1);
                spawn_animated_text(commands, player_id, "Wake-up: +1 VP", Color::from(YELLOW));
            }
        }
        WakeUpBonus::DrawWineOrderCard => {
            if let Some(mut hand) = hands.iter_mut().find(|h| h.owner == player_id) {
                if let Some(card) = card_decks.draw_wine_order_card() {
                    hand.wine_order_cards.push(card);
                    spawn_animated_text(commands, player_id, "Wake-up: +Order", Color::from(Srgba::new(0.6, 0.2, 0.8, 1.0)));
                }
            }
        }
        _ => {}
    }
}

pub fn mouse_input_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut workers: Query<(Entity, &mut Worker, &Clickable)>,
    mut action_spaces: Query<(Entity, &mut ActionSpaceSlot, &Clickable), Without<Worker>>,
    mut hands: Query<&mut Hand>,
    mut vineyards: Query<&mut Vineyard>,
    mut players: Query<&mut Player>,
    mut card_decks: ResMut<CardDecks>,
    mut commands: Commands,
    turn_order: Res<TurnOrder>,
    current_state: Res<State<GameState>>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    
    if let Some(cursor_pos) = window.cursor_position() {
        let world_pos = camera.viewport_to_world_2d(camera_transform, cursor_pos).unwrap_or(Vec2::ZERO);
        
        if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
            for (space_entity, mut action_space, clickable) in action_spaces.iter_mut() {
                let bounds = Rect::from_center_size(action_space.position, clickable.size);
                
                if bounds.contains(world_pos) && action_space.can_place_worker(*current_player_id, current_state.get()) {
                    for (worker_entity, mut worker, _) in workers.iter_mut() {
                        if worker.owner == *current_player_id && worker.placed_at.is_none() {
                            worker.placed_at = Some(action_space.action);
                            worker.position = action_space.position;
                            action_space.occupied_by = Some(*current_player_id);
                            
                            execute_action(action_space.action, *current_player_id, &mut hands, &mut vineyards, &mut players, &mut card_decks, &mut commands);
                            
                            info!("Player {:?} placed worker on {:?}", current_player_id, action_space.action);
                            break;
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn execute_action(
    action: ActionSpace,
    player_id: PlayerId,
    hands: &mut Query<&mut Hand>,
    vineyards: &mut Query<&mut Vineyard>,
    players: &mut Query<&mut Player>,
    card_decks: &mut ResMut<CardDecks>,
    commands: &mut Commands,
) {
    let mut hand = hands.iter_mut().find(|h| h.owner == player_id);
    let mut vineyard = vineyards.iter_mut().find(|v| v.owner == player_id);
    let mut player = players.iter_mut().find(|p| p.id == player_id);
    
    match action {
        ActionSpace::DrawVine => {
            if let (Some(mut hand), Some(card)) = (hand.as_mut(), card_decks.draw_vine_card()) {
                hand.vine_cards.push(card);
                spawn_animated_text(commands, player_id, "+Vine", Color::from(Srgba::new(0.2, 0.8, 0.2, 1.0)));
            }
        }
        ActionSpace::DrawWineOrder => {
            if let (Some(mut hand), Some(card)) = (hand.as_mut(), card_decks.draw_wine_order_card()) {
                hand.wine_order_cards.push(card);
                spawn_animated_text(commands, player_id, "+Order", Color::from(Srgba::new(0.6, 0.2, 0.8, 1.0)));
            }
        }
        ActionSpace::PlantVine => {
            if let (Some(mut hand), Some(mut vineyard)) = (hand.as_mut(), vineyard.as_mut()) {
                if !hand.vine_cards.is_empty() {
                    let vine_card = hand.vine_cards.remove(0);
                    let structures = Vec::new();
                    for i in 0..9 {
                        if vineyard.can_plant_vine(i, &vine_card, &structures) {
                            if vineyard.plant_vine(i, vine_card.clone(), &structures) {
                                spawn_animated_text(commands, player_id, "Planted!", Color::from(Srgba::new(0.4, 0.8, 0.4, 1.0)));
                                break;
                            }
                        }
                    }
                }
            }
        }
        ActionSpace::BuildStructure => {
            if let Some(mut vineyard) = vineyard.as_mut() {
                if vineyard.can_build_structure(StructureType::Trellis) {
                    if vineyard.build_structure(StructureType::Trellis) {
                        spawn_animated_text(commands, player_id, "+Structure", Color::from(Srgba::new(0.8, 0.8, 0.2, 1.0)));
                    }
                }
            }
        }
        ActionSpace::Harvest => {
            if let Some(mut vineyard) = vineyard.as_mut() {
                let structures = Vec::new();
                let gained = vineyard.harvest_grapes(&structures);
                if gained > 0 {
                    spawn_animated_text(commands, player_id, &format!("+{} Grapes", gained), Color::from(Srgba::new(0.8, 0.4, 0.8, 1.0)));
                }
            }
        }
        ActionSpace::MakeWine => {
            if let Some(mut vineyard) = vineyard.as_mut() {
                let red_to_use = if vineyard.red_grapes > 0 { 1 } else { 0 };
                let white_to_use = if vineyard.white_grapes > 0 { 1 } else { 0 };
                
                if vineyard.make_wine(red_to_use, white_to_use) {
                    let total_wine = red_to_use + white_to_use;
                    if total_wine > 0 {
                        spawn_animated_text(commands, player_id, &format!("+{} Wine", total_wine), Color::from(Srgba::new(0.7, 0.2, 0.2, 1.0)));
                    }
                }
            }
        }
        ActionSpace::FillOrder => {
            if let (Some(mut hand), Some(mut vineyard), Some(mut player)) = (hand.as_mut(), vineyard.as_mut(), player.as_mut()) {
                if !hand.wine_order_cards.is_empty() {
                    let order = &hand.wine_order_cards[0];
                    if vineyard.can_fulfill_order(order) {
                        let order = hand.wine_order_cards.remove(0);
                        vineyard.fulfill_order(&order);
                        player.gain_victory_points(order.victory_points);
                        player.gain_lira(order.payout);
                        spawn_animated_text(commands, player_id, &format!("+{} VP", order.victory_points), Color::from(YELLOW));
                        if order.payout > 0 {
                            spawn_animated_text(commands, player_id, &format!("+{} Lira", order.payout), Color::from(GOLD));
                        }
                    }
                }
            }
        }
        ActionSpace::GiveTour => {
            if let Some(mut player) = player.as_mut() {
                player.gain_lira(2);
                spawn_animated_text(commands, player_id, "+2 Lira", Color::from(GOLD));
            }
        }
        ActionSpace::SellGrapes => {
            if let (Some(mut vineyard), Some(mut player)) = (vineyard.as_mut(), player.as_mut()) {
                let grapes_sold = vineyard.red_grapes + vineyard.white_grapes;
                if grapes_sold > 0 {
                    player.gain_lira(grapes_sold);
                    vineyard.red_grapes = 0;
                    vineyard.white_grapes = 0;
                    spawn_animated_text(commands, player_id, &format!("+{} Lira", grapes_sold), Color::from(GOLD));
                }
            }
        }
        ActionSpace::TrainWorker => {
            if let Some(mut player) = player.as_mut() {
                if player.lira >= 4 {
                    player.lira -= 4;
                    player.workers += 1;
                    spawn_animated_text(commands, player_id, "+Worker", Color::from(Srgba::new(0.5, 0.8, 1.0, 1.0)));
                }
            }
        }
        _ => {}
    }
}

fn spawn_animated_text(commands: &mut Commands, player_id: PlayerId, text: &str, color: Color) {
    let start_pos = Vec2::new(-400.0 + (player_id.0 as f32 * 200.0), 200.0);
    let end_pos = Vec2::new(start_pos.x, start_pos.y + 50.0);
    
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font_size: 20.0,
                    color,
                    ..default()
                },
            ),
            transform: Transform::from_translation(start_pos.extend(10.0)),
            ..default()
        },
        AnimatedText {
            timer: Timer::from_seconds(1.5, TimerMode::Once),
            start_pos,
            end_pos,
        },
    ));
}

pub fn animate_text_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animated_texts: Query<(Entity, &mut Transform, &mut AnimatedText, &mut Text)>,
) {
    for (entity, mut transform, mut animated_text, mut text) in animated_texts.iter_mut() {
        animated_text.timer.tick(time.delta());
        
        let progress = animated_text.timer.elapsed_secs() / animated_text.timer.duration().as_secs_f32();
        let current_pos = animated_text.start_pos.lerp(animated_text.end_pos, progress);
        transform.translation = current_pos.extend(10.0);
        
        let alpha = (1.0 - progress).max(0.0);
        for section in text.sections.iter_mut() {
            let mut color = section.style.color;
            match &mut color {
                Color::Srgba(srgba) => srgba.alpha = alpha,
                Color::LinearRgba(linear) => linear.alpha = alpha,
                _ => {}
            }
            section.style.color = color;
        }
        
        if animated_text.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn ui_button_system(
    mut interaction_query: Query<(&Interaction, &ActionButton, &mut BackgroundColor)>,
    mut workers: Query<&mut Worker>,
    mut action_spaces: Query<&mut ActionSpaceSlot>,
    mut hands: Query<&mut Hand>,
    mut vineyards: Query<&mut Vineyard>,
    mut players: Query<&mut Player>,
    mut card_decks: ResMut<CardDecks>,
    mut commands: Commands,
    turn_order: Res<TurnOrder>,
    current_state: Res<State<GameState>>,
) {
    for (interaction, action_button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let is_summer_action = matches!(action_button.action, 
                    ActionSpace::DrawVine | ActionSpace::PlantVine | ActionSpace::BuildStructure | ActionSpace::GiveTour);
                let is_valid_season = match current_state.get() {
                    GameState::Summer => is_summer_action,
                    GameState::Winter => !is_summer_action,
                    _ => false,
                };
                
                if !is_valid_season {
                    continue;
                }
                
                if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
                    let has_available_worker = workers.iter()
                        .any(|w| w.owner == *current_player_id && w.placed_at.is_none());
                    
                    if has_available_worker {
                        for mut worker in workers.iter_mut() {
                            if worker.owner == *current_player_id && worker.placed_at.is_none() {
                                worker.placed_at = Some(action_button.action);
                                break;
                            }
                        }
                        
                        execute_action(action_button.action, *current_player_id, &mut hands, &mut vineyards, &mut players, &mut card_decks, &mut commands);
                        
                        for mut space in action_spaces.iter_mut() {
                            if space.action == action_button.action {
                                space.occupied_by = Some(*current_player_id);
                                break;
                            }
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *color = Color::from(Srgba::new(0.9, 0.9, 0.9, 1.0)).into();
            }
            Interaction::None => {
                let is_summer_action = matches!(action_button.action, 
                    ActionSpace::DrawVine | ActionSpace::PlantVine | ActionSpace::BuildStructure | ActionSpace::GiveTour);
                *color = if is_summer_action {
                    Color::from(Srgba::new(0.8, 0.8, 0.2, 0.8)).into()
                } else {
                    Color::from(Srgba::new(0.2, 0.2, 0.8, 0.8)).into()
                };
            }
        }
    }
}

pub fn worker_placement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut turn_order: ResMut<TurnOrder>,
    mut config: ResMut<GameConfig>,
    players: Query<&Player>,
    workers: Query<&Worker>,
    current_state: Res<State<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
            let available_workers = workers.iter()
                .filter(|w| w.owner == *current_player_id && w.placed_at.is_none())
                .count();
            
            if available_workers == 0 || keyboard.just_pressed(KeyCode::Enter) {
                turn_order.current_player = (turn_order.current_player + 1) % players.iter().count();
                
                if turn_order.current_player == 0 {
                    match current_state.get() {
                        GameState::Summer => next_state.set(GameState::Fall),
                        GameState::Winter => {
                            config.current_year += 1;
                            next_state.set(GameState::Spring);
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn fall_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut vineyards: Query<&mut Vineyard>,
    mut commands: Commands,
    text_query: Query<Entity, (With<Text>, Without<UIPanel>)>,
) {
    if text_query.is_empty() {
        commands.spawn(Text2dBundle {
            text: Text::from_section(
                "FALL PHASE\nAutomatic harvest from planted vines\n\nPress SPACE to continue to Winter",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 200.0, 1000.0)),
            ..default()
        });
    }
    
    if keyboard.just_pressed(KeyCode::Space) {
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        
        let structures = Vec::new();
        for mut vineyard in vineyards.iter_mut() {
            vineyard.harvest_grapes(&structures);
        }
        next_state.set(GameState::Winter);
    }
}

pub fn check_victory_system(
    players: Query<&Player>,
    mut next_state: ResMut<NextState<GameState>>,
    config: Res<GameConfig>,
    mut commands: Commands,
    text_query: Query<Entity, With<Text>>,
) {
    let mut winner: Option<&Player> = None;
    let mut highest_vp = 0;
    
    for player in players.iter() {
        if player.victory_points >= config.target_victory_points {
            if player.victory_points > highest_vp {
                highest_vp = player.victory_points;
                winner = Some(player);
            }
        }
    }
    
    let year_limit_reached = config.current_year > config.max_years;
    
    if winner.is_some() || year_limit_reached {
        if winner.is_none() && year_limit_reached {
            for player in players.iter() {
                if player.victory_points > highest_vp {
                    highest_vp = player.victory_points;
                    winner = Some(player);
                }
            }
        }
        
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        
        if let Some(winning_player) = winner {
            commands.spawn(TextBundle::from_section(
                format!("GAME OVER!\n{} WINS with {} Victory Points!\n\nPress SPACE to play again", 
                        winning_player.name, winning_player.victory_points),
                TextStyle {
                    font_size: 32.0,
                    color: Color::from(GOLD),
                    ..default()
                },
            ).with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(200.0),
                left: Val::Px(100.0),
                ..default()
            }));
        }
        
        next_state.set(GameState::GameOver);
    }
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

pub fn update_sprites_system(
    mut commands: Commands,
    workers: Query<&Worker>,
    vineyards: Query<&Vineyard>,
    hands: Query<&Hand>,
    worker_sprites: Query<Entity, With<WorkerSprite>>,
    vineyard_sprites: Query<Entity, With<VineyardSprite>>,
    card_sprites: Query<Entity, With<CardSprite>>,
    turn_order: Res<TurnOrder>,
) {
    for entity in worker_sprites.iter() {
        commands.entity(entity).despawn();
    }
    for entity in vineyard_sprites.iter() {
        commands.entity(entity).despawn();
    }
    for entity in card_sprites.iter() {
        commands.entity(entity).despawn();
    }
    
    for worker in workers.iter() {
        let player_colors = [
            Color::from(Srgba::RED),
            Color::from(Srgba::BLUE),
            Color::from(Srgba::GREEN),
            Color::from(Srgba::new(1.0, 0.0, 1.0, 1.0)),
        ];
        
        let color_grey = Color::from(GREY);
        let color = player_colors.get(worker.owner.0 as usize).unwrap_or(&color_grey);
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: *color,
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..default()
                },
                transform: Transform::from_translation(worker.position.extend(1.0)),
                ..default()
            },
            WorkerSprite { player_id: worker.owner },
        ));
    }
    
    for vineyard in vineyards.iter() {
        for (field_idx, field) in vineyard.fields.iter().enumerate() {
            let field_x = -200.0 + ((field_idx % 3) as f32 * 40.0);
            let field_y = 100.0 - ((field_idx / 3) as f32 * 40.0);
            let field_pos = Vec2::new(field_x + (vineyard.owner.0 as f32 * 200.0), field_y);
            
            let field_color = match field {
                Some(VineType::Red(_)) => Color::from(Srgba::new(0.8, 0.2, 0.2, 1.0)),
                Some(VineType::White(_)) => Color::from(Srgba::new(0.9, 0.9, 0.7, 1.0)),
                None => Color::from(Srgba::new(0.4, 0.3, 0.2, 0.8)),
            };
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: field_color,
                        custom_size: Some(Vec2::new(35.0, 35.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(field_pos.extend(0.5)),
                    ..default()
                },
                VineyardSprite { 
                    player_id: vineyard.owner,
                    field_index: field_idx,
                },
            ));
        }
    }
    
    if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
        if let Some(hand) = hands.iter().find(|h| h.owner == *current_player_id) {
            let hand_y = -200.0;
            let mut card_x = -300.0;
            
            for (i, _vine_card) in hand.vine_cards.iter().enumerate() {
                let card_pos = Vec2::new(card_x + (i as f32 * 35.0), hand_y);
                
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::from(Srgba::new(0.2, 0.8, 0.2, 0.9)),
                            custom_size: Some(Vec2::new(30.0, 40.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(card_pos.extend(2.0)),
                        ..default()
                    },
                    CardSprite { card_type: CardType::Vine },
                ));
            }
            
            card_x += hand.vine_cards.len() as f32 * 35.0 + 20.0;
            
            for (i, _order_card) in hand.wine_order_cards.iter().enumerate() {
                let card_pos = Vec2::new(card_x + (i as f32 * 35.0), hand_y);
                
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::from(Srgba::new(0.6, 0.2, 0.8, 0.9)),
                            custom_size: Some(Vec2::new(30.0, 40.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(card_pos.extend(2.0)),
                        ..default()
                    },
                    CardSprite { card_type: CardType::WineOrder },
                ));
            }
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