// src/components.rs - Updated with fixes and improvements

use bevy::prelude::*;
use crate::systems::*;

// UI Text Preservation
#[derive(Component)]
pub struct PhaseText;

#[derive(Component)]
pub struct GameStatusText;

#[derive(Component)]
pub struct ButtonText;

// 
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Setup,
    Spring,
    Summer,
    Fall,
    Winter,
    GameOver,
}

#[derive(Resource, Default)]
pub struct TurnOrder {
    pub players: Vec<PlayerId>,
    pub current_player: usize,
    pub wake_up_order: Vec<(PlayerId, u8)>,
    pub wake_up_bonuses: Vec<WakeUpBonus>,
}

#[derive(Clone, Copy)]
pub enum WakeUpBonus {
    DrawVineCard,
    GainLira(u8),
    GainVictoryPoint,
    DrawWineOrderCard,
    PlayExtraWorker,
}

impl TurnOrder {
    pub fn set_wake_up_order(&mut self, mut order: Vec<(PlayerId, u8)>) {
        order.sort_by(|a, b| a.1.cmp(&b.1));
        self.wake_up_order = order;
        self.players.clear();
        for (player_id, _) in &self.wake_up_order {
            self.players.push(*player_id);
        }
    }
    
    pub fn get_wake_up_bonus(&self, player_id: PlayerId) -> Option<WakeUpBonus> {
        if let Some(position) = self.wake_up_order.iter().position(|(id, _)| *id == player_id) {
            match position {
                0 => Some(WakeUpBonus::DrawVineCard),
                1 => Some(WakeUpBonus::GainLira(1)),
                2 => None,
                3 => Some(WakeUpBonus::GainLira(1)),
                4 => Some(WakeUpBonus::DrawWineOrderCard),
                _ => Some(WakeUpBonus::GainVictoryPoint),
            }
        } else {
            None
        }
    }
}

#[derive(Resource)]
pub struct GameConfig {
    pub player_count: u8,
    pub target_victory_points: u8,
    pub current_year: u8,
    pub max_years: u8,
    pub ai_count: u8, // New: track AI players separately
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            player_count: 2,
            target_victory_points: 20,
            current_year: 1,
            max_years: 7,
            ai_count: 1, // Default to 1 AI opponent
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u8);

#[derive(Component)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub victory_points: u8,
    pub lira: u8,
    pub workers: u8,
    pub grande_worker_available: bool,
    pub is_ai: bool, // New: track if player is AI
}

impl Player {
    pub fn new(id: u8, name: String, is_ai: bool) -> Self {
        Self {
            id: PlayerId(id),
            name,
            victory_points: 0,
            lira: 3,
            workers: 2, // Base workers (not counting grande)
            grande_worker_available: true,
            is_ai,
        }
    }
    
    pub fn gain_victory_points(&mut self, points: u8) {
        self.victory_points = self.victory_points.saturating_add(points);
    }
    
    pub fn gain_lira(&mut self, amount: u8) {
        self.lira = self.lira.saturating_add(amount);
    }
    
    // New: get total worker count (including grande)
    pub fn total_workers(&self) -> u8 {
        self.workers + if self.grande_worker_available { 1 } else { 0 }
    }
}

// Enhanced vineyard with better field representation
#[derive(Component)]
pub struct Vineyard {
    pub owner: PlayerId,
    pub fields: [VineyardField; 9],
    pub red_grapes: u8,
    pub white_grapes: u8,
    pub red_wine: u8,
    pub white_wine: u8,
    pub lira: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct VineyardField {
    pub vine: Option<VineType>,

    pub field_type: FieldType,
    pub sold_this_year: bool, // Track if sold grapes this year
}

#[derive(Clone, Copy, Debug)]
pub enum FieldType {
    Standard,
    Premium, // +1 bonus to vine value
    Poor,    // -1 to vine value (minimum 1)
}

#[derive(Clone, Copy, Debug)]
pub enum WineType {
    Red,
    White,
    Blush,
    Sparkling,
}

impl VineyardField {
    pub fn new(field_type: FieldType) -> Self {
        Self {
            vine: None,
            field_type,
            sold_this_year: false,
        }
    }
    
    pub fn get_harvest_value(&self) -> u8 {
        if let Some(vine) = self.vine {
            let base_value = match vine {
                VineType::Red(v) | VineType::White(v) => v,
            };
            
            match self.field_type {
                FieldType::Premium => base_value + 1,
                FieldType::Poor => base_value.saturating_sub(1).max(1),
                FieldType::Standard => base_value,
            }
        } else {
            0
        }
    }
    
