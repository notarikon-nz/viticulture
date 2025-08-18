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

pub fn spring_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut turn_order: ResMut<TurnOrder>,
    mut workers: Query<&mut Worker>,
    mut action_spaces: Query<&mut ActionSpaceSlot>,
    mut commands: Commands,
    text_query: Query<Entity, With<Text>>,
) {
    // Spawn spring text if it doesn't exist
    if text_query.is_empty() {
        commands.spawn(TextBundle::from_section(
            "SPRING PHASE\nWorkers return home\n\nPress SPACE to continue to Summer",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(50.0),
            ..default()
        }));
    }
    
    // Reset workers and action spaces for new year
    if keyboard.just_pressed(KeyCode::Space) {
        // Remove spring text
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
                            // Reset workers for next year
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
    text_query: Query<Entity, With<Text>>,
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
            top: Val::Px(50.0),
            left: Val::Px(50.0),
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
) {
    for player in players.iter() {
        if player.victory_points >= config.target_victory_points {
            next_state.set(GameState::GameOver);
            break;
        }
    }
}

pub fn ui_system(
    mut gizmos: Gizmos,
    players: Query<&Player>,
    vineyards: Query<&Vineyard>,
    workers: Query<&Worker>,
    action_spaces: Query<&ActionSpaceSlot>,
    hands: Query<&Hand>,
    turn_order: Res<TurnOrder>,
    current_state: Res<State<GameState>>,
    mut commands: Commands,
    text_query: Query<Entity, With<Text>>,
) {
    // Add game state text for Summer/Winter phases
    match current_state.get() {
        GameState::Summer => {
            if text_query.is_empty() {
                if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
                    commands.spawn(TextBundle::from_section(
                        format!("SUMMER PHASE - Player {}'s Turn\nClick green action spaces (left side) to place workers\nPress ENTER to pass turn", current_player_id.0 + 1),
                        TextStyle {
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ).with_style(Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(50.0),
                        left: Val::Px(50.0),
                        ..default()
                    }));
                }
            }
        }
        GameState::Winter => {
            if text_query.is_empty() {
                if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
                    commands.spawn(TextBundle::from_section(
                        format!("WINTER PHASE - Player {}'s Turn\nClick blue action spaces (right side) to place workers\nPress ENTER to pass turn", current_player_id.0 + 1),
                        TextStyle {
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ).with_style(Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(50.0),
                        left: Val::Px(50.0),
                        ..default()
                    }));
                }
            }
        }
        GameState::GameOver => {
            if text_query.is_empty() {
                commands.spawn(TextBundle::from_section(
                    "GAME OVER!\nSomeone reached 20 Victory Points!",
                    TextStyle {
                        font_size: 32.0,
                        color: Color::from(GOLD),
                        ..default()
                    },
                ).with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(200.0),
                    left: Val::Px(200.0),
                    ..default()
                }));
            }
        }
        _ => {}
    }
    
    // Draw action spaces
    for space in action_spaces.iter() {
        let color = if space.is_summer {
            Color::from(Srgba::new(1.0, 1.0, 0.5, 0.8)) // Light yellow for summer
        } else {
            Color::from(Srgba::new(0.5, 0.5, 1.0, 0.8)) // Light blue for winter
        };
        
        // Highlight available spaces for current season
        let available = match current_state.get() {
            GameState::Summer => space.is_summer && space.occupied_by.is_none(),
            GameState::Winter => !space.is_summer && space.occupied_by.is_none(),
            _ => false,
        };
        
        let final_color = if available {
            Color::from(Srgba::new(0.0, 1.0, 0.0, 0.6)) // Green for available
        } else if space.occupied_by.is_some() {
            Color::from(Srgba::new(1.0, 0.0, 0.0, 0.6)) // Red for occupied
        } else {
            color
        };
        
        gizmos.rect_2d(space.position, 0.0, Vec2::new(60.0, 30.0), final_color);
    }
    
    // Draw workers
    for worker in workers.iter() {
        let player_colors = [
            Srgba::new(1.0, 0.0, 0.0, 1.0), // Red
            Srgba::new(0.0, 0.0, 1.0, 1.0), // Blue
            Srgba::new(0.0, 1.0, 0.0, 1.0), // Green
            Srgba::new(1.0, 0.0, 1.0, 1.0), // Magenta
        ];
        
        let default_color = Srgba::new(0.5, 0.5, 0.5, 1.0);
        let color = player_colors.get(worker.owner.0 as usize)
            .unwrap_or(&default_color);
        
        let size = if worker.is_grande { 15.0 } else { 10.0 };
        gizmos.circle_2d(worker.position, size, Color::from(*color));
    }
    
    // Draw game state info
    let mut y_offset = 300.0;
    
    match current_state.get() {
        GameState::Summer => {
            if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
                gizmos.circle_2d(Vec2::new(-400.0, y_offset), 10.0, Color::from(Srgba::GREEN));
            }
        }
        GameState::Winter => {
            if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
                gizmos.circle_2d(Vec2::new(-400.0, y_offset - 50.0), 10.0, Color::from(Srgba::BLUE));
            }
        }
        GameState::GameOver => {
            gizmos.circle_2d(Vec2::new(0.0, 0.0), 50.0, Color::from(GOLD));
        }
        _ => {}
    }
    
    // Draw player info
    for (i, player) in players.iter().enumerate() {
        let x_pos = -400.0 + (i as f32 * 200.0);
        
        // Player indicator
        gizmos.circle_2d(Vec2::new(x_pos, y_offset), 5.0, Color::from(WHITE));
        
        // Victory points indicator (simplified - number of small circles)
        for vp in 0..player.victory_points {
            gizmos.circle_2d(
                Vec2::new(x_pos + (vp as f32 * 10.0), y_offset - 30.0),
                2.0,
                Color::from(YELLOW),
            );
        }
        
        // Lira indicator (small rectangles)
        for l in 0..player.lira.min(10) {
            gizmos.rect_2d(
                Vec2::new(x_pos + (l as f32 * 6.0), y_offset - 45.0),
                0.0,
                Vec2::new(4.0, 8.0),
                Color::from(GOLD),
            );
        }
    }
    
    // Draw vineyard info
    for (i, vineyard) in vineyards.iter().enumerate() {
        let x_pos = -400.0 + (i as f32 * 200.0);
        
        // Draw vineyard fields (3x3 grid)
        for field_idx in 0..9 {
            let field_x = x_pos + ((field_idx % 3) as f32 * 15.0) - 15.0;
            let field_y = y_offset - 100.0 - ((field_idx / 3) as f32 * 15.0);
            
            let field_color = match vineyard.fields[field_idx] {
                Some(VineType::Red(_)) => Color::from(Srgba::new(0.8, 0.2, 0.2, 0.8)),
                Some(VineType::White(_)) => Color::from(Srgba::new(0.9, 0.9, 0.7, 0.8)),
                None => Color::from(Srgba::new(0.3, 0.2, 0.1, 0.5)), // Empty field
            };
            
            gizmos.rect_2d(Vec2::new(field_x, field_y), 0.0, Vec2::new(12.0, 12.0), field_color);
        }
        
        // Red grapes
        for grape in 0..vineyard.red_grapes {
            gizmos.circle_2d(
                Vec2::new(x_pos + (grape as f32 * 8.0), y_offset - 60.0),
                3.0,
                Color::from(Srgba::RED),
            );
        }
        
        // White grapes  
        for grape in 0..vineyard.white_grapes {
            gizmos.circle_2d(
                Vec2::new(x_pos + (grape as f32 * 8.0), y_offset - 75.0),
                3.0,
                Color::from(WHITE),
            );
        }
        
        // Red wine (diamonds)
        for wine in 0..vineyard.red_wine {
            let wine_pos = Vec2::new(x_pos + (wine as f32 * 8.0), y_offset - 170.0);
            gizmos.line_2d(wine_pos + Vec2::new(0.0, 4.0), wine_pos + Vec2::new(4.0, 0.0), Color::from(Srgba::new(0.6, 0.0, 0.0, 1.0)));
            gizmos.line_2d(wine_pos + Vec2::new(4.0, 0.0), wine_pos + Vec2::new(0.0, -4.0), Color::from(Srgba::new(0.6, 0.0, 0.0, 1.0)));
            gizmos.line_2d(wine_pos + Vec2::new(0.0, -4.0), wine_pos + Vec2::new(-4.0, 0.0), Color::from(Srgba::new(0.6, 0.0, 0.0, 1.0)));
            gizmos.line_2d(wine_pos + Vec2::new(-4.0, 0.0), wine_pos + Vec2::new(0.0, 4.0), Color::from(Srgba::new(0.6, 0.0, 0.0, 1.0)));
        }
        
        // White wine (diamonds)
        for wine in 0..vineyard.white_wine {
            let wine_pos = Vec2::new(x_pos + (wine as f32 * 8.0), y_offset - 185.0);
            gizmos.line_2d(wine_pos + Vec2::new(0.0, 4.0), wine_pos + Vec2::new(4.0, 0.0), Color::from(Srgba::new(0.9, 0.9, 0.7, 1.0)));
            gizmos.line_2d(wine_pos + Vec2::new(4.0, 0.0), wine_pos + Vec2::new(0.0, -4.0), Color::from(Srgba::new(0.9, 0.9, 0.7, 1.0)));
            gizmos.line_2d(wine_pos + Vec2::new(0.0, -4.0), wine_pos + Vec2::new(-4.0, 0.0), Color::from(Srgba::new(0.9, 0.9, 0.7, 1.0)));
            gizmos.line_2d(wine_pos + Vec2::new(-4.0, 0.0), wine_pos + Vec2::new(0.0, 4.0), Color::from(Srgba::new(0.9, 0.9, 0.7, 1.0)));
        }
    }
    
    // Draw hand visualization for current player
    if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
        let hand_y = -250.0;
        let mut card_x = -400.0;
        
        // Find current player's hand
        for hand in hands.iter() {
            if hand.owner == *current_player_id {
                // Draw vine cards (green)
                for (i, vine_card) in hand.vine_cards.iter().enumerate() {
                    let card_pos = Vec2::new(card_x + (i as f32 * 40.0), hand_y);
                    gizmos.rect_2d(card_pos, 0.0, Vec2::new(30.0, 40.0), Color::from(Srgba::new(0.0, 0.8, 0.0, 0.8)));
                    
                    // Show vine type with small circle
                    let vine_color = match vine_card.vine_type {
                        VineType::Red(_) => Srgba::RED,
                        VineType::White(_) => WHITE,
                    };
                    gizmos.circle_2d(card_pos + Vec2::new(0.0, 10.0), 3.0, Color::from(vine_color));
                }
                
                card_x += hand.vine_cards.len() as f32 * 40.0 + 20.0;
                
                // Draw wine order cards (purple)
                for (i, order_card) in hand.wine_order_cards.iter().enumerate() {
                    let card_pos = Vec2::new(card_x + (i as f32 * 40.0), hand_y);
                    gizmos.rect_2d(card_pos, 0.0, Vec2::new(30.0, 40.0), Color::from(Srgba::new(0.5, 0.0, 0.5, 0.8)));
                    
                    // Show victory points as small circles
                    for vp in 0..order_card.victory_points.min(3) {
                        gizmos.circle_2d(
                            card_pos + Vec2::new(-10.0 + (vp as f32 * 7.0), 10.0),
                            2.0,
                            Color::from(YELLOW),
                        );
                    }
                }
                break;
            }
        }
    }
}