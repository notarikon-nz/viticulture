use bevy::prelude::*;
use crate::components::*;

#[derive(Resource, Default)]
pub struct ExpansionSettings {
    pub tuscany_enabled: bool,
    pub visitor_cards_enabled: bool,
    pub advanced_boards_enabled: bool,
    pub extended_board: bool,
}

#[derive(Component, Clone)]
pub struct VisitorCard {
    pub id: u32,
    pub name: String,
    pub effect: VisitorEffect,
    pub season: VisitorSeason,
    pub cost: u8,
}

#[derive(Clone)]
pub enum VisitorEffect {
    GainLira(u8),
    GainVP(u8),
    DrawCards(u8),
    PlantFreeVine,
    HarvestBonus(u8),
    WineBonus(u8),
    StructureDiscount(u8),
    ExtraWorker,
    SwapFields,
}


#[derive(Clone, Copy)]
pub enum VisitorSeason {
    Summer,
    Winter,
}

#[derive(Clone, Copy)]
pub enum VisitorTiming {
    Summer,
    Winter,
    Either,
}

#[derive(Component)]
pub struct AdvancedVineyard {
    pub owner: PlayerId,
    pub board_type: VineyardBoardType,
    pub special_ability: SpecialAbility,
    pub bonus_fields: Vec<(usize, BonusFieldType)>,
}

#[derive(Clone, Copy)]
pub enum VineyardBoardType {
    Standard,
    Tuscan,    // +1 lira when planting
    Sicilian,  // +1 VP when making wine
    Venetian,  // Extra worker at start
    Roman,     // Cheaper structures
}

#[derive(Clone, Copy)]
pub enum SpecialAbility {
    None,
    WakeUpFirst,      // Always wake up first
    BonusHarvest,     // +1 grape when harvesting
    CheapStructures,  // -1 cost for all structures
    ExtraLira,        // +1 lira per turn
}

#[derive(Clone, Copy)]
pub enum BonusFieldType {
    ExtraGrape,    // +1 grape when harvesting this field
    ExtraWine,     // +1 wine when using grapes from this field
    ExtraLira,     // +1 lira when planting here
}

#[derive(Resource)]
pub struct VisitorDeck {
    pub summer_visitors: Vec<VisitorCard>,
    pub winter_visitors: Vec<VisitorCard>,
    pub summer_discard: Vec<VisitorCard>,
    pub winter_discard: Vec<VisitorCard>,
}

impl VisitorDeck {
    pub fn new() -> Self {
        let mut summer_visitors = Vec::new();
        let mut winter_visitors = Vec::new();
        
        // Summer visitors (income/preparation focused)
        summer_visitors.push(VisitorCard {
            id: 1000,
            name: "Merchant".to_string(),
            effect: VisitorEffect::GainLira(3),
            season: VisitorSeason::Summer,
            cost: 0,
        });
        
        summer_visitors.push(VisitorCard {
            id: 1001,
            name: "Architect".to_string(),
            effect: VisitorEffect::StructureDiscount(2),
            season: VisitorSeason::Summer,
            cost: 1,
        });
        
        summer_visitors.push(VisitorCard {
            id: 1002,
            name: "Innkeeper".to_string(),
            effect: VisitorEffect::DrawCards(2),
            season: VisitorSeason::Summer,
            cost: 0,
        });
        
        summer_visitors.push(VisitorCard {
            id: 1003,
            name: "Traveling Salesman".to_string(),
            effect: VisitorEffect::PlantFreeVine,
            season: VisitorSeason::Summer,
            cost: 0,
        });
        
        // Winter visitors (production/scoring focused)
        winter_visitors.push(VisitorCard {
            id: 2000,
            name: "Wine Critic".to_string(),
            effect: VisitorEffect::GainVP(2),
            season: VisitorSeason::Winter,
            cost: 1,
        });
        
        winter_visitors.push(VisitorCard {
            id: 2001,
            name: "Harvest Master".to_string(),
            effect: VisitorEffect::HarvestBonus(2),
            season: VisitorSeason::Winter,
            cost: 0,
        });
        
        winter_visitors.push(VisitorCard {
            id: 2002,
            name: "Cellar Master".to_string(),
            effect: VisitorEffect::WineBonus(2),
            season: VisitorSeason::Winter,
            cost: 1,
        });
        
        winter_visitors.push(VisitorCard {
            id: 2003,
            name: "Noble Patron".to_string(),
            effect: VisitorEffect::GainVP(3),
            season: VisitorSeason::Winter,
            cost: 2,
        });
        
        Self {
            summer_visitors,
            winter_visitors,
            summer_discard: Vec::new(),
            winter_discard: Vec::new(),
        }
    }
    
