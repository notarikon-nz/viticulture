use bevy::prelude::*;
use crate::components::*;
use crate::systems::expansions::*;

// Extended wine order variety for more strategic depth
pub fn create_premium_wine_orders() -> Vec<WineOrderCard> {
    vec![
        // Varietal wines (single grape type, higher value)
        WineOrderCard::new(300, 4, 0, 7, 5),   // Premium Red Varietal
        WineOrderCard::new(301, 0, 4, 7, 5),   // Premium White Varietal
        WineOrderCard::new(302, 5, 0, 9, 6),   // Reserve Red
        WineOrderCard::new(303, 0, 5, 9, 6),   // Reserve White
        
        // Aged wines (require multiple seasons)
        WineOrderCard::new(304, 3, 1, 6, 4),   // Aged Blend
        WineOrderCard::new(305, 2, 3, 8, 5),   // Vintage White Blend
        WineOrderCard::new(306, 6, 0, 12, 8),  // Grand Reserve Red
        WineOrderCard::new(307, 0, 6, 12, 8),  // Grand Reserve White
        
        // Special blends
        WineOrderCard::new(308, 2, 2, 6, 4),   // Classic Blend
        WineOrderCard::new(309, 3, 3, 10, 7),  // Master's Blend
        WineOrderCard::new(310, 4, 4, 15, 10), // Legendary Vintage
        
        // Late season orders (higher risk/reward)
        WineOrderCard::new(311, 7, 1, 15, 9),  // Late Harvest Red
        WineOrderCard::new(312, 1, 7, 15, 9),  // Ice Wine Special
    ]
}

// Advanced vine varieties with special properties
pub fn create_premium_vine_cards() -> Vec<VineCard> {
    vec![
        // High-value vines
        VineCard { 
            id: 400, 
            vine_type: VineType::Red(4), 
            cost: 3,
            art_style: CardArt::PremiumRed,        // ADDED: Missing field
            special_ability: None,                 // ADDED: Missing field
        },
        VineCard { 
            id: 401, 
            vine_type: VineType::White(4), 
            cost: 3,
            art_style: CardArt::PremiumWhite,      // ADDED: Missing field
            special_ability: None,                 // ADDED: Missing field
        },
        VineCard { 
            id: 402, 
            vine_type: VineType::Red(3), 
            cost: 2,
            art_style: CardArt::SpecialtyRed,      // ADDED: Missing field
            special_ability: Some(VineAbility::HighYield), // ADDED: Missing field with ability
        },
        VineCard { 
            id: 403, 
            vine_type: VineType::White(3), 
            cost: 2,
            art_style: CardArt::SpecialtyWhite,    // ADDED: Missing field
            special_ability: Some(VineAbility::HighYield), // ADDED: Missing field with ability
        },
        
        // Specialty vines (with special rules in full implementation)
        VineCard { 
            id: 404, 
            vine_type: VineType::Red(2), 
            cost: 1,
            art_style: CardArt::BasicRed,          // ADDED: Missing field
            special_ability: Some(VineAbility::EarlyHarvest), // ADDED: Missing field with ability
        },
        VineCard { 
            id: 405, 
            vine_type: VineType::White(2), 
            cost: 1,
            art_style: CardArt::BasicWhite,        // ADDED: Missing field
            special_ability: Some(VineAbility::DiseaseResistant), // ADDED: Missing field with ability
        },
        VineCard { 
            id: 406, 
            vine_type: VineType::Red(3), 
            cost: 1,
            art_style: CardArt::SpecialtyRed,      // ADDED: Missing field
            special_ability: Some(VineAbility::DiseaseResistant), // ADDED: Missing field with ability
        },
        VineCard { 
            id: 407, 
            vine_type: VineType::White(3), 
            cost: 1,
            art_style: CardArt::SpecialtyWhite,    // ADDED: Missing field
            special_ability: Some(VineAbility::EarlyHarvest), // ADDED: Missing field with ability
        },
    ]
}

