use bevy::prelude::*;
use crate::components::*;

// Custom color constants
const YELLOW: Srgba = Srgba::new(1.0, 1.0, 0.0, 1.0);
const GOLD: Srgba = Srgba::new(1.0, 0.84, 0.0, 1.0);
const WHITE: Srgba = Srgba::new(1.0, 1.0, 1.0, 1.0);

pub fn main_menu_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    text_query: Query<Entity, With<Text>>,
) {
    // Spawn menu text if it doesn't exist
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
        // Remove menu text
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
    // Clean up all game entities when restarting
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
    // Remove any existing text
    for entity in text_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Create players and their workers
    for i in 0..config.player_count {
        let player = Player::new(i, format!("Player {}", i + 1));
        let vineyard = Vineyard::new(PlayerId(i));
        let hand = Hand::new(PlayerId(i));
        
        commands.spawn(player);
        commands.spawn(vineyard);
        commands.spawn(hand);
        
        // Create workers for each player
        for w in 0..2 {
            let worker_pos = Vec2::new(-500.0 + (i as f32 * 100.0), -200.0 + (w as f32 * 30.0));
            commands.spawn((
                Worker::new(PlayerId(i), false, worker_pos),
                Clickable { size: Vec2::new(20.0, 20.0) },
            ));
        }
        
        turn_order.players.push(PlayerId(i));
    }
    
    // Create action board with clickable spaces
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
    // Root UI container
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
        parent.spawn((
            NodeBundle {
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
            },
        )).with_children(|status_bar| {
            // Game status text
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
            
            // Turn indicator
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
                // Summer actions
                action_area.spawn(TextBundle::from_section(
                    "SUMMER ACTIONS",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::from(Srgba::new(1.0, 1.0, 0.5, 1.0)),
                        ..default()
                    },
                ));
                
                for action in [ActionSpace::DrawVine, ActionSpace::PlantVine, ActionSpace::GiveTour] {
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
                
                // Winter actions
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
                // Create dashboards for each player
                for i in 0..2 { // Default 2 players
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
                        // Player name
                        dashboard.spawn(TextBundle::from_section(
                            format!("Player {}", i + 1),
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                        
                        // Resources row
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
                        
                        // Vineyard status
                        dashboard.spawn(TextBundle::from_section(
                            "Grapes: R:0 W:0 | Wine: R:0 W:0",
                            TextStyle {
                                font_size: 14.0,
                                color: Color::from(Srgba::new(0.8, 0.8, 0.8, 1.0)),
                                ..default()
                            },
                        ));
                        
                        // Hand info
                        dashboard.spawn(TextBundle::from_section(
                            "Hand: Vines:0 Orders:0",
                            TextStyle {
                                font_size: 14.0,
                                color: Color::from(Srgba::new(0.6, 0.8, 0.6, 1.0)),
                                ..default()
                            },
                        ));
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
    text_query: Query<Entity, (With<Text>, Without<UIPanel>)>, // Only remove standalone text, not UI text
    ui_query: Query<Entity, With<UIPanel>>,
) {
    // Setup UI if it doesn't exist (first time entering Spring)
    if ui_query.is_empty() {
        setup_ui(&mut commands);
    }
    
    // Spawn spring text if it doesn't exist
    if text_query.is_empty() {
        commands.spawn(TextBundle::from_section(
            format!("SPRING PHASE - YEAR {}\nWorkers return home\n\nPress SPACE to continue to Summer", config.current_year),
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(300.0),
            left: Val::Px(400.0),
            // z_index: ZIndex::Global(1000),
            ..default()
        }));
    }
    
    // Reset workers and action spaces for new year
    if keyboard.just_pressed(KeyCode::Space) {
        // Remove spring text only
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Reset all workers
        for mut worker in workers.iter_mut() {
            worker.placed_at = None;
            // Reset worker positions to starting area
            let player_id = worker.owner.0;
            worker.position = Vec2::new(-500.0 + (player_id as f32 * 100.0), -200.0);
        }
        
        // Reset action spaces
        for mut space in action_spaces.iter_mut() {
            space.occupied_by = None;
            space.bonus_worker_slot = None;
        }
        
        turn_order.current_player = 0;
        next_state.set(GameState::Summer);
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
    turn_order: Res<TurnOrder>,
    current_state: Res<State<GameState>>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert screen coordinates to world coordinates  
        let world_pos = camera.viewport_to_world_2d(camera_transform, cursor_pos).unwrap_or(Vec2::ZERO);
        
        // Get current player
        if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
            // Check if clicking on an action space
            for (space_entity, mut action_space, clickable) in action_spaces.iter_mut() {
                let bounds = Rect::from_center_size(action_space.position, clickable.size);
                
                if bounds.contains(world_pos) && action_space.can_place_worker(*current_player_id, current_state.get()) {
                    // Find an available worker for the current player
                    for (worker_entity, mut worker, _) in workers.iter_mut() {
                        if worker.owner == *current_player_id && worker.placed_at.is_none() {
                            // Place the worker and execute action
                            worker.placed_at = Some(action_space.action);
                            worker.position = action_space.position;
                            action_space.occupied_by = Some(*current_player_id);
                            
                            // Execute the action
                            execute_action(action_space.action, *current_player_id, &mut hands, &mut vineyards, &mut players, &mut card_decks);
                            
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
) {
    // Find the player's hand, vineyard, and player data
    let mut hand = hands.iter_mut().find(|h| h.owner == player_id);
    let mut vineyard = vineyards.iter_mut().find(|v| v.owner == player_id);
    let mut player = players.iter_mut().find(|p| p.id == player_id);
    
    match action {
        ActionSpace::DrawVine => {
            if let (Some(mut hand), Some(card)) = (hand.as_mut(), card_decks.draw_vine_card()) {
                hand.vine_cards.push(card);
                info!("Player {:?} drew a vine card", player_id);
            }
        }
        ActionSpace::DrawWineOrder => {
            if let (Some(mut hand), Some(card)) = (hand.as_mut(), card_decks.draw_wine_order_card()) {
                hand.wine_order_cards.push(card);
                info!("Player {:?} drew a wine order card", player_id);
            }
        }
        ActionSpace::PlantVine => {
            if let (Some(mut hand), Some(mut vineyard)) = (hand.as_mut(), vineyard.as_mut()) {
                if !hand.vine_cards.is_empty() {
                    let vine_card = hand.vine_cards.remove(0);
                    // Find first empty field
                    for i in 0..9 {
                        if vineyard.can_plant_vine(i, &vine_card) {
                            if vineyard.plant_vine(i, vine_card.clone()) {
                                info!("Player {:?} planted vine: {:?} in field {}", player_id, vine_card.vine_type, i);
                                break;
                            }
                        }
                    }
                }
            }
        }
        ActionSpace::Harvest => {
            if let Some(mut vineyard) = vineyard.as_mut() {
                let old_red = vineyard.red_grapes;
                let old_white = vineyard.white_grapes;
                vineyard.harvest_grapes();
                info!("Player {:?} harvested grapes: +{} red, +{} white", 
                      player_id, vineyard.red_grapes - old_red, vineyard.white_grapes - old_white);
            }
        }
        ActionSpace::MakeWine => {
            if let Some(mut vineyard) = vineyard.as_mut() {
                // Simple wine making - convert 1 red grape and 1 white grape to wine if available
                let red_to_use = if vineyard.red_grapes > 0 { 1 } else { 0 };
                let white_to_use = if vineyard.white_grapes > 0 { 1 } else { 0 };
                
                if vineyard.make_wine(red_to_use, white_to_use) {
                    info!("Player {:?} made wine: {} red, {} white", player_id, red_to_use, white_to_use);
                }
            }
        }
        ActionSpace::FillOrder => {
            if let (Some(mut hand), Some(mut vineyard), Some(mut player)) = 
                (hand.as_mut(), vineyard.as_mut(), player.as_mut()) {
                if !hand.wine_order_cards.is_empty() {
                    let order = &hand.wine_order_cards[0];
                    if vineyard.can_fulfill_order(order) {
                        let order = hand.wine_order_cards.remove(0);
                        vineyard.fulfill_order(&order);
                        player.gain_victory_points(order.victory_points);
                        player.gain_lira(order.payout);
                        info!("Player {:?} filled order for {} VP and {} lira", 
                              player_id, order.victory_points, order.payout);
                    } else {
                        info!("Player {:?} cannot fulfill order - insufficient wine", player_id);
                    }
                }
            }
        }
        ActionSpace::GiveTour => {
            if let Some(mut player) = player.as_mut() {
                player.gain_lira(2);
                info!("Player {:?} gave tour for 2 lira", player_id);
            }
        }
        ActionSpace::SellGrapes => {
            if let (Some(mut vineyard), Some(mut player)) = (vineyard.as_mut(), player.as_mut()) {
                let grapes_sold = vineyard.red_grapes + vineyard.white_grapes;
                player.gain_lira(grapes_sold);
                vineyard.red_grapes = 0;
                vineyard.white_grapes = 0;
                info!("Player {:?} sold {} grapes for {} lira", player_id, grapes_sold, grapes_sold);
            }
        }
        ActionSpace::TrainWorker => {
            if let Some(mut player) = player.as_mut() {
                if player.lira >= 4 {
                    player.lira -= 4;
                    player.workers += 1;
                    info!("Player {:?} trained a worker for 4 lira", player_id);
                } else {
                    info!("Player {:?} cannot afford to train worker (need 4 lira)", player_id);
                }
            }
        }
        _ => {
            info!("Player {:?} executed action: {:?}", player_id, action);
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
    // Check if current player has placed all workers or pressed enter to pass
    if keyboard.just_pressed(KeyCode::Enter) {
        // Get current player
        if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
            // Check if player has available workers
            let available_workers = workers.iter()
                .filter(|w| w.owner == *current_player_id && w.placed_at.is_none())
                .count();
            
            if available_workers == 0 || keyboard.just_pressed(KeyCode::Enter) {
                // Advance to next player or next phase
                turn_order.current_player = (turn_order.current_player + 1) % players.iter().count();
                
                if turn_order.current_player == 0 {
                    // All players have taken their turn
                    match current_state.get() {
                        GameState::Summer => next_state.set(GameState::Fall),
                        GameState::Winter => {
                            // Advance to next year
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
    text_query: Query<Entity, (With<Text>, Without<UIPanel>)>, // Only standalone text
) {
    // Spawn fall text if it doesn't exist
    if text_query.is_empty() {
        commands.spawn(TextBundle::from_section(
            "FALL PHASE\nAutomatic harvest from planted vines\n\nPress SPACE to continue to Winter",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(300.0),
            left: Val::Px(400.0),
            // z_index: ZIndex::Global(1000),
            ..default()
        }));
    }
    
    // Automatic harvest from planted vines
    if keyboard.just_pressed(KeyCode::Space) {
        // Remove fall text
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        
        for mut vineyard in vineyards.iter_mut() {
            vineyard.harvest_grapes();
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
    // Check VP victory condition
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
    
    // Check year limit
    let year_limit_reached = config.current_year > config.max_years;
    
    if winner.is_some() || year_limit_reached {
        // Determine final winner
        if winner.is_none() && year_limit_reached {
            // Find player with most VP if year limit reached
            for player in players.iter() {
                if player.victory_points > highest_vp {
                    highest_vp = player.victory_points;
                    winner = Some(player);
                }
            }
        }
        
        // Remove any existing text and show victory screen
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

pub fn ui_button_system(
    mut interaction_query: Query<(&Interaction, &ActionButton, &mut BackgroundColor)>,
    mut workers: Query<&mut Worker>,
    mut action_spaces: Query<&mut ActionSpaceSlot>,
    mut hands: Query<&mut Hand>,
    mut vineyards: Query<&mut Vineyard>,
    mut players: Query<&mut Player>,
    mut card_decks: ResMut<CardDecks>,
    turn_order: Res<TurnOrder>,
    current_state: Res<State<GameState>>,
) {
    for (interaction, action_button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Check if it's the right season for this action
                let is_summer_action = matches!(action_button.action, 
                    ActionSpace::DrawVine | ActionSpace::PlantVine | ActionSpace::GiveTour);
                let is_valid_season = match current_state.get() {
                    GameState::Summer => is_summer_action,
                    GameState::Winter => !is_summer_action,
                    _ => false,
                };
                
                if !is_valid_season {
                    continue;
                }
                
                // Check if player has available workers
                if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
                    let has_available_worker = workers.iter()
                        .any(|w| w.owner == *current_player_id && w.placed_at.is_none());
                    
                    if has_available_worker {
                        // Find and place worker
                        for mut worker in workers.iter_mut() {
                            if worker.owner == *current_player_id && worker.placed_at.is_none() {
                                worker.placed_at = Some(action_button.action);
                                break;
                            }
                        }
                        
                        // Execute the action
                        execute_action(action_button.action, *current_player_id, &mut hands, &mut vineyards, &mut players, &mut card_decks);
                        
                        // Update action space
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
                // Reset to original color based on season
                let is_summer_action = matches!(action_button.action, 
                    ActionSpace::DrawVine | ActionSpace::PlantVine | ActionSpace::GiveTour);
                *color = if is_summer_action {
                    Color::from(Srgba::new(0.8, 0.8, 0.2, 0.8)).into()
                } else {
                    Color::from(Srgba::new(0.2, 0.2, 0.8, 0.8)).into()
                };
            }
        }
    }
}

pub fn update_ui_system(
    mut status_query: Query<&mut Text, (With<GameStatusText>, Without<TurnIndicator>)>,
    mut turn_query: Query<&mut Text, (With<TurnIndicator>, Without<GameStatusText>)>,
    dashboard_query: Query<&PlayerDashboard>,
    mut dashboard_text_query: Query<&mut Text, (Without<GameStatusText>, Without<TurnIndicator>)>,
    players: Query<&Player>,
    vineyards: Query<&Vineyard>,
    hands: Query<&Hand>,
    turn_order: Res<TurnOrder>,
    current_state: Res<State<GameState>>,
    config: Res<GameConfig>,
) {
    // Update game status
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
    
    // Update turn indicator
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
    
    // Update player dashboards
    for dashboard in dashboard_query.iter() {
        if let Some(player) = players.iter().find(|p| p.id == dashboard.player_id) {
            if let Some(vineyard) = vineyards.iter().find(|v| v.owner == dashboard.player_id) {
                if let Some(hand) = hands.iter().find(|h| h.owner == dashboard.player_id) {
                    // Find the dashboard's text components and update them
                    // This is simplified - in a real implementation you'd use more specific queries
                    // For now, we'll update via the existing text display system
                }
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
        // Remove UI panels
        for entity in ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        next_state.set(GameState::MainMenu);
    }
}