    pub fn draw_summer_visitor(&mut self) -> Option<VisitorCard> {
        if self.summer_visitors.is_empty() && !self.summer_discard.is_empty() {
            self.summer_visitors.append(&mut self.summer_discard);
            use rand::seq::SliceRandom;
            let mut rng = rand::rng();
            self.summer_visitors.shuffle(&mut rng);
        }
        self.summer_visitors.pop()
    }
    
    pub fn draw_winter_visitor(&mut self) -> Option<VisitorCard> {
        if self.winter_visitors.is_empty() && !self.winter_discard.is_empty() {
            self.winter_visitors.append(&mut self.winter_discard);
            use rand::seq::SliceRandom;
            let mut rng = rand::rng();
            self.winter_visitors.shuffle(&mut rng);
        }
        self.winter_visitors.pop()
    }
}

// Extended action spaces for Tuscany
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExtendedActionSpace {
    // New summer actions
    DrawVisitor,
    Trade,
    YokeBuilding,
    
    // New winter actions
    PlayVisitor,
    AgedWineProduction,
    Wedding,
}

pub fn setup_tuscany_expansion_system(
    mut commands: Commands,
    expansion_settings: Res<ExpansionSettings>,
    existing_visitors: Query<Entity, (With<VisitorCard>,Without<MarkedForDespawn>)>,
) {
    if !expansion_settings.tuscany_enabled {
        return;
    }
    
    // Clean up existing visitor cards
    for entity in existing_visitors.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
    
    // Initialize visitor deck
    commands.insert_resource(VisitorDeck::new());
    
    info!("Tuscany expansion enabled with visitor cards");
}

pub fn handle_visitor_cards_system(
    visitor_deck: Option<ResMut<VisitorDeck>>,
    mut hands: Query<&mut Hand>,
    mut players: Query<&mut Player>,
    mut vineyards: Query<&mut Vineyard>,
    keyboard: Res<ButtonInput<KeyCode>>,
    turn_order: Res<TurnOrder>,
    current_state: Res<State<GameState>>,
    expansion_settings: Res<ExpansionSettings>,
) {
    // Early return if expansion not enabled or resource not available
    if !expansion_settings.visitor_cards_enabled {
        return;
    }
    
    let Some(mut visitor_deck) = visitor_deck else {
        return; // Resource not available, skip system
    };
    
    // Draw visitor card with V key
    if keyboard.just_pressed(KeyCode::KeyV) {
        if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
            let visitor = match current_state.get() {
                GameState::Summer => visitor_deck.draw_summer_visitor(),
                GameState::Winter => visitor_deck.draw_winter_visitor(),
                _ => None,
            };
            
            if let Some(visitor_card) = visitor {
                execute_visitor_effect(*current_player_id, &visitor_card, &mut hands, &mut players, &mut vineyards);
                info!("Player {:?} played visitor: {}", current_player_id, visitor_card.name);
            }
        }
    }
}