// Advanced structure types for Tuscany expansion
#[derive(Clone, Copy, Debug)]
pub enum AdvancedStructureType {
    // Original structures
    Basic(StructureType),
    
    // Tuscany expansion structures
    Warehouse,     // Store extra wine/grapes
    Laboratory,    // Advanced wine making
    Chapel,        // VP bonus at game end
    Storehouse,    // Draw extra cards
}

impl AdvancedStructureType {
    pub fn cost(&self) -> u8 {
        match self {
            AdvancedStructureType::Basic(s) => match s {
                StructureType::Trellis => 2,
                StructureType::Irrigation => 3,
                StructureType::Yoke => 2,
                StructureType::Windmill => 5,
                StructureType::Cottage => 4,
                StructureType::TastingRoom => 6,
            },
            AdvancedStructureType::Warehouse => 4,
            AdvancedStructureType::Laboratory => 6,
            AdvancedStructureType::Chapel => 3,
            AdvancedStructureType::Storehouse => 3,
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            AdvancedStructureType::Basic(StructureType::Trellis) => "+1 to all vine values when harvesting",
            AdvancedStructureType::Basic(StructureType::Irrigation) => "Plant vines for 1 less lira",
            AdvancedStructureType::Basic(StructureType::Yoke) => "+1 lira when harvesting grapes",
            AdvancedStructureType::Basic(StructureType::Windmill) => "+1 VP for every 7 lira at game end",
            AdvancedStructureType::Basic(StructureType::Cottage) => "Gain 1 additional worker",
            AdvancedStructureType::Basic(StructureType::TastingRoom) => "+1 lira when giving tours",
            AdvancedStructureType::Warehouse => "Store up to 3 extra wine or grapes",
            AdvancedStructureType::Laboratory => "Advanced wine recipes: make 3 wine from 2 grapes",
            AdvancedStructureType::Chapel => "+1 VP at end of game for every 3 structures",
            AdvancedStructureType::Storehouse => "Draw 1 extra card when drawing vine or wine order cards",
        }
    }
}

// Season-specific events for added variety
#[derive(Clone)]
pub struct SeasonEvent {
    pub name: String,
    pub description: String,
    pub effect: SeasonEventEffect,
    pub season: GameState,
}

#[derive(Clone)]
pub enum SeasonEventEffect {
    GlobalBonus(String),      // Affects all players
    FirstPlayerBonus(String), // Affects first player in turn order
    WeatherEffect(String),    // Affects harvest/planting
    MarketEvent(String),      // Affects wine orders/prices
}

pub fn create_season_events() -> Vec<SeasonEvent> {
    vec![
        SeasonEvent {
            name: "Perfect Weather".to_string(),
            description: "Ideal growing conditions this season".to_string(),
            effect: SeasonEventEffect::GlobalBonus("All vines produce +1 grape when harvested".to_string()),
            season: GameState::Summer,
        },
        SeasonEvent {
            name: "Wine Festival".to_string(),
            description: "Local celebration increases demand".to_string(),
            effect: SeasonEventEffect::GlobalBonus("All wine orders pay +1 lira".to_string()),
            season: GameState::Winter,
        },
        SeasonEvent {
            name: "Early Frost".to_string(),
            description: "Unexpected cold weather affects harvest".to_string(),
            effect: SeasonEventEffect::WeatherEffect("Each player loses 1 grape of each type".to_string()),
            season: GameState::Fall,
        },
        SeasonEvent {
            name: "Merchant Visit".to_string(),
            description: "Traveling merchant offers good prices".to_string(),
            effect: SeasonEventEffect::FirstPlayerBonus("First player gains 3 lira".to_string()),
            season: GameState::Spring,
        },
        SeasonEvent {
            name: "Wine Competition".to_string(),
            description: "Regional wine competition".to_string(),
            effect: SeasonEventEffect::MarketEvent("New high-value wine orders available".to_string()),
            season: GameState::Winter,
        },
    ]
}

