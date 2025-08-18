# Viticulture Digital - Game Design Document

## 1. Project Overview

### 1.1 Game Concept
**Title:** Viticulture Digital  
**Genre:** Turn-based Strategy, Worker Placement  
**Platform:** PC (Windows, Linux, macOS)  
**Engine:** Bevy 2D (Rust)  
**Players:** 1-6 (AI and multiplayer support)  
**Development Timeline:** 12-18 months  

### 1.2 Core Vision
Create a faithful digital adaptation of Stonemaier Games' Viticulture that captures the tactile satisfaction of the board game while leveraging digital advantages like automated scoring, rule enforcement, and enhanced visual feedback.

### 1.3 Key Features
- Complete implementation of base Viticulture rules
- Beautiful 2D hand-drawn art style reminiscent of Tuscany
- Smooth animations and satisfying UI interactions
- AI opponents with multiple difficulty levels
- Online multiplayer with matchmaking
- Tutorial system for new players
- Achievement system and statistics tracking

## 2. Technical Architecture

### 2.1 Engine Choice: Bevy 2D
**Advantages:**
- Rust's memory safety and performance
- ECS architecture perfect for game state management
- Cross-platform compilation
- Growing ecosystem and active development
- Excellent for 2D board game adaptations

### 2.2 Core System Architecture

```rust
// Main game systems organization
mod systems {
    pub mod game_state;      // Game flow and turn management
    pub mod worker_placement; // Worker placement logic
    pub mod card_management;  // Vine and wine order cards
    pub mod scoring;         // Victory point calculation
    pub mod ai;              // AI opponent behavior
    pub mod networking;      // Multiplayer functionality
    pub mod ui;              // User interface systems
    pub mod animation;       // Visual feedback and transitions
}

mod components {
    pub mod player;          // Player data and resources
    pub mod board;           // Game board state
    pub mod cards;           // Card entities and data
    pub mod workers;         // Worker meeples
    pub mod vineyard;        // Individual vineyard boards
}

mod resources {
    pub mod game_config;     // Global game configuration
    pub mod turn_state;      // Current turn and phase
    pub mod ui_state;        // UI interaction state
}
```

### 2.3 ECS Component Structure

```rust
#[derive(Component)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub victory_points: u8,
    pub lira: u8,
    pub workers: Vec<WorkerId>,
    pub hand_size_limit: u8,
}

#[derive(Component)]
pub struct Vineyard {
    pub owner: PlayerId,
    pub fields: [Option<VineCard>; 9], // 3x3 grid
    pub cellars: CellarStorage,
    pub structures: Vec<Structure>,
}

#[derive(Component)]
pub struct Worker {
    pub id: WorkerId,
    pub owner: PlayerId,
    pub placed: bool,
    pub location: Option<ActionSpace>,
    pub is_grande_worker: bool,
}

#[derive(Component)]
pub struct ActionSpace {
    pub space_type: ActionType,
    pub season: Season,
    pub capacity: u8,
    pub bonus: Option<Bonus>,
    pub occupied_by: Vec<WorkerId>,
}
```

## 3. Game State Management

### 3.1 Game Flow States
```rust
#[derive(States, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    MainMenu,
    GameSetup,
    Spring,      // Wake-up order selection
    Summer,      // Summer action placement
    Fall,        // Harvest phase
    Winter,      // Winter action placement
    EndOfYear,   // Cleanup and scoring
    GameOver,    // Victory screen
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash)]
pub enum TurnPhase {
    SelectWakeUpTime,
    PlaceWorkers,
    ResolveActions,
    DrawCards,
    Cleanup,
}
```

### 3.2 Turn Management System
```rust
pub fn advance_turn_system(
    mut turn_state: ResMut<TurnState>,
    mut next_state: ResMut<NextState<GameState>>,
    players: Query<&Player>,
    workers: Query<&Worker>,
) {
    // Check if all players have completed their actions
    // Advance to next phase or next player
    // Handle end-of-year cleanup
    // Check victory conditions
}
```

## 4. Core Gameplay Systems

