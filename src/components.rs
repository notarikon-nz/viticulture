use bevy::prelude::*;

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
    pub wake_up_order: Vec<(PlayerId, u8)>, // (player_id, wake_up_time)
}

#[derive(Resource)]
pub struct GameConfig {
    pub player_count: u8,
    pub target_victory_points: u8,
    pub current_year: u8,
    pub max_years: u8,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            player_count: 2,
            target_victory_points: 20,
            current_year: 1,
            max_years: 7, // Game ends after 7 years if no one reaches 20 VP
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
}

impl Player {
    pub fn new(id: u8, name: String) -> Self {
        Self {
            id: PlayerId(id),
            name,
            victory_points: 0,
            lira: 3, // Starting lira
            workers: 2,
            grande_worker_available: true,
        }
    }
    
    pub fn gain_victory_points(&mut self, points: u8) {
        self.victory_points = self.victory_points.saturating_add(points);
    }
    
    pub fn gain_lira(&mut self, amount: u8) {
        self.lira = self.lira.saturating_add(amount);
    }
}

#[derive(Component)]
pub struct Vineyard {
    pub owner: PlayerId,
    pub fields: [Option<VineType>; 9], // 3x3 grid, simplified for now
    pub red_grapes: u8,
    pub white_grapes: u8,
    pub red_wine: u8,
    pub white_wine: u8,
    pub lira: u8,
}

impl Vineyard {
    pub fn new(owner: PlayerId) -> Self {
        Self {
            owner,
            fields: [None; 9],
            red_grapes: 0,
            white_grapes: 0,
            red_wine: 0,
            white_wine: 0,
            lira: 3, // Starting lira
        }
    }
    
    pub fn can_plant_vine(&self, field_index: usize, vine_card: &VineCard) -> bool {
        field_index < 9 && self.fields[field_index].is_none() && self.lira >= vine_card.cost
    }
    
    pub fn plant_vine(&mut self, field_index: usize, vine_card: VineCard) -> bool {
        if self.can_plant_vine(field_index, &vine_card) {
            self.fields[field_index] = Some(vine_card.vine_type);
            self.lira = self.lira.saturating_sub(vine_card.cost);
            true
        } else {
            false
        }
    }
    
    pub fn harvest_grapes(&mut self) {
        for field in &self.fields {
            if let Some(vine_type) = field {
                match vine_type {
                    VineType::Red(value) => self.red_grapes += value,
                    VineType::White(value) => self.white_grapes += value,
                }
            }
        }
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
}

#[derive(Clone, Copy, Debug)]
pub enum VineType {
    Red(u8),   // harvest value
    White(u8), // harvest value
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
}

#[derive(Component)]
pub struct Clickable {
    pub size: Vec2,
}

impl ActionSpaceSlot {
    pub fn new(action: ActionSpace, position: Vec2, is_summer: bool) -> Self {
        Self {
            action,
            occupied_by: None,
            bonus_worker_slot: None,
            position,
            is_summer,
        }
    }
    