// Enhanced player powers for asymmetric gameplay
#[derive(Clone, Copy, Debug)]
pub enum PlayerPower {
    None,
    EfficientViticulturist, // Plant vines for free once per year
    MasterVintner,          // Make wine more efficiently
    BusinessMogul,          // Extra lira from tours and sales
    TechnicalInnovator,     // Build structures cheaper
    WineConnoisseur,        // Draw extra wine order cards
    FieldMaster,            // Extra vineyard fields
}

impl PlayerPower {
    pub fn description(&self) -> &'static str {
        match self {
            PlayerPower::None => "No special ability",
            PlayerPower::EfficientViticulturist => "Once per year, plant a vine for free",
            PlayerPower::MasterVintner => "When making wine, produce 1 extra wine",
            PlayerPower::BusinessMogul => "Gain +1 lira from tours and selling grapes",
            PlayerPower::TechnicalInnovator => "Build all structures for 1 less lira",
            PlayerPower::WineConnoisseur => "Draw 1 extra card when drawing wine orders",
            PlayerPower::FieldMaster => "Start with 1 extra vineyard field (10 total)",
        }
    }
    
    pub fn apply_effect(&self, action: ActionSpace, base_result: &mut ActionResult) {
        match (self, action) {
            (PlayerPower::MasterVintner, ActionSpace::MakeWine) => {
                base_result.bonus_wine += 1;
            }
            (PlayerPower::BusinessMogul, ActionSpace::GiveTour) => {
                base_result.bonus_lira += 1;
            }
            (PlayerPower::BusinessMogul, ActionSpace::SellGrapes) => {
                base_result.bonus_lira += 1;
            }
            (PlayerPower::WineConnoisseur, ActionSpace::DrawWineOrder) => {
                base_result.bonus_cards += 1;
            }
            _ => {} // No effect for this combination
        }
    }
}

#[derive(Default)]
pub struct ActionResult {
    pub bonus_lira: u8,
    pub bonus_wine: u8,
    pub bonus_cards: u8,
    pub bonus_vp: u8,
}

// Dynamic difficulty scaling based on player performance
#[derive(Resource, Default)]
pub struct DifficultyScaling {
    pub base_difficulty: f32,
    pub performance_modifier: f32,
    pub game_length_modifier: f32,
}

impl DifficultyScaling {
    pub fn adjust_for_performance(&mut self, games_won: u32, games_played: u32) {
        if games_played < 3 {
            return; // Not enough data
        }
        
        let win_rate = games_won as f32 / games_played as f32;
        
        // Adjust difficulty based on win rate
        self.performance_modifier = match win_rate {
            r if r > 0.7 => 0.2,   // Winning too much, increase difficulty
            r if r > 0.5 => 0.0,   // Good balance
            r if r > 0.3 => -0.1,  // Struggling, decrease difficulty
            _ => -0.2,             // Losing too much, significant decrease
        };
    }
    
    pub fn get_effective_difficulty(&self) -> f32 {
        (self.base_difficulty + self.performance_modifier + self.game_length_modifier).clamp(0.1, 2.0)
    }
}

// Advanced AI personalities for more varied gameplay
#[derive(Clone, Copy, Debug)]
pub enum AIPersonality {
    Aggressive,   // Focuses on VP, takes risks
    Conservative, // Steady economy, safe plays
    Opportunist,  // Adapts to game state
    Specialist,   // Focuses on specific strategy
}