### 4.1 Worker Placement System
```rust
pub struct WorkerPlacementSystem;

impl WorkerPlacementSystem {
    pub fn can_place_worker(
        worker: &Worker,
        space: &ActionSpace,
        grande_worker_rules: bool,
    ) -> bool {
        // Check space capacity
        // Validate worker ownership and availability
        // Apply grande worker rules
    }
    
    pub fn place_worker(
        &mut self,
        worker_id: WorkerId,
        space_id: ActionSpaceId,
        world: &mut World,
    ) -> Result<(), PlacementError> {
        // Execute worker placement
        // Update game state
        // Trigger animations
    }
}
```

### 4.2 Card Management System
```rust
#[derive(Component)]
pub struct VineCard {
    pub id: CardId,
    pub name: String,
    pub cost: u8,
    pub vine_type: VineType, // Red, White
    pub harvest_value: u8,
    pub structure_bonus: Option<StructureType>,
}

#[derive(Component)]
pub struct WineOrderCard {
    pub id: CardId,
    pub name: String,
    pub red_wine_required: u8,
    pub white_wine_required: u8,
    pub blush_wine_required: u8,
    pub sparkling_wine_required: u8,
    pub victory_points: u8,
    pub residual_payment: u8,
}

pub fn draw_card_system(
    mut players: Query<&mut Player>,
    mut card_events: EventReader<DrawCardEvent>,
    card_decks: Res<CardDecks>,
) {
    // Handle card drawing logic
    // Manage deck shuffling
    // Update player hands
}
```

### 4.3 Vineyard Management
```rust
pub struct VineyardSystem;

impl VineyardSystem {
    pub fn plant_vine(
        vineyard: &mut Vineyard,
        field_position: (u8, u8),
        vine_card: VineCard,
    ) -> Result<(), PlantingError> {
        // Validate field availability
        // Check planting costs
        // Place vine in field
    }
    
    pub fn harvest_grapes(
        vineyard: &Vineyard,
        harvest_actions: &[HarvestAction],
    ) -> Vec<GrapeToken> {
        // Calculate grape production
        // Apply field bonuses
        // Generate grape tokens
    }
}
```

### 4.4 Wine Production System
```rust
#[derive(Component)]
pub struct CellarStorage {
    pub red_grapes: HashMap<u8, u8>,    // Value -> Quantity
    pub white_grapes: HashMap<u8, u8>,
    pub red_wine: HashMap<u8, u8>,
    pub white_wine: HashMap<u8, u8>,
    pub blush_wine: HashMap<u8, u8>,
    pub sparkling_wine: HashMap<u8, u8>,
}

pub fn make_wine_system(
    mut vineyards: Query<&mut Vineyard>,
    mut wine_events: EventReader<MakeWineEvent>,
) {
    // Convert grapes to wine
    // Handle aging bonuses
    // Update cellar storage
}
```

## 5. User Interface Design

### 5.1 UI Layout Architecture
```rust
pub struct UIState {
    pub current_screen: UIScreen,
    pub selected_worker: Option<WorkerId>,
    pub hovered_action_space: Option<ActionSpaceId>,
    pub card_hand_visible: bool,
    pub vineyard_zoom_level: f32,
}

#[derive(Component)]
pub struct UIPanel {
    pub panel_type: PanelType,
    pub visibility: bool,
    pub position: Vec2,
    pub size: Vec2,
}

pub enum PanelType {
    PlayerHand,
    ActionBoard,
    VineyardBoard,
    ScoreTrack,
    GameLog,
    SettingsMenu,
}
```

### 5.2 Visual Hierarchy
- **Primary Focus:** Current player's vineyard board
- **Secondary Focus:** Main action board
- **Tertiary Elements:** Other players' vineyards (miniaturized)
- **UI Overlays:** Hand cards, score track, game controls

### 5.3 Input Handling
```rust
pub fn handle_input_system(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut ui_state: ResMut<UIState>,
) {
    // Handle mouse clicks for worker placement
    // Process keyboard shortcuts
    // Manage drag-and-drop interactions
    // Update hover states
}
```

## 6. Visual Design

### 6.1 Art Style
- **Theme:** Hand-painted Tuscan countryside aesthetic
- **Color Palette:** Warm earth tones, vineyard greens, sunset oranges
- **Style:** Semi-realistic with stylized elements
- **Resolution:** Vector-based graphics for scalability