fn execute_visitor_effect(
    player_id: PlayerId,
    visitor: &VisitorCard,
    hands: &mut Query<&mut Hand>,
    players: &mut Query<&mut Player>,
    vineyards: &mut Query<&mut Vineyard>,
) {
    let mut player = players.iter_mut().find(|p| p.id == player_id);
    let mut vineyard = vineyards.iter_mut().find(|v| v.owner == player_id);
    let mut hand = hands.iter_mut().find(|h| h.owner == player_id);
    
    // Check if player can afford the visitor
    if let Some(ref mut p) = player {
        if p.lira < visitor.cost {
            return; // Can't afford
        }
        p.lira = p.lira.saturating_sub(visitor.cost);
    }
    
    match visitor.effect {
        VisitorEffect::GainLira(amount) => {
            if let Some(ref mut p) = player {
                p.gain_lira(amount);
            }
        }
        VisitorEffect::GainVP(amount) => {
            if let Some(ref mut p) = player {
                p.gain_victory_points(amount);
            }
        }
        VisitorEffect::DrawCards(amount) => {
            if let Some(ref mut h) = hand {
                // Simplified: just track that cards were drawn
                info!("Player draws {} cards", amount);
            }
        }
        VisitorEffect::PlantFreeVine => {
            if let (Some(ref mut h), Some(ref mut v)) = (hand.as_mut(), vineyard.as_mut()) {
                if !h.vine_cards.is_empty() {
                    let vine_card = h.vine_cards.remove(0);
                    // FIXED: Find an empty field and plant the vine
                    for field in v.fields.iter_mut() {
                        if field.vine.is_none() { // FIXED: Check field.vine instead of field
                            field.vine = Some(vine_card.vine_type); // FIXED: Set field.vine instead of field
                            break;
                        }
                    }
                }
            }
        }
        VisitorEffect::HarvestBonus(amount) => {
            if let Some(ref mut v) = vineyard {
                v.red_grapes += amount;
                v.white_grapes += amount;
            }
        }
        VisitorEffect::WineBonus(amount) => {
            if let Some(ref mut v) = vineyard {
                v.red_wine += amount;
                v.white_wine += amount;
            }
        }
        VisitorEffect::StructureDiscount(_amount) => {
            // Temporary discount applied to next structure build
            info!("Structure discount applied");
        }
        VisitorEffect::ExtraWorker => {
            if let Some(ref mut p) = player {
                p.workers += 1;
            }
        }
        VisitorEffect::SwapFields => {
            // Advanced effect - swap two vineyard fields
            // FIXED: Update to work with VineyardField
            if let Some(ref mut v) = vineyard {
                // Simple implementation: swap first two non-empty fields
                let mut first_vine = None;
                let mut second_vine = None;
                let mut first_idx = None;
                let mut second_idx = None;
                
                for (i, field) in v.fields.iter().enumerate() {
                    if field.vine.is_some() {
                        if first_idx.is_none() {
                            first_vine = field.vine;
                            first_idx = Some(i);
                        } else if second_idx.is_none() {
                            second_vine = field.vine;
                            second_idx = Some(i);
                            break;
                        }
                    }
                }
                
                // Perform the swap
                if let (Some(first), Some(second)) = (first_idx, second_idx) {
                    v.fields[first].vine = second_vine;
                    v.fields[second].vine = first_vine;
                    info!("Swapped vines between fields {} and {}", first, second);
                }
            }
        }
    }
}

pub fn setup_advanced_vineyards_system(
    mut commands: Commands,
    expansion_settings: Res<ExpansionSettings>,
    players: Query<&Player>,
    existing_advanced: Query<Entity, (With<AdvancedVineyard>, Without<MarkedForDespawn>)>,
) {
    if !expansion_settings.advanced_boards_enabled {
        return;
    }
    
    // Clean up existing advanced vineyards
    for entity in existing_advanced.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
    
    // Create advanced vineyard for each player
    for (i, player) in players.iter().enumerate() {
        let board_type = match i % 4 {
            0 => VineyardBoardType::Tuscan,
            1 => VineyardBoardType::Sicilian,
            2 => VineyardBoardType::Venetian,
            _ => VineyardBoardType::Roman,
        };
        
        let special_ability = match board_type {
            VineyardBoardType::Tuscan => SpecialAbility::ExtraLira,
            VineyardBoardType::Sicilian => SpecialAbility::BonusHarvest,
            VineyardBoardType::Venetian => SpecialAbility::WakeUpFirst,
            VineyardBoardType::Roman => SpecialAbility::CheapStructures,
            VineyardBoardType::Standard => SpecialAbility::None,
        };
        
        commands.spawn(AdvancedVineyard {
            owner: player.id,
            board_type,
            special_ability,
            bonus_fields: vec![
                (2, BonusFieldType::ExtraGrape),
                (5, BonusFieldType::ExtraWine),
                (8, BonusFieldType::ExtraLira),
            ],
        });
    }
    
    info!("Advanced vineyard boards enabled");
}

