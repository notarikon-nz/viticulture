use bevy::prelude::*;
use crate::components::*;

#[derive(Resource, Default)]
pub struct UndoSystem {
    pub snapshots: Vec<GameSnapshot>,
    pub max_snapshots: usize,
    pub undo_available: bool,
}

impl Default for UndoSystem {
    fn default() -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots: 5, // Keep last 5 actions
            undo_available: false,
        }
    }
}

#[derive(Clone)]
pub struct GameSnapshot {
    pub players: Vec<PlayerSnapshot>,
    pub vineyards: Vec<VineyardSnapshot>,
    pub hands: Vec<HandSnapshot>,
    pub workers: Vec<WorkerSnapshot>,
    pub turn_order: TurnOrderSnapshot,
    pub action_spaces: Vec<ActionSpaceSnapshot>,
    pub timestamp: f32,
}

#[derive(Clone)]
pub struct PlayerSnapshot {
    pub id: u8,
    pub victory_points: u8,
    pub lira: u8,
    pub workers: u8,
}

#[derive(Clone)]
pub struct VineyardSnapshot {
    pub owner_id: u8,
    pub red_grapes: u8,
    pub white_grapes: u8,
    pub red_wine: u8,
    pub white_wine: u8,
    pub lira: u8,
    pub fields: [Option<(bool, u8)>; 9], // (is_red, value)
}

#[derive(Clone)]
pub struct HandSnapshot {
    pub owner_id: u8,
    pub vine_card_count: usize,
    pub wine_order_count: usize,
}

#[derive(Clone)]
pub struct WorkerSnapshot {
    pub owner_id: u8,
    pub is_grande: bool,
    pub placed_at: Option<u8>, // ActionSpace as u8
    pub position_x: f32,
    pub position_y: f32,
}

#[derive(Clone)]
pub struct TurnOrderSnapshot {
    pub current_player: usize,
}

#[derive(Clone)]
pub struct ActionSpaceSnapshot {
    pub action: u8,
    pub occupied_by: Option<u8>,
    pub bonus_worker_slot: Option<u8>,
}

pub fn create_snapshot_system(
    mut undo_system: ResMut<UndoSystem>,
    players: Query<&Player>,
    vineyards: Query<&Vineyard>,
    hands: Query<&Hand>,
    workers: Query<&Worker>,
    turn_order: Res<TurnOrder>,
    action_spaces: Query<&ActionSpaceSlot>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Create snapshot before each player action (when ENTER is pressed)
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        let snapshot = create_game_snapshot(
            &players, &vineyards, &hands, &workers,
            &turn_order, &action_spaces, time.elapsed_seconds()
        );
        
        undo_system.snapshots.push(snapshot);
        
        // Keep only the last N snapshots
        if undo_system.snapshots.len() > undo_system.max_snapshots {
            undo_system.snapshots.remove(0);
        }
        
        undo_system.undo_available = !undo_system.snapshots.is_empty();
    }
}

pub fn undo_action_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut undo_system: ResMut<UndoSystem>,
    mut commands: Commands,
    entities: Query<Entity, (Without<Camera>, Without<Window>)>,
    mut turn_order: ResMut<TurnOrder>,
    time: Res<Time>,
) {
    // Undo with Ctrl+Z
    if (keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight)) && 
       keyboard.just_pressed(KeyCode::KeyZ) {
        
        if let Some(snapshot) = undo_system.snapshots.pop() {
            // Only allow undo within 30 seconds of the action
            if time.elapsed_seconds() - snapshot.timestamp < 30.0 {
                info!("Undoing last action");
                
                // Clear current game state
                for entity in entities.iter() {
                    commands.entity(entity).despawn();
                }
                
                // Restore from snapshot
                restore_from_snapshot(&mut commands, &snapshot, &mut turn_order);
                
                undo_system.undo_available = !undo_system.snapshots.is_empty();
            } else {
                info!("Undo expired (too much time passed)");
                undo_system.snapshots.push(snapshot); // Put it back
            }
        } else {
            info!("No actions to undo");
        }
    }
}