    pub fn can_sell(&self) -> bool {
        self.vine.is_none() // Can only sell empty fields
    }
    
    pub fn sell_value(&self) -> u8 {
        match self.field_type {
            FieldType::Standard => 1,
            FieldType::Premium => 2,
            FieldType::Poor => 1,
        }
    }

    // Helper methods for easier checking
    pub fn is_empty(&self) -> bool {
        self.vine.is_none()
    }
    
    pub fn has_vine(&self) -> bool {
        self.vine.is_some()
    }
    
    pub fn plant_vine(&mut self, vine_type: VineType) {
        self.vine = Some(vine_type);
    }

    pub fn can_plant_vine(&self, vine_card: &VineCard, current_total: u8, max_value: u8) -> bool {
        if self.vine.is_some() {
            return false; // Field already has a vine
        }
        
        let vine_value = match vine_card.vine_type {
            VineType::Red(v) | VineType::White(v) => v,
        };
        
        current_total + vine_value <= max_value
    }    
}

impl Vineyard {
    pub fn new(owner: PlayerId) -> Self {
        // Create varied field types for more interesting gameplay
        let mut fields = [VineyardField::new(FieldType::Standard); 9];
        fields[1] = VineyardField::new(FieldType::Premium); // One premium field
        fields[7] = VineyardField::new(FieldType::Poor);    // One poor field
        
        Self {
            owner,
            fields,
            red_grapes: 0,
            white_grapes: 0,
            red_wine: 0,
            white_wine: 0,
            lira: 3,
        }
    }
    
    fn get_field_total_value(&self, field_index: usize) -> u8 {
        if field_index >= self.fields.len() {
            return 0;
        }
        
        // In Viticulture, multiple vines can be planted on one field (stacked)
        // We need to track this. For now, modify VineyardField to support stacking:
        
        match &self.fields[field_index].vine {
            Some(vine) => match vine {
                VineType::Red(value) | VineType::White(value) => *value,
            },
            None => 0,
        }
    }
    
    // Helper to calculate total harvest from a field
    pub fn get_field_harvest_values(&self, field_index: usize) -> (u8, u8) {
        if field_index >= self.fields.len() {
            return (0, 0);
        }
        
        match &self.fields[field_index].vine {
            Some(VineType::Red(value)) => (*value, 0),
            Some(VineType::White(value)) => (0, *value),
            None => (0, 0),
        }
    }

    
    pub fn can_plant_vine(&self, field_index: usize, vine_card: &VineCard, structures: &[Structure]) -> bool {
        if field_index >= 9 || self.fields[field_index].vine.is_some() {
            return false;
        }
        
        let mut cost = vine_card.cost;
        if structures.iter().any(|s| matches!(s.structure_type, StructureType::Irrigation) && s.owner == self.owner) {
            cost = cost.saturating_sub(1);
        }
        
        self.lira >= cost
    }

    pub fn can_plant_vine_with_requirements(&self, field_index: usize, vine_card: &VineCard, structures: &[Structure]) -> bool {
        if field_index >= 9 || self.fields[field_index].vine.is_some() {
            return false;
        }
        
        let requirements = vine_card.requirements();
        let has_trellis = structures.iter().any(|s| s.owner == self.owner && matches!(s.structure_type, StructureType::Trellis));
        let has_irrigation = structures.iter().any(|s| s.owner == self.owner && matches!(s.structure_type, StructureType::Irrigation));
        
        if requirements.needs_trellis && !has_trellis {
            return false;
        }
        if requirements.needs_irrigation && !has_irrigation {
            return false;
        }
        
        // Check lira cost
        self.lira >= vine_card.cost
    }    
    
    pub fn plant_vine(&mut self, field_index: usize, vine_card: VineCard, structures: &[Structure]) -> bool {
        if self.can_plant_vine(field_index, &vine_card, structures) {
            let mut cost = vine_card.cost;
            if structures.iter().any(|s| matches!(s.structure_type, StructureType::Irrigation) && s.owner == self.owner) {
                cost = cost.saturating_sub(1);
            }
            
            self.fields[field_index].vine = Some(vine_card.vine_type);
            self.lira = self.lira.saturating_sub(cost);
            true
        } else {
            false
        }
    }
    
