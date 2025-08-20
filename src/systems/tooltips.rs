use bevy::prelude::*;
use crate::components::*;

#[derive(Component)]
pub struct Tooltip {
    pub text: String,
    pub offset: Vec2,
}

#[derive(Component)]
pub struct TooltipTarget {
    pub tooltip_text: String,
    pub bounds: Rect,
}

#[derive(Component)]
pub struct TooltipUI;

#[derive(Resource)]
pub struct TooltipState {
    pub current_tooltip: Option<String>,
    pub hover_timer: Timer,
    pub mouse_position: Vec2,
}

impl Default for TooltipState {
    fn default() -> Self {
        Self {
            current_tooltip: None,
            hover_timer: Timer::from_seconds(0.5, TimerMode::Once), // Show after 0.5s hover
            mouse_position: Vec2::ZERO,
        }
    }
}

pub fn setup_tooltips_system(mut commands: Commands) {
    commands.insert_resource(TooltipState::default());
    
    // Add tooltips to action buttons
    setup_action_tooltips(&mut commands);
}

fn setup_action_tooltips(commands: &mut Commands) {
    // These would be positioned based on actual UI layout
    let action_tooltips = vec![
        (ActionSpace::DrawVine, "Draw a vine card from the deck. Vine cards are needed to plant vines in your vineyard fields."),
        (ActionSpace::PlantVine, "Plant a vine card from your hand into an empty field. Costs lira based on vine type."),
        (ActionSpace::BuildStructure, "Build a structure that provides ongoing benefits. Structures cost lira but give permanent advantages."),
        (ActionSpace::GiveTour, "Gain 2 lira by giving tours to visitors. Tasting Room structure increases this bonus."),
        (ActionSpace::SellGrapes, "Sell all your grapes for 1 lira each. Useful when you need quick money."),
        (ActionSpace::TrainWorker, "Pay 4 lira to gain an additional worker for future turns."),
        (ActionSpace::DrawWineOrder, "Draw a wine order card. These show what wines customers want and reward VP."),
        (ActionSpace::Harvest, "Collect grapes from your planted vines. Each vine produces grapes equal to its value."),
        (ActionSpace::MakeWine, "Convert grapes into wine. Can make regular wine, blush (red+white), or sparkling wine."),
        (ActionSpace::FillOrder, "Fulfill a wine order card for victory points and lira rewards."),
    ];
    
    for (action, tooltip_text) in action_tooltips {
        // This would be connected to actual UI elements in a real implementation
        info!("Tooltip for {:?}: {}", action, tooltip_text);
    }
}

pub fn tooltip_hover_system(
    mut tooltip_state: ResMut<TooltipState>,
    time: Res<Time>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    tooltip_targets: Query<&TooltipTarget>,
    settings: Res<crate::systems::settings::UserSettings>,
) {
    if !settings.show_tooltips {
        tooltip_state.current_tooltip = None;
        return;
    }
    
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    
    if let Some(cursor_pos) = window.cursor_position() {
        let world_pos = camera.viewport_to_world_2d(camera_transform, cursor_pos).unwrap_or(Vec2::ZERO);
        tooltip_state.mouse_position = world_pos;
        
        // Check if mouse is over any tooltip target
        let mut hovering_tooltip = None;
        for target in tooltip_targets.iter() {
            if target.bounds.contains(world_pos) {
                hovering_tooltip = Some(target.tooltip_text.clone());
                break;
            }
        }
        
        match hovering_tooltip {
            Some(new_tooltip) => {
                if tooltip_state.current_tooltip.as_ref() != Some(&new_tooltip) {
                    // Started hovering new element
                    tooltip_state.current_tooltip = Some(new_tooltip);
                    tooltip_state.hover_timer.reset();
                }
                tooltip_state.hover_timer.tick(time.delta());
            }
            None => {
                // Not hovering anything
                tooltip_state.current_tooltip = None;
                tooltip_state.hover_timer.reset();
            }
        }
    }
}

pub fn tooltip_display_system(
    mut commands: Commands,
    tooltip_state: Res<TooltipState>,
    existing_tooltips: Query<Entity, With<TooltipUI>>,
    settings: Res<crate::systems::settings::UserSettings>,
) {
    if !settings.show_tooltips {
        // Clear any existing tooltips
        for entity in existing_tooltips.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }
    
    // Clear existing tooltips
    for entity in existing_tooltips.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // Show tooltip if timer finished and we have tooltip text
    if let Some(ref tooltip_text) = tooltip_state.current_tooltip {
        if tooltip_state.hover_timer.finished() {
            spawn_tooltip(&mut commands, tooltip_text, tooltip_state.mouse_position);
        }
    }
}