### 6.2 Animation System
```rust
#[derive(Component)]
pub struct AnimationComponent {
    pub animation_type: AnimationType,
    pub duration: f32,
    pub elapsed: f32,
    pub easing: EasingFunction,
}

pub enum AnimationType {
    WorkerMovement { from: Vec2, to: Vec2 },
    CardDraw { target_position: Vec2 },
    ScoreChange { old_score: u8, new_score: u8 },
    ResourceGain { resource_type: ResourceType, amount: u8 },
}

pub fn animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animated_entities: Query<(Entity, &mut Transform, &mut AnimationComponent)>,
) {
    // Update animation progress
    // Apply easing functions
    // Remove completed animations
}
```

### 6.3 Visual Feedback Systems
- **Hover Effects:** Subtle highlighting of interactable elements
- **Valid Action Indicators:** Green outlines for legal moves
- **Invalid Action Feedback:** Red flashing for illegal attempts
- **Progress Animations:** Smooth transitions for score changes
- **Particle Effects:** Grape harvesting, wine pouring animations

## 7. AI Implementation

### 7.1 AI Architecture
```rust
pub struct AIPlayer {
    pub difficulty: AIDifficulty,
    pub strategy_weights: StrategyWeights,
    pub planning_depth: u8,
}

pub enum AIDifficulty {
    Beginner,    // Focus on valid moves, basic strategy
    Intermediate, // Medium-term planning, some optimization
    Advanced,    // Long-term strategy, competitive play
    Expert,      // Min-max with pruning, optimal play
}

#[derive(Component)]
pub struct StrategyWeights {
    pub vine_planting_priority: f32,
    pub structure_building_priority: f32,
    pub wine_order_fulfillment: f32,
    pub worker_efficiency: f32,
    pub end_game_rush: f32,
}
```

### 7.2 Decision Making System
```rust
pub fn ai_decision_system(
    ai_players: Query<&AIPlayer>,
    game_state: Res<TurnState>,
    board_state: Res<BoardState>,
) {
    // Evaluate current game state
    // Generate possible moves
    // Score moves based on strategy
    // Select and execute best move
}

impl AIPlayer {
    pub fn evaluate_move(&self, game_state: &GameState, action: &Action) -> f32 {
        // Heuristic evaluation of action value
        // Consider immediate benefits
        // Factor in long-term strategy
        // Weight against difficulty level
    }
}
```

## 8. Networking Architecture

### 8.1 Multiplayer Design
```rust
pub struct NetworkPlayer {
    pub connection_id: u64,
    pub player_data: Player,
    pub connection_state: ConnectionState,
    pub last_heartbeat: Instant,
}

pub enum NetworkMessage {
    PlayerJoin(String),         // Player name
    PlayerLeave(u64),          // Connection ID
    ActionSubmission(Action),   // Game action
    GameStateSync(GameState),   // Full state sync
    Heartbeat,                 // Keep-alive
}

pub fn network_message_system(
    mut network_events: EventReader<NetworkMessage>,
    mut game_state: ResMut<GameState>,
    mut players: Query<&mut Player>,
) {
    // Process incoming network messages
    // Validate actions against game rules
    // Broadcast state changes
    // Handle disconnections gracefully
}
```

### 8.2 Synchronization Strategy
- **Authoritative Server:** Server validates all actions
- **Client Prediction:** Immediate local feedback
- **State Synchronization:** Regular game state broadcasts
- **Rollback System:** Handle conflicts and corrections

## 9. Audio Design

### 9.1 Audio Architecture
```rust
#[derive(Resource)]
pub struct AudioManager {
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub current_music_track: Option<Handle<AudioSource>>,
    pub audio_enabled: bool,
}

pub enum SoundEffect {
    WorkerPlacement,
    CardDraw,
    VinePlanting,
    WinePouring,
    ScoreIncrease,
    SeasonTransition,
    Victory,
}

pub fn audio_system(
    mut commands: Commands,
    audio: Res<Audio>,
    audio_manager: Res<AudioManager>,
    mut audio_events: EventReader<AudioEvent>,
    asset_server: Res<AssetServer>,
) {
    // Play sound effects based on game events
    // Manage background music transitions
    // Handle volume controls
}
```

### 9.2 Sound Design
- **Ambient Music:** Peaceful Italian countryside themes
- **Sound Effects:** Wooden board game sounds (worker placement, card shuffling)
- **Feedback Sounds:** Success/failure audio cues
- **Seasonal Audio:** Different ambient tracks for each season

## 10. Performance Optimization