pub fn apply_board_bonuses_system(
    advanced_vineyards: Query<&AdvancedVineyard>,
    mut players: Query<&mut Player>,
    vineyards: Query<&mut Vineyard>,
    expansion_settings: Res<ExpansionSettings>,
) {
    if !expansion_settings.advanced_boards_enabled {
        return;
    }
    
    for advanced in advanced_vineyards.iter() {
        if let Some(player) = players.iter_mut().find(|p| p.id == advanced.owner) {
            match advanced.special_ability {
                SpecialAbility::ExtraLira => {
                    // Applied per turn in other systems
                }
                SpecialAbility::WakeUpFirst => {
                    // Applied in wake-up system
                }
                _ => {}
            }
        }
    }
}

pub fn create_extended_wine_orders() -> Vec<WineOrderCard> {
    vec![
        WineOrderCard::new(200, 1, 2, 3, 3),  // Blush order
        WineOrderCard::new(201, 2, 1, 3, 3),  // Mixed order
        WineOrderCard::new(202, 3, 3, 8, 7),  // Sparkling order
        WineOrderCard::new(203, 4, 2, 9, 8),  // Premium blend
        WineOrderCard::new(204, 2, 4, 9, 8),  // White premium
        WineOrderCard::new(205, 5, 1, 10, 9), // Master vintage
        WineOrderCard::new(206, 1, 5, 10, 9), // Master white
        WineOrderCard::new(207, 4, 4, 12, 10), // Grand vintage
    ]
}

pub fn expansion_toggle_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut expansion_settings: ResMut<ExpansionSettings>,
) {
    // Toggle expansions with function keys
    if keyboard.just_pressed(KeyCode::F6) {
        expansion_settings.tuscany_enabled = !expansion_settings.tuscany_enabled;
        info!("Tuscany expansion: {}", if expansion_settings.tuscany_enabled { "ON" } else { "OFF" });
    }
    
    if keyboard.just_pressed(KeyCode::F7) {
        expansion_settings.visitor_cards_enabled = !expansion_settings.visitor_cards_enabled;
        info!("Visitor cards: {}", if expansion_settings.visitor_cards_enabled { "ON" } else { "OFF" });
    }
    
    if keyboard.just_pressed(KeyCode::F8) {
        expansion_settings.advanced_boards_enabled = !expansion_settings.advanced_boards_enabled;
        info!("Advanced boards: {}", if expansion_settings.advanced_boards_enabled { "ON" } else { "OFF" });
    }
}

// Update any field checking functions to use the new structure:
pub fn check_vineyard_capacity(vineyard: &Vineyard) -> usize {
    vineyard.fields.iter().filter(|field| field.vine.is_none()).count()
}

pub fn count_planted_vines(vineyard: &Vineyard) -> usize {
    vineyard.fields.iter().filter(|field| field.vine.is_some()).count()
}

pub fn get_vine_types_planted(vineyard: &Vineyard) -> Vec<VineType> {
    vineyard.fields.iter()
        .filter_map(|field| field.vine)
        .collect()
}

// Enhanced visitor card that works with field types
pub fn create_enhanced_visitor_cards() -> Vec<VisitorCard> {
    vec![
        VisitorCard {
            id: 1010,
            name: "Field Inspector".to_string(),
            effect: VisitorEffect::SwapFields,
            season: VisitorSeason::Summer,
            cost: 1,
        },
        VisitorCard {
            id: 1011,
            name: "Soil Expert".to_string(),
            effect: VisitorEffect::PlantFreeVine, // Can plant on premium fields
            season: VisitorSeason::Summer,
            cost: 0,
        },
        VisitorCard {
            id: 1012,
            name: "Master Gardener".to_string(),
            effect: VisitorEffect::HarvestBonus(3), // More grapes from all fields
            season: VisitorSeason::Winter,
            cost: 2,
        },
    ]
}