impl AIPersonality {
    pub fn modify_action_score(&self, action: ActionSpace, base_score: f32, game_context: &GameContext) -> f32 {
        match self {
            AIPersonality::Aggressive => {
                match action {
                    ActionSpace::FillOrder => base_score * 1.5, // Prioritize VP
                    ActionSpace::TrainWorker => base_score * 1.2, // Expand faster
                    ActionSpace::GiveTour => base_score * 0.8, // Less focus on safe income
                    _ => base_score,
                }
            }
            AIPersonality::Conservative => {
                match action {
                    ActionSpace::GiveTour => base_score * 1.3, // Prioritize safe income
                    ActionSpace::BuildStructure => base_score * 1.2, // Long-term benefits
                    ActionSpace::FillOrder => base_score * 0.9, // Less risky
                    _ => base_score,
                }
            }
            AIPersonality::Opportunist => {
                // Adjust based on current resources and opportunities
                if game_context.lira < 3 {
                    match action {
                        ActionSpace::GiveTour | ActionSpace::SellGrapes => base_score * 1.4,
                        _ => base_score,
                    }
                } else {
                    base_score
                }
            }
            AIPersonality::Specialist => {
                // Focus on wine production chain
                match action {
                    ActionSpace::PlantVine | ActionSpace::Harvest | ActionSpace::MakeWine => base_score * 1.3,
                    _ => base_score * 0.9,
                }
            }
        }
    }
}

#[derive(Default)]
pub struct GameContext {
    pub lira: u8,
    pub vp: u8,
    pub year: u8,
    pub wine_count: u8,
    pub grape_count: u8,
}

// Expansion content manager
#[derive(Resource, Default, Clone)]
pub struct ExpansionContent {
    pub premium_wine_orders: Vec<WineOrderCard>,
    pub premium_vine_cards: Vec<VineCard>,
    pub season_events: Vec<SeasonEvent>,
    pub current_event: Option<SeasonEvent>,
}

pub fn initialize_expansion_content_system(
    mut commands: Commands,
) {
    let content = ExpansionContent {
        premium_wine_orders: create_premium_wine_orders(),
        premium_vine_cards: create_premium_vine_cards(),
        season_events: create_season_events(),
        current_event: None,
    };
    
    commands.insert_resource(content);
    commands.insert_resource(DifficultyScaling::default());
}

pub fn trigger_season_event_system(
    mut expansion_content: ResMut<ExpansionContent>,
    current_state: Res<State<GameState>>,
    mut players: Query<&mut Player>,
    mut vineyards: Query<&mut Vineyard>,
    expansion_settings: Res<ExpansionSettings>,
) {
    if !expansion_settings.tuscany_enabled {
        return;
    }
    
    // 20% chance of event each season change
    use rand::Rng;
    let mut rng = rand::rng();
    
    if current_state.is_changed() && rng.random_bool(0.2) {
        let matching_events: Vec<_> = expansion_content.season_events.iter()
            .filter(|event| event.season == *current_state.get())
            .collect();
        
        let mut expansion_content_clone = expansion_content.clone();
        if !matching_events.is_empty() {
            let random_index = rng.random_range(0..matching_events.len());
            let event = &matching_events[random_index]; // Use indexing instead of choose
            expansion_content_clone.current_event = Some((*event).clone());
            apply_season_event_effect(&event.effect, &mut players, &mut vineyards);
            info!("Season Event: {} - {}", event.name, event.description);
        }
    }
}

fn apply_season_event_effect(
    effect: &SeasonEventEffect,
    players: &mut Query<&mut Player>,
    vineyards: &mut Query<&mut Vineyard>,
) {
    match effect {
        SeasonEventEffect::GlobalBonus(description) => {
            info!("Global bonus: {}", description);
            // Apply to all players - implementation depends on specific effect
        }
        SeasonEventEffect::FirstPlayerBonus(description) => {
            info!("First player bonus: {}", description);
            if let Some(mut first_player) = players.iter_mut().next() {
                first_player.gain_lira(3); // Example effect
            }
        }
        SeasonEventEffect::WeatherEffect(description) => {
            info!("Weather effect: {}", description);
            for mut vineyard in vineyards.iter_mut() {
                vineyard.red_grapes = vineyard.red_grapes.saturating_sub(1);
                vineyard.white_grapes = vineyard.white_grapes.saturating_sub(1);
            }
        }
        SeasonEventEffect::MarketEvent(description) => {
            info!("Market event: {}", description);
            // Would add special wine orders to the market
        }
    }
}