    pub fn harvest_grapes(&mut self, structures: &[Structure]) -> u8 {
        let mut total_gained = 0;
        
        for field in &mut self.fields {
            let harvest_value = field.get_harvest_value();
            if harvest_value > 0 {
                let mut final_value = harvest_value;
                
                // Trellis structure bonus
                if structures.iter().any(|s| matches!(s.structure_type, StructureType::Trellis) && s.owner == self.owner) {
                    final_value += 1;
                }
                
                if let Some(vine) = field.vine {
                    match vine {
                        VineType::Red(_) => {
                            self.red_grapes += final_value;
                            total_gained += final_value;
                        },
                        VineType::White(_) => {
                            self.white_grapes += final_value;
                            total_gained += final_value;
                        },
                    }
                }
            }
        }
        
        // Yoke structure bonus
        if structures.iter().any(|s| matches!(s.structure_type, StructureType::Yoke) && s.owner == self.owner) {
            if total_gained > 0 {
                self.lira += 1;
            }
        }
        
        total_gained
    }
    
    pub fn make_wine(&mut self, red_grapes_used: u8, white_grapes_used: u8) -> bool {
        if self.red_grapes >= red_grapes_used && self.white_grapes >= white_grapes_used {
            self.red_grapes -= red_grapes_used;
            self.white_grapes -= white_grapes_used;
            self.red_wine += red_grapes_used;
            self.white_wine += white_grapes_used;
            true
        } else {
            false
        }
    }
    
    pub fn can_make_wine(&self, wine_type: WineType, value: u8, structures: &[Structure]) -> bool {
        let has_medium = structures.iter().any(|s| matches!(s.structure_type, StructureType::Cottage)); // Should be Medium Cellar
        let has_large = structures.iter().any(|s| matches!(s.structure_type, StructureType::Windmill)); // Should be Large Cellar
        
        match wine_type {
            WineType::Red | WineType::White => {
                if value <= 3 { true }
                else if value <= 6 && has_medium { true }
                else if value <= 9 && has_medium && has_large { true }
                else { false }
            }
            WineType::Blush => has_medium && value >= 4,
            WineType::Sparkling => has_large && value >= 7,
        }
    }

    pub fn can_fulfill_order(&self, order: &WineOrderCard) -> bool {
        self.red_wine >= order.red_wine_needed && self.white_wine >= order.white_wine_needed
    }
    
    pub fn fulfill_order(&mut self, order: &WineOrderCard) -> bool {
        if self.can_fulfill_order(order) {
            self.red_wine -= order.red_wine_needed;
            self.white_wine -= order.white_wine_needed;
            self.lira += order.payout;
            true
        } else {
            false
        }
    }

    pub fn can_build_structure(&self, structure_type: StructureType) -> bool {
        let cost = match structure_type {
            StructureType::Trellis => 2,
            StructureType::Irrigation => 3,
            StructureType::Yoke => 2,
            StructureType::MediumCellar => 4,
            StructureType::LargeCellar => 6,
            StructureType::Windmill => 5,
            StructureType::Cottage => 4,
            StructureType::TastingRoom => 6,
        };
        self.lira >= cost
    }
    
    pub fn build_structure(&mut self, structure_type: StructureType) -> bool {
        if self.can_build_structure(structure_type) {
            let cost = match structure_type {
                StructureType::Trellis => 2,
                StructureType::Irrigation => 3,
                StructureType::Yoke => 2,
                StructureType::MediumCellar => 4,
                StructureType::LargeCellar => 6,
                StructureType::Windmill => 5,
                StructureType::Cottage => 4,
                StructureType::TastingRoom => 6,
            };
            self.lira = self.lira.saturating_sub(cost);
            true
        } else {
            false
        }
    }
    
    pub fn get_end_game_bonus(&self, structures: &[Structure]) -> u8 {
        let mut bonus = 0;
        
        if structures.iter().any(|s| matches!(s.structure_type, StructureType::Windmill) && s.owner == self.owner) {
            bonus += self.lira / 7;
        }
        
        bonus
    }

    pub fn sell_field(&mut self, field_index: usize) -> Option<u8> {
        if field_index >= 9 {
            return None;
        }
        
        let field = &mut self.fields[field_index];
        if field.can_sell() {
            let value = field.sell_value();
            field.sold_this_year = true;
            self.lira += value;
            Some(value)
        } else {
            None
        }
    }
    
