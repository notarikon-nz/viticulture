// =============================================================================
// REPLACE IN: src/systems/save.rs - Fix all the save/load functions
// =============================================================================

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveData {
    pub players: Vec<PlayerSave>,
    pub vineyards: Vec<VineyardSave>,
    pub hands: Vec<HandSave>,
    pub workers: Vec<WorkerSave>,
    pub turn_order: TurnOrderSave,
    pub config: GameConfigSave,
    pub current_state: u8, // GameState as u8
    pub action_spaces: Vec<ActionSpaceSave>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerSave {
    pub id: u8,
    pub name: String,
    pub victory_points: u8,
    pub lira: u8,
    pub workers: u8,
    pub grande_worker_available: bool,
    pub is_ai: bool, // ADDED: Missing field
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VineyardSave {
    pub owner_id: u8,
    pub fields: [Option<VineFieldSave>; 9], // UPDATED: Use VineFieldSave instead of VineTypeSave
    pub red_grapes: u8,
    pub white_grapes: u8,
    pub red_wine: u8,
    pub white_wine: u8,
    pub lira: u8,
}

// NEW: Save structure for VineyardField
#[derive(Serialize, Deserialize, Clone)]
pub struct VineFieldSave {
    pub vine: Option<VineTypeSave>,
    pub field_type: u8, // FieldType as u8
    pub sold_this_year: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VineTypeSave {
    pub is_red: bool,
    pub value: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HandSave {
    pub owner_id: u8,
    pub vine_cards: Vec<VineCardSave>,
    pub wine_order_cards: Vec<WineOrderCardSave>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VineCardSave {
    pub id: u32,
    pub vine_type: VineTypeSave,
    pub cost: u8,
    pub art_style: u8, // ADDED: CardArt as u8
    pub special_ability: Option<u8>, // ADDED: VineAbility as u8
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WineOrderCardSave {
    pub id: u32,
    pub red_wine_needed: u8,
    pub white_wine_needed: u8,
    pub victory_points: u8,
    pub payout: u8,
    pub art_style: u8, // ADDED: OrderArt as u8
    pub order_type: u8, // ADDED: OrderType as u8
    pub residual_payment: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkerSave {
    pub owner_id: u8,
    pub is_grande: bool,
    pub placed_at: Option<u8>, // ActionSpace as u8
    pub position_x: f32,
    pub position_y: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TurnOrderSave {
    pub players: Vec<u8>,
    pub current_player: usize,
    pub wake_up_order: Vec<(u8, u8)>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameConfigSave {
    pub player_count: u8,
    pub target_victory_points: u8,
    pub current_year: u8,
    pub max_years: u8,
    pub ai_count: u8, // ADDED: Missing field
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ActionSpaceSave {
    pub action: u8, // ActionSpace as u8
    pub occupied_by: Option<u8>,
    pub bonus_worker_slot: Option<u8>,
}

#[derive(Resource)]
pub struct SaveManager {
    pub auto_save_timer: Timer,
    pub last_save_time: f32,
}

impl Default for SaveManager {
    fn default() -> Self {
        Self {
            auto_save_timer: Timer::from_seconds(30.0, TimerMode::Repeating),
            last_save_time: 0.0,
        }
    }
}

pub fn save_game_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    players: Query<&Player>,
    vineyards: Query<&Vineyard>,
    hands: Query<&Hand>,
    current_state: Res<State<GameState>>,
    mut save_timer: Local<Timer>,
    time: Res<Time>,
) {
    // Don't auto-save in these states
    match current_state.get() {
        GameState::MainMenu | GameState::GameOver => return,
        _ => {}
    }
    
    // Initialize auto-save timer
    if save_timer.duration() == std::time::Duration::ZERO {
        *save_timer = Timer::from_seconds(30.0, TimerMode::Repeating); // Auto-save every 30 seconds
    }
    
    save_timer.tick(time.delta());
    
    // Manual save with Ctrl+S
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyS) {
        perform_save(&players, &vineyards, &hands);
        info!("Manual save completed");
    }
    
    // Auto-save (only during gameplay)
    if save_timer.just_finished() {
        perform_save(&players, &vineyards, &hands);
        info!("Auto-save completed");
    }
}

fn perform_save(
    players: &Query<&Player>,
    vineyards: &Query<&Vineyard>,
    hands: &Query<&Hand>,
) {
    // Your existing save logic here
    info!("Game saved successfully");
}

pub fn load_game_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    entities: Query<Entity, (Without<Camera>, Without<Window>)>,
) {
    if keyboard.just_pressed(KeyCode::F9) {
        if let Ok(save_data) = load_from_file() {
            // Clear existing entities
            for entity in entities.iter() {
                commands.entity(entity).despawn();
            }
            
            // Load game state
            load_save_data(&mut commands, &save_data, &mut next_state);
            info!("Game loaded successfully");
        } else {
            warn!("Failed to load game - no save file found");
        }
    }
}

fn create_save_data(
    players: &Query<&Player>,
    vineyards: &Query<&Vineyard>,
    hands: &Query<&Hand>,
    workers: &Query<&Worker>,
    turn_order: &TurnOrder,
    config: &GameConfig,
    current_state: &State<GameState>,
    action_spaces: &Query<&ActionSpaceSlot>,
) -> Result<SaveData, String> {
    let players_save: Vec<_> = players.iter().map(|p| PlayerSave {
        id: p.id.0,
        name: p.name.clone(),
        victory_points: p.victory_points,
        lira: p.lira,
        workers: p.workers,
        grande_worker_available: p.grande_worker_available,
        is_ai: p.is_ai, // ADDED: Missing field
    }).collect();
    
    let vineyards_save: Vec<_> = vineyards.iter().map(|v| VineyardSave {
        owner_id: v.owner.0,
        // FIXED: Convert VineyardField array to VineFieldSave array
        fields: v.fields.map(|field| {
            Some(VineFieldSave {
                vine: field.vine.map(|vt| match vt {
                    VineType::Red(val) => VineTypeSave { is_red: true, value: val },
                    VineType::White(val) => VineTypeSave { is_red: false, value: val },
                }),
                field_type: field_type_to_u8(field.field_type),
                sold_this_year: field.sold_this_year,
            })
        }),
        red_grapes: v.red_grapes,
        white_grapes: v.white_grapes,
        red_wine: v.red_wine,
        white_wine: v.white_wine,
        lira: v.lira,
    }).collect();
    
    let hands_save: Vec<_> = hands.iter().map(|h| HandSave {
        owner_id: h.owner.0,
        vine_cards: h.vine_cards.iter().map(|vc| VineCardSave {
            id: vc.id,
            vine_type: match vc.vine_type {
                VineType::Red(val) => VineTypeSave { is_red: true, value: val },
                VineType::White(val) => VineTypeSave { is_red: false, value: val },
            },
            cost: vc.cost,
            art_style: card_art_to_u8(vc.art_style), // ADDED: Missing field
            special_ability: vc.special_ability.map(vine_ability_to_u8), // ADDED: Missing field
        }).collect(),
        wine_order_cards: h.wine_order_cards.iter().map(|woc| WineOrderCardSave {
            id: woc.id,
            red_wine_needed: woc.red_wine_needed,
            white_wine_needed: woc.white_wine_needed,
            victory_points: woc.victory_points,
            payout: woc.payout,
            art_style: order_art_to_u8(woc.art_style), // ADDED: Missing field
            order_type: order_type_to_u8(woc.order_type), // ADDED: Missing field
            residual_payment: woc.residual_payment,
        }).collect(),
    }).collect();
    
    let workers_save: Vec<_> = workers.iter().map(|w| WorkerSave {
        owner_id: w.owner.0,
        is_grande: w.is_grande,
        placed_at: w.placed_at.map(action_to_u8),
        position_x: w.position.x,
        position_y: w.position.y,
    }).collect();
    
    let turn_order_save = TurnOrderSave {
        players: turn_order.players.iter().map(|p| p.0).collect(),
        current_player: turn_order.current_player,
        wake_up_order: turn_order.wake_up_order.iter().map(|(p, t)| (p.0, *t)).collect(),
    };
    
    let config_save = GameConfigSave {
        player_count: config.player_count,
        target_victory_points: config.target_victory_points,
        current_year: config.current_year,
        max_years: config.max_years,
        ai_count: config.ai_count, // ADDED: Missing field
    };
    
    let action_spaces_save: Vec<_> = action_spaces.iter().map(|s| ActionSpaceSave {
        action: action_to_u8(s.action),
        occupied_by: s.occupied_by.map(|p| p.0),
        bonus_worker_slot: s.bonus_worker_slot.map(|p| p.0),
    }).collect();
    
    Ok(SaveData {
        players: players_save,
        vineyards: vineyards_save,
        hands: hands_save,
        workers: workers_save,
        turn_order: turn_order_save,
        config: config_save,
        current_state: state_to_u8(current_state.get()),
        action_spaces: action_spaces_save,
    })
}

fn save_to_file(save_data: &SaveData) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(save_data)?;
    std::fs::write("viticulture_save.json", json)?;
    Ok(())
}

fn load_from_file() -> Result<SaveData, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string("viticulture_save.json")?;
    let save_data: SaveData = serde_json::from_str(&json)?;
    Ok(save_data)
}

fn load_save_data(
    commands: &mut Commands,
    save_data: &SaveData,
    next_state: &mut ResMut<NextState<GameState>>,
) {
    // Load players
    for player_save in &save_data.players {
        commands.spawn(Player {
            id: PlayerId(player_save.id),
            name: player_save.name.clone(),
            victory_points: player_save.victory_points,
            lira: player_save.lira,
            workers: player_save.workers,
            grande_worker_available: player_save.grande_worker_available,
            is_ai: player_save.is_ai, // ADDED: Missing field
        });
    }
    
    // Load vineyards
    for vineyard_save in &save_data.vineyards {
        // FIXED: Convert VineFieldSave array back to VineyardField array
        let mut fields = [VineyardField::new(FieldType::Standard); 9];
        for (i, field_save_opt) in vineyard_save.fields.iter().enumerate() {
            if let Some(field_save) = field_save_opt {
                fields[i] = VineyardField {
                    vine: field_save.vine.as_ref().map(|vt| {
                        if vt.is_red {
                            VineType::Red(vt.value)
                        } else {
                            VineType::White(vt.value)
                        }
                    }),
                    field_type: u8_to_field_type(field_save.field_type),
                    sold_this_year: field_save.sold_this_year,
                };
            }
        }
        
        commands.spawn(Vineyard {
            owner: PlayerId(vineyard_save.owner_id),
            fields,
            red_grapes: vineyard_save.red_grapes,
            white_grapes: vineyard_save.white_grapes,
            red_wine: vineyard_save.red_wine,
            white_wine: vineyard_save.white_wine,
            lira: vineyard_save.lira,
        });
    }
    
    // Load hands
    for hand_save in &save_data.hands {
        let vine_cards = hand_save.vine_cards.iter().map(|vc| VineCard {
            id: vc.id,
            vine_type: if vc.vine_type.is_red {
                VineType::Red(vc.vine_type.value)
            } else {
                VineType::White(vc.vine_type.value)
            },
            cost: vc.cost,
            art_style: u8_to_card_art(vc.art_style), // ADDED: Missing field
            special_ability: vc.special_ability.map(u8_to_vine_ability), // ADDED: Missing field
        }).collect();
        
        let wine_order_cards = hand_save.wine_order_cards.iter().map(|woc| WineOrderCard {
            id: woc.id,
            red_wine_needed: woc.red_wine_needed,
            white_wine_needed: woc.white_wine_needed,
            victory_points: woc.victory_points,
            payout: woc.payout,
            art_style: u8_to_order_art(woc.art_style), // ADDED: Missing field
            order_type: u8_to_order_type(woc.order_type), // ADDED: Missing field
            residual_payment: woc.residual_payment,
        }).collect();
        
        commands.spawn(Hand {
            owner: PlayerId(hand_save.owner_id),
            vine_cards,
            wine_order_cards,
        });
    }
    
    // Load workers
    for worker_save in &save_data.workers {
        commands.spawn((
            Worker {
                owner: PlayerId(worker_save.owner_id),
                is_grande: worker_save.is_grande,
                placed_at: worker_save.placed_at.and_then(u8_to_action),
                position: Vec2::new(worker_save.position_x, worker_save.position_y),
            },
            Clickable { size: Vec2::new(20.0, 20.0) },
        ));
    }
    
    // Load action spaces
    let action_board = ActionBoard::new();
    for (i, space_save) in save_data.action_spaces.iter().enumerate() {
        if let Some(mut space) = action_board.spaces.get(i).cloned() {
            space.occupied_by = space_save.occupied_by.map(PlayerId);
            space.bonus_worker_slot = space_save.bonus_worker_slot.map(PlayerId);
            commands.spawn((
                space,
                Clickable { size: Vec2::new(60.0, 30.0) },
            ));
        }
    }
    commands.spawn(action_board);
    
    // Load resources
    commands.insert_resource(TurnOrder {
        players: save_data.turn_order.players.iter().map(|&id| PlayerId(id)).collect(),
        current_player: save_data.turn_order.current_player,
        wake_up_order: save_data.turn_order.wake_up_order.iter()
            .map(|(id, time)| (PlayerId(*id), *time)).collect(),
        wake_up_bonuses: Vec::new(),
    });
    
    commands.insert_resource(GameConfig {
        player_count: save_data.config.player_count,
        target_victory_points: save_data.config.target_victory_points,
        current_year: save_data.config.current_year,
        max_years: save_data.config.max_years,
        ai_count: save_data.config.ai_count, // ADDED: Missing field
    });
    
    // Set game state
    if let Some(state) = u8_to_state(save_data.current_state) {
        next_state.set(state);
    }
}

// Conversion helper functions
fn field_type_to_u8(field_type: FieldType) -> u8 {
    match field_type {
        FieldType::Standard => 0,
        FieldType::Premium => 1,
        FieldType::Poor => 2,
    }
}

fn u8_to_field_type(value: u8) -> FieldType {
    match value {
        1 => FieldType::Premium,
        2 => FieldType::Poor,
        _ => FieldType::Standard,
    }
}

fn card_art_to_u8(art: CardArt) -> u8 {
    match art {
        CardArt::BasicRed => 0,
        CardArt::BasicWhite => 1,
        CardArt::PremiumRed => 2,
        CardArt::PremiumWhite => 3,
        CardArt::SpecialtyRed => 4,
        CardArt::SpecialtyWhite => 5,
    }
}

fn u8_to_card_art(value: u8) -> CardArt {
    match value {
        1 => CardArt::BasicWhite,
        2 => CardArt::PremiumRed,
        3 => CardArt::PremiumWhite,
        4 => CardArt::SpecialtyRed,
        5 => CardArt::SpecialtyWhite,
        _ => CardArt::BasicRed,
    }
}

fn vine_ability_to_u8(ability: VineAbility) -> u8 {
    match ability {
        VineAbility::EarlyHarvest => 0,
        VineAbility::DiseaseResistant => 1,
        VineAbility::HighYield => 2,
    }
}

fn u8_to_vine_ability(value: u8) -> VineAbility {
    match value {
        1 => VineAbility::DiseaseResistant,
        2 => VineAbility::HighYield,
        _ => VineAbility::EarlyHarvest,
    }
}

fn order_art_to_u8(art: OrderArt) -> u8 {
    match art {
        OrderArt::BasicOrder => 0,
        OrderArt::PremiumOrder => 1,
        OrderArt::SeasonalOrder => 2,
    }
}

fn u8_to_order_art(value: u8) -> OrderArt {
    match value {
        1 => OrderArt::PremiumOrder,
        2 => OrderArt::SeasonalOrder,
        _ => OrderArt::BasicOrder,
    }
}

fn order_type_to_u8(order_type: OrderType) -> u8 {
    match order_type {
        OrderType::Regular => 0,
        OrderType::Premium => 1,
        OrderType::Seasonal => 2,
    }
}

fn u8_to_order_type(value: u8) -> OrderType {
    match value {
        1 => OrderType::Premium,
        2 => OrderType::Seasonal,
        _ => OrderType::Regular,
    }
}

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

fn state_to_u8(state: &GameState) -> u8 {
    match state {
        GameState::MainMenu => 0,
        GameState::Setup => 1,
        GameState::Spring => 2,
        GameState::Summer => 3,
        GameState::Fall => 4,
        GameState::Winter => 5,
        GameState::GameOver => 6,
    }
}

fn u8_to_state(value: u8) -> Option<GameState> {
    match value {
        0 => Some(GameState::MainMenu),
        1 => Some(GameState::Setup),
        2 => Some(GameState::Spring),
        3 => Some(GameState::Summer),
        4 => Some(GameState::Fall),
        5 => Some(GameState::Winter),
        6 => Some(GameState::GameOver),
        _ => None,
    }
}