    pub fn can_place_worker(&self, player_id: PlayerId, current_state: &GameState) -> bool {
        // Check if space is occupied
        if self.occupied_by.is_some() {
            return false;
        }
        
        // Check if it's the right season
        match current_state {
            GameState::Summer => self.is_summer,
            GameState::Winter => !self.is_summer,
            _ => false,
        }
    }
}

impl ActionBoard {
    pub fn new() -> Self {
        let mut spaces = Vec::new();
        
        // Summer actions (left side)
        spaces.push(ActionSpaceSlot::new(ActionSpace::DrawVine, Vec2::new(-300.0, 100.0), true));
        spaces.push(ActionSpaceSlot::new(ActionSpace::PlantVine, Vec2::new(-300.0, 50.0), true));
        spaces.push(ActionSpaceSlot::new(ActionSpace::BuildStructure, Vec2::new(-300.0, 0.0), true));
        spaces.push(ActionSpaceSlot::new(ActionSpace::GiveTour, Vec2::new(-300.0, -50.0), true));
        spaces.push(ActionSpaceSlot::new(ActionSpace::SellGrapes, Vec2::new(-300.0, -100.0), true));
        
        // Winter actions (right side)
        spaces.push(ActionSpaceSlot::new(ActionSpace::DrawWineOrder, Vec2::new(300.0, 100.0), false));
        spaces.push(ActionSpaceSlot::new(ActionSpace::Harvest, Vec2::new(300.0, 50.0), false));
        spaces.push(ActionSpaceSlot::new(ActionSpace::MakeWine, Vec2::new(300.0, 0.0), false));
        spaces.push(ActionSpaceSlot::new(ActionSpace::FillOrder, Vec2::new(300.0, -50.0), false));
        spaces.push(ActionSpaceSlot::new(ActionSpace::TrainWorker, Vec2::new(300.0, -100.0), false));
        
        Self { spaces }
    }
}

// Simple card representation for now
#[derive(Component, Clone)]
pub struct VineCard {
    pub id: u32,
    pub vine_type: VineType,
    pub cost: u8,
}

#[derive(Component, Clone)]
pub struct WineOrderCard {
    pub id: u32,
    pub red_wine_needed: u8,
    pub white_wine_needed: u8,
    pub victory_points: u8,
    pub payout: u8,
}

impl WineOrderCard {
    pub fn new(id: u32, red: u8, white: u8, vp: u8, payout: u8) -> Self {
        Self {
            id,
            red_wine_needed: red,
            white_wine_needed: white,
            victory_points: vp,
            payout,
        }
    }
}

#[derive(Resource)]
pub struct CardDecks {
    pub vine_deck: Vec<VineCard>,
    pub wine_order_deck: Vec<WineOrderCard>,
    pub vine_discard: Vec<VineCard>,
    pub wine_order_discard: Vec<WineOrderCard>,
}

impl CardDecks {
    pub fn new() -> Self {
        let mut vine_deck = Vec::new();
        let mut wine_order_deck = Vec::new();
        
        // Create basic vine cards
        for i in 0..20 {
            vine_deck.push(VineCard {
                id: i,
                vine_type: if i % 2 == 0 { VineType::Red(2) } else { VineType::White(2) },
                cost: 1,
            });
        }
        
        // Create varied wine order cards with different VP values
        wine_order_deck.push(WineOrderCard::new(100, 1, 0, 1, 1)); // Easy red order
        wine_order_deck.push(WineOrderCard::new(101, 0, 1, 1, 1)); // Easy white order
        wine_order_deck.push(WineOrderCard::new(102, 2, 0, 2, 2)); // Medium red order
        wine_order_deck.push(WineOrderCard::new(103, 0, 2, 2, 2)); // Medium white order
        wine_order_deck.push(WineOrderCard::new(104, 1, 1, 2, 2)); // Mixed order
        wine_order_deck.push(WineOrderCard::new(105, 3, 0, 4, 3)); // Hard red order
        wine_order_deck.push(WineOrderCard::new(106, 0, 3, 4, 3)); // Hard white order
        wine_order_deck.push(WineOrderCard::new(107, 2, 2, 5, 4)); // Hard mixed order
        wine_order_deck.push(WineOrderCard::new(108, 4, 0, 6, 5)); // Very hard red
        wine_order_deck.push(WineOrderCard::new(109, 0, 4, 6, 5)); // Very hard white
        wine_order_deck.push(WineOrderCard::new(110, 3, 2, 7, 6)); // Epic order
        wine_order_deck.push(WineOrderCard::new(111, 2, 3, 7, 6)); // Epic order 2
        
        Self {
            vine_deck,
            wine_order_deck,
            vine_discard: Vec::new(),
            wine_order_discard: Vec::new(),
        }
    }
    
    pub fn draw_vine_card(&mut self) -> Option<VineCard> {
        self.vine_deck.pop()
    }
    
    pub fn draw_wine_order_card(&mut self) -> Option<WineOrderCard> {
        self.wine_order_deck.pop()
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
pub struct GameStatusText;