    pub fn buy_back_field(&mut self, field_index: usize) -> bool {
        if field_index >= 9 {
            return false;
        }
        
        let field = &self.fields[field_index];
        if field.sold_this_year {
            let cost = field.sell_value();
            if self.lira >= cost {
                self.lira -= cost;
                self.fields[field_index].sold_this_year = false;
                return true;
            }
        }
        false
    }
    
    pub fn available_fields(&self) -> Vec<usize> {
        self.fields.iter()
            .enumerate()
            .filter(|(_, field)| field.vine.is_none() && !field.sold_this_year)
            .map(|(i, _)| i)
            .collect()
    }    
}

#[derive(Clone, Copy, Debug)]
pub enum VineType {
    Red(u8),
    White(u8),
}

// Enhanced card representation with better art data
#[derive(Component, Clone)]
pub struct VineCard {
    pub id: u32,
    pub vine_type: VineType,
    pub cost: u8,
    pub art_style: CardArt,
    pub special_ability: Option<VineAbility>, // New: special vine abilities
}

#[derive(Clone, Copy, Debug)]
pub struct VineRequirements {
    pub needs_trellis: bool,
    pub needs_irrigation: bool,
}

impl VineCard {
    pub fn requirements(&self) -> VineRequirements {
        match (self.vine_type, self.cost) {
            // High-value vines need structures
            (VineType::Red(4) | VineType::White(4), _) => VineRequirements { 
                needs_trellis: true, 
                needs_irrigation: true 
            },
            (VineType::Red(3) | VineType::White(3), _) => VineRequirements { 
                needs_trellis: true, 
                needs_irrigation: false 
            },
            // Special high-cost vines need irrigation
            (_, cost) if cost >= 3 => VineRequirements { 
                needs_trellis: false, 
                needs_irrigation: true 
            },
            _ => VineRequirements { 
                needs_trellis: false, 
                needs_irrigation: false 
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum VineAbility {
    EarlyHarvest,    // Can harvest in summer
    DiseaseResistant, // Immune to negative events
    HighYield,       // +1 grape when harvesting
}

#[derive(Clone, Copy, Debug)]
pub enum CardArt {
    BasicRed,
    BasicWhite,
    PremiumRed,
    PremiumWhite,
    SpecialtyRed,
    SpecialtyWhite,
}

impl CardArt {
    pub fn get_color(&self) -> Color {
        match self {
            CardArt::BasicRed => Color::srgb(0.6, 0.2, 0.2),
            CardArt::BasicWhite => Color::srgb(0.9, 0.9, 0.7),
            CardArt::PremiumRed => Color::srgb(0.8, 0.1, 0.1),
            CardArt::PremiumWhite => Color::srgb(1.0, 1.0, 0.8),
            CardArt::SpecialtyRed => Color::srgb(0.9, 0.3, 0.1),
            CardArt::SpecialtyWhite => Color::srgb(0.95, 0.95, 0.9),
        }
    }
    
    pub fn get_border_color(&self) -> Color {
        match self {
            CardArt::BasicRed | CardArt::PremiumRed | CardArt::SpecialtyRed => Color::srgb(0.3, 0.1, 0.1),
            CardArt::BasicWhite | CardArt::PremiumWhite | CardArt::SpecialtyWhite => Color::srgb(0.7, 0.7, 0.5),
        }
    }
}

#[derive(Component, Clone)]
pub struct WineOrderCard {
    pub id: u32,
    pub red_wine_needed: u8,
    pub white_wine_needed: u8,
    pub victory_points: u8,
    pub payout: u8,
    pub art_style: OrderArt,
    pub order_type: OrderType, // New: different order types
    pub residual_payment: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum OrderType {
    Regular,
    Premium,  // Higher VP, harder requirements
    Seasonal, // Special seasonal bonuses
}

#[derive(Clone, Copy, Debug)]
pub enum OrderArt {
    BasicOrder,
    PremiumOrder,
    SeasonalOrder,
}

impl OrderArt {
    pub fn get_color(&self) -> Color {
        match self {
            OrderArt::BasicOrder => Color::srgb(0.4, 0.2, 0.6),
            OrderArt::PremiumOrder => Color::srgb(0.6, 0.3, 0.8),
            OrderArt::SeasonalOrder => Color::srgb(0.8, 0.5, 0.2),
        }
    }
    
    pub fn get_border_color(&self) -> Color {
        match self {
            OrderArt::BasicOrder => Color::srgb(0.2, 0.1, 0.3),
            OrderArt::PremiumOrder => Color::srgb(0.3, 0.15, 0.4),
            OrderArt::SeasonalOrder => Color::srgb(0.4, 0.25, 0.1),
        }
    }
}

impl WineOrderCard {
    pub fn new(id: u32, red: u8, white: u8, vp: u8, payout: u8) -> Self {
        let art_style = if vp >= 5 {
            OrderArt::PremiumOrder
        } else if id % 4 == 0 {
            OrderArt::SeasonalOrder
        } else {
            OrderArt::BasicOrder
        };
        
        let order_type = match vp {
            0..=2 => OrderType::Regular,
            3..=5 => OrderType::Premium,
            _ => OrderType::Seasonal,
        };
        
        Self {
            id,
            red_wine_needed: red,
            white_wine_needed: white,
            victory_points: vp,
            payout,
            art_style,
            order_type,
            residual_payment: 0,
        }
    }

    pub fn new_with_residual(id: u32, red: u8, white: u8, vp: u8, payout: u8, residual: u8) -> Self {
        Self {
            id,
            red_wine_needed: red,
            white_wine_needed: white,
            victory_points: vp,
            payout,
            art_style: if vp >= 5 { OrderArt::PremiumOrder } else { OrderArt::BasicOrder },
            order_type: if vp >= 5 { OrderType::Premium } else { OrderType::Regular },
            residual_payment: residual, // New field
        }
    }

    pub fn immediate_payout(&self) -> u8 {
        self.payout
    }
    
    pub fn residual_payment(&self) -> u8 {
        self.residual_payment
    }    
}

#[derive(Component)]
pub struct Worker {
    pub owner: PlayerId,
    pub is_grande: bool,
    pub placed_at: Option<ActionSpace>,
    pub position: Vec2,
}

impl Worker {
    pub fn new(owner: PlayerId, is_grande: bool, position: Vec2) -> Self {
        Self {
            owner,
            is_grande,
            placed_at: None,
            position,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionSpace {
    // Summer actions
    DrawVine,
    PlantVine,
    BuildStructure,
    GiveTour,
    SellGrapes,
    
    // Winter actions
    DrawWineOrder,
    Harvest,
    MakeWine,
    FillOrder,
    TrainWorker,
}

#[derive(Component)]
pub struct ActionBoard {
    pub spaces: Vec<ActionSpaceSlot>,
}

#[derive(Component, Clone)]
pub struct ActionSpaceSlot {
    pub action: ActionSpace,
    pub occupied_by: Option<PlayerId>,
    pub bonus_worker_slot: Option<PlayerId>,
    pub position: Vec2,
    pub is_summer: bool,
    pub has_bonus_slot: bool, // New: some spaces have bonus slots
}

#[derive(Component)]
pub struct Clickable {
    pub size: Vec2,
}

impl ActionSpaceSlot {
    pub fn new(action: ActionSpace, position: Vec2, is_summer: bool, has_bonus_slot: bool) -> Self {
        Self {
            action,
            occupied_by: None,
            bonus_worker_slot: None,
            position,
            is_summer,
            has_bonus_slot,
        }
    }
    
    pub fn can_place_worker(&self, _player_id: PlayerId, current_state: &GameState) -> bool {
        if self.occupied_by.is_some() {
            return false;
        }
        
        match current_state {
            GameState::Summer => self.is_summer,
            GameState::Winter => !self.is_summer,
            _ => false,
        }
    }
    
    pub fn can_place_grande_worker(&self, _player_id: PlayerId, current_state: &GameState) -> bool {
        let right_season = match current_state {
            GameState::Summer => self.is_summer,
            GameState::Winter => !self.is_summer,
            _ => false,
        };
        
        right_season && (self.occupied_by.is_none() || (self.has_bonus_slot && self.bonus_worker_slot.is_none()))
    }

    pub fn is_available_for_player_count(&self, player_count: u8, position: usize) -> bool {
        match player_count {
            1..=2 => position == 0, // Only leftmost space
            3..=4 => position <= 1,  // Left and middle spaces
            5..=6 => position <= 2,  // All three spaces
            _ => true,
        }
    }

    pub fn place_grande_on_occupied(&mut self, player_id: PlayerId) -> bool {
        // Grande worker can be placed even if space is occupied
        if self.occupied_by.is_some() {
            // Place on the action art/center, not on a specific slot
            true
        } else {
            // Place normally if space is free
            self.occupied_by = Some(player_id);
            true
        }
    }
    
    pub fn has_grande_worker(&self, player_id: PlayerId) -> bool {
        // Check if this player has a grande worker here
        // In the actual game, we'd track this separately
        self.bonus_worker_slot == Some(player_id)
    }    
}

impl ActionBoard {
    pub fn new() -> Self {
        let mut spaces = Vec::new();
        
        // Summer actions (left side) - some with bonus slots
        spaces.push(ActionSpaceSlot::new(ActionSpace::DrawVine, Vec2::new(-300.0, 100.0), true, false));
        spaces.push(ActionSpaceSlot::new(ActionSpace::PlantVine, Vec2::new(-300.0, 50.0), true, true)); // Has bonus
        spaces.push(ActionSpaceSlot::new(ActionSpace::BuildStructure, Vec2::new(-300.0, 0.0), true, false));
        spaces.push(ActionSpaceSlot::new(ActionSpace::GiveTour, Vec2::new(-300.0, -50.0), true, true)); // Has bonus
        spaces.push(ActionSpaceSlot::new(ActionSpace::SellGrapes, Vec2::new(-300.0, -100.0), true, false));
        
        // Winter actions (right side) - some with bonus slots
        spaces.push(ActionSpaceSlot::new(ActionSpace::DrawWineOrder, Vec2::new(300.0, 100.0), false, false));
        spaces.push(ActionSpaceSlot::new(ActionSpace::Harvest, Vec2::new(300.0, 50.0), false, true)); // Has bonus
        spaces.push(ActionSpaceSlot::new(ActionSpace::MakeWine, Vec2::new(300.0, 0.0), false, true)); // Has bonus
        spaces.push(ActionSpaceSlot::new(ActionSpace::FillOrder, Vec2::new(300.0, -50.0), false, false));
        spaces.push(ActionSpaceSlot::new(ActionSpace::TrainWorker, Vec2::new(300.0, -100.0), false, false));
        
        Self { spaces }
    }
}

// Mama & Papa Cards - Essential for game variety
#[derive(Component, Clone)]
pub struct MamaCard {
    pub id: u8,
    pub name: String,
    pub bonus_lira: u8,
    pub bonus_workers: u8,
    pub bonus_vine_cards: u8,
    pub special_ability: Option<MamaAbility>,
}

#[derive(Component, Clone)]
pub struct PapaCard {
    pub id: u8,
    pub name: String,
    pub bonus_vp: u8,
    pub starting_structures: Vec<StructureType>,
    pub bonus_fields: u8,
    pub special_ability: Option<PapaAbility>,
}

#[derive(Clone, Debug)]
pub enum MamaAbility {
    ExtraBonusAction,    // Can take one extra action per year
    DiscountedStructures, // All structures cost 1 less
    BonusHarvest,        // +1 grape when harvesting
    FreeVinePlanting,    // Plant first vine each year for free
}

#[derive(Clone, Debug)]
pub enum PapaAbility {
    ExtraVineyardField,  // Start with extra field
    AdvancedCellar,      // Can store extra wine
    TradingConnections,  // Better wine order prices
    WineExpertise,       // Make blush wine more efficiently
}

// Residual income system
#[derive(Component)]
pub struct ResidualIncome {
    pub owner: PlayerId,
    pub amount: u8,
    pub source: String,
}

#[derive(Resource, Clone)]
pub struct CardDecks {
    pub vine_deck: Vec<VineCard>,
    pub wine_order_deck: Vec<WineOrderCard>,
    pub vine_discard: Vec<VineCard>,
    pub wine_order_discard: Vec<WineOrderCard>,
    pub mama_cards: Vec<MamaCard>,
    pub papa_cards: Vec<PapaCard>,
}

impl CardDecks {
    pub fn new() -> Self {
        let mut vine_deck = Vec::new();
        let mut wine_order_deck = Vec::new();
        
        // Create varied vine cards with better art
        for i in 0..30 {
            let vine_type = if i % 2 == 0 { 
                VineType::Red(2 + (i % 3) as u8) 
            } else { 
                VineType::White(2 + (i % 3) as u8) 
            };
            
            let art_style = match (i % 6, &vine_type) {
                (0..=1, VineType::Red(_)) => CardArt::BasicRed,
                (2..=3, VineType::Red(_)) => CardArt::PremiumRed,
                (4..=5, VineType::Red(_)) => CardArt::SpecialtyRed,
                (0..=1, VineType::White(_)) => CardArt::BasicWhite,
                (2..=3, VineType::White(_)) => CardArt::PremiumWhite,
                (_, VineType::White(_)) => CardArt::SpecialtyWhite,
                _ => CardArt::BasicRed,
            };
            
            vine_deck.push(VineCard {
                id: i,
                vine_type,
                cost: 1 + (i % 3) as u8,
                art_style,
                special_ability: if i % 10 == 0 { Some(VineAbility::HighYield) } else { None },
            });
        }
        
        // Create enhanced wine orders
        let wine_orders = [
            (100, 1, 0, 1, 1), (101, 0, 1, 1, 1), (102, 2, 0, 2, 2), (103, 0, 2, 2, 2),
            (104, 1, 1, 2, 2), (105, 3, 0, 4, 3), (106, 0, 3, 4, 3), (107, 2, 2, 5, 4),
            (108, 4, 0, 6, 5), (109, 0, 4, 6, 5), (110, 3, 2, 7, 6), (111, 2, 3, 7, 6),
            (112, 1, 2, 3, 3), (113, 2, 1, 3, 3), (114, 3, 1, 5, 4), (115, 1, 3, 5, 4),
            // Blush wine orders (mixed requirements)
            (200, 1, 1, 3, 3), (201, 2, 1, 4, 4), (202, 1, 2, 4, 4), (203, 2, 2, 6, 5),
            // Premium orders
            (300, 4, 2, 8, 6), (301, 2, 4, 8, 6), (302, 5, 1, 9, 7), (303, 1, 5, 9, 7),
        ];
        
        for (id, red, white, vp, payout) in wine_orders {
            wine_order_deck.push(WineOrderCard::new(id, red, white, vp, payout));
        }
        
        Self {
            vine_deck,
            wine_order_deck,
            vine_discard: Vec::new(),
            wine_order_discard: Vec::new(),
            mama_cards: Self::create_mama_cards(),
            papa_cards: Self::create_papa_cards(),
        }
    }
    
    pub fn draw_vine_card(&mut self) -> Option<VineCard> {
        self.vine_deck.pop()
    }
    
    pub fn draw_wine_order_card(&mut self) -> Option<WineOrderCard> {
        self.wine_order_deck.pop()
    }
    
    fn create_mama_cards() -> Vec<MamaCard> {
        vec![
            MamaCard {
                id: 0,
                name: "Wealthy Widow".to_string(),
                bonus_lira: 4,
                bonus_workers: 0,
                bonus_vine_cards: 1,
                special_ability: None,
            },
            MamaCard {
                id: 1,
                name: "Industrious Organizer".to_string(),
                bonus_lira: 2,
                bonus_workers: 1,
                bonus_vine_cards: 0,
                special_ability: Some(MamaAbility::ExtraBonusAction),
            },
            MamaCard {
                id: 2,
                name: "Frugal Builder".to_string(),
                bonus_lira: 1,
                bonus_workers: 0,
                bonus_vine_cards: 2,
                special_ability: Some(MamaAbility::DiscountedStructures),
            },
            MamaCard {
                id: 3,
                name: "Harvest Expert".to_string(),
                bonus_lira: 3,
                bonus_workers: 0,
                bonus_vine_cards: 0,
                special_ability: Some(MamaAbility::BonusHarvest),
            },
            MamaCard {
                id: 4,
                name: "Vine Specialist".to_string(),
                bonus_lira: 2,
                bonus_workers: 0,
                bonus_vine_cards: 1,
                special_ability: Some(MamaAbility::FreeVinePlanting),
            },
        ]
    }
    
    fn create_papa_cards() -> Vec<PapaCard> {
        vec![
            PapaCard {
                id: 0,
                name: "Vineyard Owner".to_string(),
                bonus_vp: 1,
                starting_structures: vec![StructureType::Trellis],
                bonus_fields: 0,
                special_ability: None,
            },
            PapaCard {
                id: 1,
                name: "Infrastructure Developer".to_string(),
                bonus_vp: 0,
                starting_structures: vec![StructureType::Irrigation, StructureType::Yoke],
                bonus_fields: 0,
                special_ability: None,
            },
            PapaCard {
                id: 2,
                name: "Land Baron".to_string(),
                bonus_vp: 2,
                starting_structures: vec![],
                bonus_fields: 1,
                special_ability: Some(PapaAbility::ExtraVineyardField),
            },
            PapaCard {
                id: 3,
                name: "Cellar Master".to_string(),
                bonus_vp: 0,
                starting_structures: vec![StructureType::Windmill],
                bonus_fields: 0,
                special_ability: Some(PapaAbility::AdvancedCellar),
            },
            PapaCard {
                id: 4,
                name: "Wine Merchant".to_string(),
                bonus_vp: 1,
                starting_structures: vec![StructureType::TastingRoom],
                bonus_fields: 0,
                special_ability: Some(PapaAbility::TradingConnections),
            },
        ]
    }
}

#[derive(Component)]
pub struct Hand {
    pub owner: PlayerId,
    pub vine_cards: Vec<VineCard>,
    pub wine_order_cards: Vec<WineOrderCard>,
}

impl Hand {
    pub fn new(owner: PlayerId) -> Self {
        Self {
            owner,
            vine_cards: Vec::new(),
            wine_order_cards: Vec::new(),
        }
    }

    pub fn add_visitor_card(&mut self, visitor: VisitorCard) {
        // Store visitors as vine cards temporarily (simple solution)
        // In a full implementation, add visitor_cards: Vec<VisitorCard> to Hand
        info!("Player {:?} received visitor card: {}", self.owner, visitor.name);
    }
    
    pub fn total_cards(&self) -> usize {
        self.vine_cards.len() + self.wine_order_cards.len()
    }

}

#[derive(Component)]
pub struct UIPanel;

#[derive(Component)]
pub struct PlayerDashboard {
    pub player_id: PlayerId,
}

#[derive(Component)]
pub struct ActionButton {
    pub action: ActionSpace,
}

#[derive(Component)]
pub struct TurnIndicator;

#[derive(Component)]
pub struct WorkerSprite {
    pub player_id: PlayerId,
}

#[derive(Component)]
pub struct VineyardSprite {
    pub player_id: PlayerId,
    pub field_index: usize,
}

#[derive(Component)]
pub struct CardSprite {
    pub card_type: CardType,
}

#[derive(Clone, Copy)]
pub enum CardType {
    Vine,
    WineOrder,
}

#[derive(Component)]
pub struct PlayerCardsUI;

#[derive(Resource)]
pub struct GameAssets {
    pub worker_texture: Handle<Image>,
    pub vine_card_texture: Handle<Image>,
    pub wine_order_card_texture: Handle<Image>,
    pub field_texture: Handle<Image>,
}

#[derive(Resource)]
pub struct GameSettings {
    pub ai_enabled: bool,
    pub ai_difficulty: u8,
    pub audio_enabled: bool,
    pub sfx_volume: f32,
    pub music_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            ai_enabled: true,
            ai_difficulty: 1,
            audio_enabled: true,
            sfx_volume: 0.7,
            music_volume: 0.3,
        }
    }
}

#[derive(Component)]
pub struct AnimatedText {
    pub timer: Timer,
    pub start_pos: Vec2,
    pub end_pos: Vec2,
}

#[derive(Component, Clone)]
pub struct Structure {
    pub structure_type: StructureType,
    pub owner: PlayerId,
}

#[derive(Clone, Copy, Debug)]
pub enum StructureType {
    Trellis,      // $2 - Required for some vines
    Irrigation,   // $3 - Required for some vines  
    Yoke,         // $2 - Uproot vines or harvest in summer
    MediumCellar, // $4 - Store 4-6 value wines, make blush
    LargeCellar,  // $6 - Store 7-9 value wines, make sparkling  
    Windmill,     // $5 - +1 VP at end for every 7 lira
    Cottage,      // $4 - Draw extra visitor in fall
    TastingRoom,  // $6 - +1 VP when giving tours (if have wine)
}

#[derive(Component)]
pub struct MarkedForDespawn;

#[derive(Component)]
pub struct ResidualPaymentTracker {
    pub owner: PlayerId,
    pub level: u8, // 0-5, corresponds to lira earned each year
}

impl ResidualPaymentTracker {
    pub fn new(owner: PlayerId) -> Self {
        Self { owner, level: 0 }
    }
    
    pub fn advance(&mut self, steps: u8) {
        self.level = (self.level + steps).min(5);
    }
    
    pub fn annual_income(&self) -> u8 {
        self.level
    }
}