fn spawn_tooltip(commands: &mut Commands, text: &str, mouse_pos: Vec2) {
    // Calculate tooltip position (offset from mouse)
    let tooltip_offset = Vec2::new(10.0, 20.0);
    let tooltip_pos = mouse_pos + tooltip_offset;
    
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(tooltip_pos.x),
                top: Val::Px(tooltip_pos.y),
                padding: UiRect::all(Val::Px(8.0)),
                max_width: Val::Px(300.0),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.0, 0.0, 0.0, 0.9)).into(),
            z_index: ZIndex::Global(1000),
            ..default()
        },
        TooltipUI,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            text,
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

#[derive(Component)]
pub struct ContextualHelp;

// Quick reference overlay
pub fn quick_reference_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    existing_reference: Query<Entity, With<QuickReference>>,
) {
    if keyboard.just_pressed(KeyCode::F2) {
        if existing_reference.is_empty() {
            show_quick_reference(&mut commands);
        } else {
            for entity in existing_reference.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn show_quick_reference(commands: &mut Commands) {
    let reference_text = 
        "QUICK REFERENCE (F2 to close)\n\n\
         🎮 CONTROLS:\n\
         F1 - Contextual Help\n\
         F2 - Quick Reference\n\
         F5 - Save Game\n\
         F9 - Load Game\n\
         TAB - Statistics\n\
         ESC - Settings\n\
         Ctrl+Z - Undo\n\
         ENTER - End Turn\n\n\
         🏆 VICTORY:\n\
         • First to 20 VP wins\n\
         • Or highest VP after 7 years\n\
         • Fulfill wine orders for VP\n\n\
         ⚙️ STRUCTURES:\n\
         • Trellis: +1 vine value\n\
         • Irrigation: -1 vine cost\n\
         • Yoke: +1 lira when harvesting\n\
         • Windmill: +1 VP per 7 lira\n\
         • Cottage: +1 worker\n\
         • Tasting Room: +1 tour lira\n\n\
         🍷 WINE TYPES:\n\
         • Regular: 1 grape = 1 wine\n\
         • Blush: 1 red + 1 white = 1 wine\n\
         • Sparkling: 1 red + 1 white = 2 wine\n\n\
         🌅 WAKE-UP BONUSES:\n\
         1st: Draw vine card\n\
         2nd: +1 lira\n\
         3rd: No bonus\n\
         4th: +1 lira\n\
         5th: Draw wine order\n\
         6th+: +1 victory point\n\n\
         🎯 EXPANSIONS (if enabled):\n\
         F6 - Toggle Tuscany\n\
         F7 - Toggle Visitor Cards\n\
         F8 - Toggle Advanced Boards\n\
         V - Draw/Play Visitor Card";
    
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                left: Val::Px(50.0),
                width: Val::Px(400.0),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.1, 0.1, 0.1, 0.95)).into(),
            z_index: ZIndex::Global(600),
            ..default()
        },
        QuickReference,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            reference_text,
            TextStyle {
                font_size: 12.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

#[derive(Component)]
pub struct QuickReference;

// Enhanced action button tooltips
pub fn setup_action_button_tooltips(
    mut commands: Commands,
    action_buttons: Query<Entity, With<crate::components::ActionButton>>,
) {
    for entity in action_buttons.iter() {
        // This would add tooltip targets to existing action buttons
        // In a real implementation, this would be integrated with the UI system
        commands.entity(entity).insert(TooltipTarget {
            tooltip_text: "Click to perform this action".to_string(),
            bounds: Rect::from_center_size(Vec2::ZERO, Vec2::new(100.0, 40.0)),
        });
    }
}

// Card explanation tooltips
pub fn card_tooltip_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    hands: Query<&Hand>,
    turn_order: Res<TurnOrder>,
    existing_card_info: Query<Entity, With<CardInfoPanel>>,
) {
    // Show detailed card info with right-click or I key
    if mouse_input.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::KeyI) {
        if existing_card_info.is_empty() {
            if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
                if let Some(hand) = hands.iter().find(|h| h.owner == *current_player_id) {
                    show_card_info_panel(&mut commands, hand);
                }
            }
        } else {
            for entity in existing_card_info.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn show_card_info_panel(commands: &mut Commands, hand: &Hand) {
    let mut info_text = "CARDS IN HAND (Right-click to close)\n\n".to_string();
    
    info_text.push_str("🍇 VINE CARDS:\n");
    for (i, vine_card) in hand.vine_cards.iter().enumerate() {
        let vine_type = match vine_card.vine_type {
            crate::components::VineType::Red(value) => format!("Red ({})", value),
            crate::components::VineType::White(value) => format!("White ({})", value),
        };
        info_text.push_str(&format!("  {}. {} - Cost: {}\n", i + 1, vine_type, vine_card.cost));
    }
    
    info_text.push_str("\n🍷 WINE ORDERS:\n");
    for (i, order) in hand.wine_order_cards.iter().enumerate() {
        info_text.push_str(&format!(
            "  {}. Need: {}R {}W → {} VP, {} lira\n",
            i + 1,
            order.red_wine_needed,
            order.white_wine_needed,
            order.victory_points,
            order.payout
        ));
    }
    
    if hand.vine_cards.is_empty() && hand.wine_order_cards.is_empty() {
        info_text.push_str("No cards in hand. Use Draw actions to get cards!");
    }
    
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(200.0),
                left: Val::Px(400.0),
                width: Val::Px(350.0),
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.2, 0.1, 0.0, 0.95)).into(),
            z_index: ZIndex::Global(400),
            ..default()
        },
        CardInfoPanel,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            info_text,
            TextStyle {
                font_size: 13.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

#[derive(Component)]
pub struct CardInfoPanel;

// Advanced tooltip system for specific game elements
pub fn setup_game_element_tooltips(
    mut commands: Commands,
    workers: Query<Entity, With<Worker>>,
    vineyards: Query<Entity, With<Vineyard>>,
) {
    // Add tooltips to workers
    for entity in workers.iter() {
        commands.entity(entity).insert(TooltipTarget {
            tooltip_text: "Worker: Place on action spaces to perform actions. Grande workers can share occupied spaces.".to_string(),
            bounds: Rect::from_center_size(Vec2::ZERO, Vec2::new(20.0, 20.0)),
        });
    }
    
    // Add tooltips to vineyard fields
    for entity in vineyards.iter() {
        commands.entity(entity).insert(TooltipTarget {
            tooltip_text: "Vineyard: Plant vines here to produce grapes. Each field can hold one vine.".to_string(),
            bounds: Rect::from_center_size(Vec2::ZERO, Vec2::new(35.0, 35.0)),
        });
    }
}

// Status indicator tooltips
pub fn setup_status_tooltips(commands: &mut Commands) {
    let status_explanations = vec![
        ("Victory Points", "Primary win condition. Reach 20 VP or have the most after 7 years."),
        ("Lira", "Game currency. Used to plant vines, build structures, and train workers."),
        ("Grapes", "Harvested from planted vines. Convert to wine using Make Wine action."),
        ("Wine", "Made from grapes. Used to fulfill wine orders for victory points."),
        ("Workers", "Action tokens. Place on spaces to perform actions each turn."),
        ("Year", "Game timer. Game ends after 7 years if no one reaches 20 VP."),
    ];
    
    for (status, explanation) in status_explanations {
        info!("Status tooltip for {}: {}", status, explanation);
        // In a real implementation, these would be attached to UI elements
    }
}

// Rule explanations for game concepts
pub fn get_rule_explanation(concept: &str) -> String {
    match concept {
        "worker_placement" => {
            "Worker Placement: Place your workers on action spaces to perform actions. \
             Each space can only hold one regular worker, but Grande workers can share spaces. \
             Spaces with bonus slots can hold an extra worker.".to_string()
        }
        "victory_points" => {
            "Victory Points: The main way to win the game. Earned primarily by fulfilling wine orders. \
             First player to reach 20 VP (or highest after 7 years) wins. \
             End-game bonuses can provide additional VP.".to_string()
        }
        "seasons" => {
            "Seasons: The game alternates between Summer and Winter each turn. \
             Summer actions focus on income and preparation (plant vines, build structures). \
             Winter actions focus on production and scoring (harvest, make wine, fill orders).".to_string()
        }
        "wake_up_order" => {
            "Wake-up Order: Each spring, choose when to wake up (1-7). \
             Earlier wake-up gives better bonuses but you act later in turn order. \
             1st: Draw vine card, 2nd: +1 lira, 3rd: No bonus, 4th: +1 lira, \
             5th: Draw wine order, 6th+: +1 victory point.".to_string()
        }
        "wine_making" => {
            "Wine Making: Convert grapes to wine using the Make Wine action. \
             Basic: 1 grape = 1 wine of same color. \
             Blush: 1 red + 1 white grape = 1 wine (stored as white). \
             Sparkling: 1 red + 1 white = 2 wine value (stored as red).".to_string()
        }
        "structures" => {
            "Structures: Permanent buildings that provide ongoing benefits. \
             Trellis (+1 vine value), Irrigation (-1 vine cost), Yoke (+1 lira when harvesting), \
             Windmill (+1 VP per 7 lira at game end), Cottage (+1 worker), \
             Tasting Room (+1 lira when giving tours).".to_string()
        }
        "grande_worker" => {
            "Grande Worker: Your special large worker. Can be placed on any action space \
             even if occupied by another worker. Each player has exactly one grande worker. \
             Use strategically when regular spaces are blocked.".to_string()
        }
        "vine_cards" => {
            "Vine Cards: Represent different grape varieties. Each has a harvest value (1-4) \
             and planting cost (usually 1-2 lira). Red and white vines produce different \
             colored grapes. Higher value vines cost more but produce more grapes.".to_string()
        }
        "wine_orders" => {
            "Wine Orders: Customer requests for specific wines. Show required red/white wine, \
             victory point reward, and lira payout. Focus on orders you can fulfill \
             with your current wine production. Higher VP orders require more wine.".to_string()
        }
        "residual_payments" => {
            "Residual Payments: Some cards provide ongoing income each year. \
             Indicated by a coin symbol. These provide steady lira income \
             throughout the game, making them valuable long-term investments.".to_string()
        }
        _ => format!("No explanation available for: {}", concept)
    }
}

// Contextual help system
pub fn contextual_help_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    existing_help: Query<Entity, With<ContextualHelp>>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        if existing_help.is_empty() {
            show_contextual_help(&mut commands, current_state.get());
        } else {
            // Hide help
            for entity in existing_help.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn show_contextual_help(commands: &mut Commands, game_state: &GameState) {
    let help_text = match game_state {
        GameState::MainMenu => {
            "VITICULTURE - Main Menu\n\n\
             • Press SPACE to start a new game\n\
             • ESC - Settings menu\n\
             • TAB - View statistics\n\
             • F1 - Toggle this help"
        }
        GameState::Spring => {
            "SPRING PHASE - Wake-up Order\n\n\
             Choose when to wake up (1-7):\n\
             • Earlier = better bonuses, later turn order\n\
             • Later = worse bonuses, earlier turn order\n\
             • Press SPACE to auto-assign and continue\n\
             • F1 - Toggle help"
        }
        GameState::Summer => {
            "SUMMER PHASE - Income & Preparation\n\n\
             Summer Actions:\n\
             • Draw Vine - Get vine cards to plant\n\
             • Plant Vine - Place vines in vineyard fields\n\
             • Build Structure - Gain permanent benefits\n\
             • Give Tour - Earn 2 lira\n\
             • Sell Grapes - Convert grapes to lira\n\
             • Train Worker - Gain extra worker\n\n\
             • Click action spaces or use buttons\n\
             • ENTER - End turn\n\
             • F1 - Toggle help"
        }
        GameState::Winter => {
            "WINTER PHASE - Production & Scoring\n\n\
             Winter Actions:\n\
             • Draw Wine Order - Get customer orders\n\
             • Harvest - Collect grapes from vines\n\
             • Make Wine - Convert grapes to wine\n\
             • Fill Order - Complete orders for VP\n\n\
             • Focus on fulfilling wine orders for VP\n\
             • ENTER - End turn\n\
             • F1 - Toggle help"
        }
        GameState::Fall => {
            "FALL PHASE - Automatic Harvest\n\n\
             • All planted vines automatically produce grapes\n\
             • No player actions required\n\
             • Press SPACE to continue to Winter\n\
             • F1 - Toggle help"
        }
        GameState::GameOver => {
            "GAME OVER\n\n\
             • View final scores and winner\n\
             • Press SPACE to start new game\n\
             • TAB - View updated statistics\n\
             • F1 - Toggle help"
        }
        _ => {
            "Game Setup\n\n\
             • Preparing new game...\n\
             • F1 - Toggle help"
        }
    };
    
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                right: Val::Px(20.0),
                width: Val::Px(350.0),
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.0, 0.2, 0.4, 0.95)).into(),
            z_index: ZIndex::Global(500),
            ..default()
        },
        ContextualHelp,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            help_text,
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}