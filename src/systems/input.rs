use bevy::prelude::*;
use crate::components::*;
use crate::systems::game_logic::*;

const GREY: Srgba = Srgba::new(0.6, 0.6, 0.6, 1.0);

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
                
                if bounds.contains(world_pos) {
                    // Check if regular worker can be placed
                    let can_place_regular = action_space.can_place_worker(*current_player_id, current_state.get());
                    // Check if grande worker can be placed (can bypass restrictions)
                    let can_place_grande = action_space.can_place_grande_worker(*current_player_id, current_state.get());
                    
                    if can_place_regular || can_place_grande {
                        // Find available worker (prefer regular, then grande)
                        let mut selected_worker = None;
                        
                        if can_place_regular {
                            // Look for regular worker first
                            for (worker_entity, worker, _) in workers.iter() {
                                if worker.owner == *current_player_id && worker.placed_at.is_none() && !worker.is_grande {
                                    selected_worker = Some((worker_entity, false));
                                    break;
                                }
                            }
                        }
                        
                        // If no regular worker available but grande can be placed, use grande
                        if selected_worker.is_none() && can_place_grande {
                            for (worker_entity, worker, _) in workers.iter() {
                                if worker.owner == *current_player_id && worker.placed_at.is_none() && worker.is_grande {
                                    selected_worker = Some((worker_entity, true));
                                    break;
                                }
                            }
                        }
                        
                        if let Some((worker_entity, is_grande)) = selected_worker {
                            // Place the worker
                            for (w_entity, mut worker, _) in workers.iter_mut() {
                                if w_entity == worker_entity {
                                    worker.placed_at = Some(action_space.action);
                                    worker.position = action_space.position;
                                    
                                    // Handle space occupation
                                    if is_grande && action_space.occupied_by.is_some() {
                                        // Grande worker uses bonus slot
                                        action_space.bonus_worker_slot = Some(*current_player_id);
                                    } else {
                                        // Regular placement
                                        action_space.occupied_by = Some(*current_player_id);
                                    }
                                    
                                    execute_action(action_space.action, *current_player_id, &mut hands, &mut vineyards, &mut players, &mut card_decks, &mut commands);
                                    
                                    info!("Player {:?} placed {} worker on {:?}", 
                                          current_player_id, 
                                          if is_grande { "grande" } else { "regular" },
                                          action_space.action);
                                    break;
                                }
                            }
                            break;
                        }
                    }
                }
            }
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
                    ActionSpace::DrawVine | ActionSpace::PlantVine | ActionSpace::BuildStructure | 
                    ActionSpace::GiveTour | ActionSpace::SellGrapes | ActionSpace::TrainWorker);
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
                    ActionSpace::DrawVine | ActionSpace::PlantVine | ActionSpace::BuildStructure | 
                    ActionSpace::GiveTour | ActionSpace::SellGrapes | ActionSpace::TrainWorker);
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