fn create_game_snapshot(
    players: &Query<&Player>,
    vineyards: &Query<&Vineyard>,
    hands: &Query<&Hand>,
    workers: &Query<&Worker>,
    turn_order: &TurnOrder,
    action_spaces: &Query<&ActionSpaceSlot>,
    timestamp: f32,
) -> GameSnapshot {
    let players_snapshot: Vec<_> = players.iter().map(|p| PlayerSnapshot {
        id: p.id.0,
        victory_points: p.victory_points,
        lira: p.lira,
        workers: p.workers,
    }).collect();
    
    let vineyards_snapshot: Vec<_> = vineyards.iter().map(|v| VineyardSnapshot {
        owner_id: v.owner.0,
        red_grapes: v.red_grapes,
        white_grapes: v.white_grapes,
        red_wine: v.red_wine,
        white_wine: v.white_wine,
        lira: v.lira,
        fields: v.fields.map(|f| f.map(|vt| match vt {
            VineType::Red(val) => (true, val),
            VineType::White(val) => (false, val),
        })),
    }).collect();
    
    let hands_snapshot: Vec<_> = hands.iter().map(|h| HandSnapshot {
        owner_id: h.owner.0,
        vine_card_count: h.vine_cards.len(),
        wine_order_count: h.wine_order_cards.len(),
    }).collect();
    
    let workers_snapshot: Vec<_> = workers.iter().map(|w| WorkerSnapshot {
        owner_id: w.owner.0,
        is_grande: w.is_grande,
        placed_at: w.placed_at.map(action_to_u8),
        position_x: w.position.x,
        position_y: w.position.y,
    }).collect();
    
    let turn_order_snapshot = TurnOrderSnapshot {
        current_player: turn_order.current_player,
    };
    
    let action_spaces_snapshot: Vec<_> = action_spaces.iter().map(|s| ActionSpaceSnapshot {
        action: action_to_u8(s.action),
        occupied_by: s.occupied_by.map(|p| p.0),
        bonus_worker_slot: s.bonus_worker_slot.map(|p| p.0),
    }).collect();
    
    GameSnapshot {
        players: players_snapshot,
        vineyards: vineyards_snapshot,
        hands: hands_snapshot,
        workers: workers_snapshot,
        turn_order: turn_order_snapshot,
        action_spaces: action_spaces_snapshot,
        timestamp,
    }
}

fn restore_from_snapshot(
    commands: &mut Commands,
    snapshot: &GameSnapshot,
    turn_order: &mut ResMut<TurnOrder>,
) {
    // Restore players
    for player_snap in &snapshot.players {
        commands.spawn(Player {
            id: PlayerId(player_snap.id),
            name: format!("Player {}", player_snap.id + 1),
            victory_points: player_snap.victory_points,
            lira: player_snap.lira,
            workers: player_snap.workers,
            grande_worker_available: true,
        });
    }
    
    // Restore vineyards
    for vineyard_snap in &snapshot.vineyards {
        commands.spawn(Vineyard {
            owner: PlayerId(vineyard_snap.owner_id),
            fields: vineyard_snap.fields.map(|f| f.map(|(is_red, val)| {
                if is_red {
                    VineType::Red(val)
                } else {
                    VineType::White(val)
                }
            })),
            red_grapes: vineyard_snap.red_grapes,
            white_grapes: vineyard_snap.white_grapes,
            red_wine: vineyard_snap.red_wine,
            white_wine: vineyard_snap.white_wine,
            lira: vineyard_snap.lira,
        });
    }
    
    // Restore hands (simplified - just create empty hands)
    for hand_snap in &snapshot.hands {
        commands.spawn(Hand {
            owner: PlayerId(hand_snap.owner_id),
            vine_cards: Vec::new(), // Simplified restoration
            wine_order_cards: Vec::new(),
        });
    }
    
    // Restore workers
    for worker_snap in &snapshot.workers {
        commands.spawn((
            Worker {
                owner: PlayerId(worker_snap.owner_id),
                is_grande: worker_snap.is_grande,
                placed_at: worker_snap.placed_at.and_then(u8_to_action),
                position: Vec2::new(worker_snap.position_x, worker_snap.position_y),
            },
            Clickable { size: Vec2::new(20.0, 20.0) },
        ));
    }
    
    // Restore action spaces
    let action_board = ActionBoard::new();
    for (i, space_snap) in snapshot.action_spaces.iter().enumerate() {
        if let Some(mut space) = action_board.spaces.get(i).cloned() {
            space.occupied_by = space_snap.occupied_by.map(PlayerId);
            space.bonus_worker_slot = space_snap.bonus_worker_slot.map(PlayerId);
            commands.spawn((
                space,
                Clickable { size: Vec2::new(60.0, 30.0) },
            ));
        }
    }
    commands.spawn(action_board);
    
    // Restore turn order
    turn_order.current_player = snapshot.turn_order.current_player;
}

pub fn display_undo_status_system(
    undo_system: Res<UndoSystem>,
    mut commands: Commands,
    existing_undo_ui: Query<Entity, With<UndoStatusText>>,
) {
    // Clean up old UI
    for entity in existing_undo_ui.iter() {
        commands.entity(entity).despawn();
    }
    
    if undo_system.undo_available {
        commands.spawn((
            TextBundle::from_section(
                "Press Ctrl+Z to undo last action",
                TextStyle {
                    font_size: 14.0,
                    color: Color::from(Srgba::new(1.0, 1.0, 0.0, 0.8)),
                    ..default()
                },
            ).with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            }),
            UndoStatusText,
        ));
    }
}

#[derive(Component)]
pub struct UndoStatusText;

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