### 10.1 Rendering Optimization
```rust
pub fn sprite_batching_system(
    mut sprite_batch: ResMut<SpriteBatch>,
    sprites: Query<(&Transform, &Sprite, &Visibility)>,
) {
    // Batch similar sprites for efficient rendering
    // Cull off-screen elements
    // Optimize texture atlas usage
}

pub fn level_of_detail_system(
    camera: Query<&Transform, With<Camera2d>>,
    mut sprites: Query<(&mut Visibility, &Transform), (With<Sprite>, Without<Camera2d>)>,
) {
    // Hide detailed elements when zoomed out
    // Show simplified versions of distant objects
    // Optimize based on viewport
}
```

### 10.2 Memory Management
- **Asset Streaming:** Load/unload assets based on game state
- **Entity Pooling:** Reuse entities for cards, tokens, effects
- **Texture Atlasing:** Combine small textures for efficiency
- **Garbage Collection:** Minimize allocations in hot paths

## 11. Development Phases

### Phase 1: Core Implementation (Months 1-4)
- Basic game loop and state management
- Core worker placement mechanics
- Simple vineyard management
- Local single-player with basic AI

### Phase 2: Visual Polish (Months 5-8)
- Complete art asset integration
- Animation system implementation
- UI/UX refinement
- Audio integration

### Phase 3: Advanced Features (Months 9-12)
- Multiplayer networking
- Advanced AI opponents
- Achievement system
- Tutorial implementation

### Phase 4: Polish & Release (Months 13-18)
- Performance optimization
- Platform-specific builds
- Beta testing and bug fixes
- Marketing and distribution

## 12. Technical Considerations

### 12.1 Cross-Platform Compatibility
```toml
# Cargo.toml
[dependencies]
bevy = { version = "0.12", features = ["dynamic_linking"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
renet = "0.0.13"  # For networking

[target.'cfg(target_os = "windows")'.dependencies]
winapi = "0.3"

[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.24"

[target.'cfg(target_os = "linux")'.dependencies]
x11 = "2.19"
```

### 12.2 Build Configuration
```rust
// Different configurations for development/release
#[cfg(debug_assertions)]
const ENABLE_DEBUG_UI: bool = true;

#[cfg(not(debug_assertions))]
const ENABLE_DEBUG_UI: bool = false;

// Platform-specific optimizations
#[cfg(target_arch = "wasm32")]
const MAX_TEXTURE_SIZE: u32 = 2048;

#[cfg(not(target_arch = "wasm32"))]
const MAX_TEXTURE_SIZE: u32 = 4096;
```

## 13. Testing Strategy

### 13.1 Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_placement_validation() {
        // Test valid worker placements
        // Test invalid placements (occupied spaces, wrong season)
        // Test grande worker special rules
    }

    #[test]
    fn test_vineyard_planting() {
        // Test valid vine planting
        // Test field capacity limits
        // Test cost validation
    }

    #[test]
    fn test_victory_conditions() {
        // Test 20-point victory trigger
        // Test tie-breaking rules
        // Test edge cases
    }
}
```

### 13.2 Integration Testing
- **Game Flow Testing:** Complete game simulations
- **Network Testing:** Multiplayer scenarios with various connection states
- **Performance Testing:** Frame rate consistency under load
- **Platform Testing:** Validation across Windows, Linux, macOS

## 14. Future Expansion Plans

### 14.1 DLC Content
- **Tuscany Expansion:** Essential edition mechanics
- **Visitors from the Rhine Valley:** Additional visitor cards
- **Moor Visitors:** Extended visitor mechanics

### 14.2 Additional Features
- **Tournament Mode:** Bracket-style competitions
- **Replay System:** Watch and analyze completed games
- **Spectator Mode:** Observe ongoing multiplayer games
- **Custom Rules:** House rules and variants

## 15. Risk Assessment

### 15.1 Technical Risks
- **Bevy API Stability:** Engine still in active development
- **Networking Complexity:** Real-time synchronization challenges
- **Performance Scaling:** Handling 6-player games smoothly

### 15.2 Mitigation Strategies
- **Engine Version Locking:** Pin to stable Bevy releases
- **Networking Abstraction:** Isolate network code for easier updates
- **Performance Budgets:** Set and monitor performance targets
- **Automated Testing:** Comprehensive test suite for regression detection

This game design document provides a comprehensive foundation for developing a digital version of Viticulture in Rust using Bevy 2D, covering all major systems, technical considerations, and development planning.