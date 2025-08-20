use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::*;

#[derive(Resource, Default)]
pub struct TutorialState {
    pub active: bool,
    pub current_step: usize,
    pub completed_steps: Vec<usize>,
    pub skip_tutorial: bool,
}

#[derive(Serialize, Deserialize, Resource, Default)]
pub struct TutorialProgress {
    pub tutorial_completed: bool,
    pub steps_completed: Vec<usize>,
    pub times_played: u32,
}

impl TutorialProgress {
    pub fn load_or_default() -> Self {
        match std::fs::read_to_string("viticulture_tutorial.json") {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
    
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("viticulture_tutorial.json", json);
        }
    }
}

#[derive(Clone)]
pub struct TutorialStep {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub action_required: TutorialAction,
    pub highlight_element: Option<String>,
    pub completion_message: String,
}

#[derive(Clone, PartialEq)]
pub enum TutorialAction {
    ClickUI(String),           // Click specific UI element
    PressKey(KeyCode),         // Press specific key
    PlaceWorker(ActionSpace),  // Place worker on action space
    ViewHelp,                  // Open help system
    CompletePhase,             // Finish current phase
    DrawCard,                  // Draw a card
    PlantVine,                 // Plant a vine
    MakeWine,                  // Make wine
    FillOrder,                 // Complete wine order
    Automatic,                 // Auto-advance step
}

#[derive(Component)]
pub struct TutorialUI;

#[derive(Component)]
pub struct TutorialHighlight;

pub fn initialize_tutorial_system(mut commands: Commands) {
    let progress = TutorialProgress::load_or_default();
    commands.insert_resource(progress);
    commands.insert_resource(TutorialState::default());
}

pub fn tutorial_main_menu_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tutorial_state: ResMut<TutorialState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    existing_tutorial_ui: Query<Entity, With<TutorialUI>>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) && !tutorial_state.active {
        // Clear any existing tutorial UI
        for entity in existing_tutorial_ui.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        // Start tutorial
        tutorial_state.active = true;
        tutorial_state.current_step = 0;
        tutorial_state.completed_steps.clear();
        
        show_tutorial_intro(&mut commands, &progress);
        info!("Tutorial started - press T to start tutorial mode");
    }
    
    if tutorial_state.active && keyboard.just_pressed(KeyCode::Space) {
        // Begin tutorial game
        tutorial_state.current_step = 1;
        next_state.set(GameState::Setup);
    }
    
    if keyboard.just_pressed(KeyCode::Escape) && tutorial_state.active {
        // Skip tutorial
        tutorial_state.skip_tutorial = true;
        tutorial_state.active = false;
        for entity in existing_tutorial_ui.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn show_tutorial_intro(commands: &mut Commands, progress: &TutorialProgress) {
    let intro_text = if progress.tutorial_completed {
        "TUTORIAL REFRESHER\n\n\
         Welcome back! You've completed the tutorial before.\n\
         This will be a quick refresher of the game mechanics.\n\n\
         Press SPACE to start tutorial\n\
         Press ESC to skip\n\
         Press T to exit tutorial mode"
    } else {
        "WELCOME TO VITICULTURE!\n\n\
         This interactive tutorial will teach you how to play.\n\
         You'll learn step-by-step through a guided game.\n\n\
         Tutorial covers:\n\
         • Basic worker placement\n\
         • Planting vines and making wine\n\
         • Fulfilling wine orders for victory\n\
         • Game phases and timing\n\n\
         Press SPACE to start tutorial\n\
         Press ESC to skip\n\
         Press T to exit tutorial mode"
    };
    
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::from(Srgba::new(0.0, 0.0, 0.0, 0.8)).into(),
            z_index: ZIndex::Global(1000),
            ..default()
        },
        TutorialUI,
    )).with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(500.0),
                padding: UiRect::all(Val::Px(30.0)),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.1, 0.1, 0.1, 0.95)).into(),
            ..default()
        }).with_children(|panel| {
            panel.spawn(TextBundle::from_section(
                intro_text,
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
    });
}

pub fn tutorial_guidance_system(
    mut tutorial_state: ResMut<TutorialState>,
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    existing_tutorial_ui: Query<Entity, With<TutorialUI>>,
    workers: Query<&Worker>,
    turn_order: Res<TurnOrder>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !tutorial_state.active || tutorial_state.skip_tutorial {
        return;
    }
    
    // Clean existing tutorial UI
    for entity in existing_tutorial_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // Get current tutorial step
    if let Some(step) = get_tutorial_step(tutorial_state.current_step, current_state.get()) {
        show_tutorial_step(&mut commands, &step);
        
        // Check if step is completed
        if check_step_completion(&step, &workers, &turn_order, &keyboard, current_state.get()) {
            tutorial_state.completed_steps.push(step.id);
            tutorial_state.current_step += 1;
            
            show_step_completion(&mut commands, &step.completion_message);
            
            // Check if tutorial is finished
            if tutorial_state.current_step > 12 {
                complete_tutorial(&mut tutorial_state, &mut commands);
            }
        }
    }
}

fn get_tutorial_step(step_num: usize, game_state: &GameState) -> Option<TutorialStep> {
    match step_num {
        1 => Some(TutorialStep {
            id: 1,
            title: "Game Setup".to_string(),
            description: "The game is being set up. You start with 2 regular workers, 1 grande worker, and 3 lira.".to_string(),
            action_required: TutorialAction::Automatic,
            highlight_element: None,
            completion_message: "Great! The game is ready to begin.".to_string(),
        }),
        2 if matches!(game_state, GameState::Spring) => Some(TutorialStep {
            id: 2,
            title: "Spring Phase - Wake-up Order".to_string(),
            description: "Each spring, choose when to wake up (1-7). Earlier gives better bonuses but later turn order.\nPress SPACE to auto-assign for now.".to_string(),
            action_required: TutorialAction::PressKey(KeyCode::Space),
            highlight_element: Some("wake_up".to_string()),
            completion_message: "Good! You've set the wake-up order and received bonuses.".to_string(),
        }),
        3 if matches!(game_state, GameState::Summer) => Some(TutorialStep {
            id: 3,
            title: "Summer Phase - Income Actions".to_string(),
            description: "Summer is for income and preparation. Try drawing a vine card first.\nClick the 'Draw Vine' action or button.".to_string(),
            action_required: TutorialAction::PlaceWorker(ActionSpace::DrawVine),
            highlight_element: Some("draw_vine".to_string()),
            completion_message: "Excellent! You drew a vine card. Vine cards are planted to grow grapes.".to_string(),
        }),
        4 if matches!(game_state, GameState::Summer) => Some(TutorialStep {
            id: 4,
            title: "Plant Your First Vine".to_string(),
            description: "Now plant the vine card you drew. This costs lira but lets you harvest grapes later.\nClick 'Plant Vine' action.".to_string(),
            action_required: TutorialAction::PlaceWorker(ActionSpace::PlantVine),
            highlight_element: Some("plant_vine".to_string()),
            completion_message: "Great! You planted a vine in your vineyard. It will produce grapes when harvested.".to_string(),
        }),
        5 if matches!(game_state, GameState::Summer) => Some(TutorialStep {
            id: 5,
            title: "End Your Turn".to_string(),
            description: "You've used your workers. Press ENTER to end your turn and let other players act.".to_string(),
            action_required: TutorialAction::PressKey(KeyCode::Enter),
            highlight_element: None,
            completion_message: "Good! Your turn is over. The game will continue to Fall phase.".to_string(),
        }),
        6 if matches!(game_state, GameState::Fall) => Some(TutorialStep {
            id: 6,
            title: "Fall Phase - Automatic Harvest".to_string(),
            description: "Fall automatically harvests grapes from all planted vines. No actions needed!\nPress SPACE to continue to Winter.".to_string(),
            action_required: TutorialAction::PressKey(KeyCode::Space),
            highlight_element: None,
            completion_message: "Perfect! Your vines produced grapes automatically.".to_string(),
        }),
        7 if matches!(game_state, GameState::Winter) => Some(TutorialStep {
            id: 7,
            title: "Winter Phase - Production".to_string(),
            description: "Winter is for production and scoring. First, get a wine order to fulfill.\nClick 'Draw Wine Order' action.".to_string(),
            action_required: TutorialAction::PlaceWorker(ActionSpace::DrawWineOrder),
            highlight_element: Some("draw_wine_order".to_string()),
            completion_message: "Great! Wine orders show what customers want and reward victory points.".to_string(),
        }),
        8 if matches!(game_state, GameState::Winter) => Some(TutorialStep {
            id: 8,
            title: "Make Wine from Grapes".to_string(),
            description: "Convert your grapes into wine. Wine is needed to fulfill orders.\nClick 'Make Wine' action.".to_string(),
            action_required: TutorialAction::PlaceWorker(ActionSpace::MakeWine),
            highlight_element: Some("make_wine".to_string()),
            completion_message: "Excellent! You converted grapes into wine. Wine can fulfill customer orders.".to_string(),
        }),
        9 if matches!(game_state, GameState::Winter) => Some(TutorialStep {
            id: 9,
            title: "Fulfill Your First Order".to_string(),
            description: "Complete a wine order to earn victory points! This is how you win the game.\nClick 'Fill Order' action.".to_string(),
            action_required: TutorialAction::PlaceWorker(ActionSpace::FillOrder),
            highlight_element: Some("fill_order".to_string()),
            completion_message: "Fantastic! You earned victory points. First to 20 VP wins!".to_string(),
        }),
        10 => Some(TutorialStep {
            id: 10,
            title: "Help System".to_string(),
            description: "Learn to use the help system. Press F1 to see contextual help.\nPress F2 for quick reference.".to_string(),
            action_required: TutorialAction::PressKey(KeyCode::F1),
            highlight_element: None,
            completion_message: "Perfect! The help system provides guidance anytime you need it.".to_string(),
        }),
        11 => Some(TutorialStep {
            id: 11,
            title: "Game Statistics".to_string(),
            description: "View your game statistics to track improvement. Press TAB to open stats.".to_string(),
            action_required: TutorialAction::PressKey(KeyCode::Tab),
            highlight_element: None,
            completion_message: "Great! Statistics help you track your progress and improvement.".to_string(),
        }),
        12 => Some(TutorialStep {
            id: 12,
            title: "Tutorial Complete!".to_string(),
            description: "Congratulations! You've learned the basics of Viticulture.\n\nKey concepts:\n• Summer: Income and preparation\n• Winter: Production and scoring\n• Plant vines → Harvest grapes → Make wine → Fulfill orders → Win!\n\nPress SPACE to finish tutorial.".to_string(),
            action_required: TutorialAction::PressKey(KeyCode::Space),
            highlight_element: None,
            completion_message: "Tutorial completed! You're ready to play on your own.".to_string(),
        }),
        _ => None,
    }
}

fn show_tutorial_step(commands: &mut Commands, step: &TutorialStep) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(400.0),
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.0, 0.3, 0.0, 0.95)).into(),
            z_index: ZIndex::Global(800),
            ..default()
        },
        TutorialUI,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            format!("📚 TUTORIAL - Step {}\n\n{}\n\n{}", 
                    step.id, step.title, step.description),
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

fn check_step_completion(
    step: &TutorialStep,
    workers: &Query<&Worker>,
    turn_order: &TurnOrder,
    keyboard: &ButtonInput<KeyCode>,
    game_state: &GameState,
) -> bool {
    match &step.action_required {
        TutorialAction::PressKey(key) => keyboard.just_pressed(*key),
        TutorialAction::PlaceWorker(action) => {
            if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
                workers.iter().any(|w| w.owner == *current_player_id && w.placed_at == Some(*action))
            } else {
                false
            }
        }
        TutorialAction::Automatic => true,
        TutorialAction::CompletePhase => {
            // Check if phase changed (simplified)
            matches!(game_state, GameState::Summer | GameState::Winter | GameState::Fall)
        }
        _ => false, // Other actions not implemented in this simple version
    }
}

fn show_step_completion(commands: &mut Commands, message: &str) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                right: Val::Px(20.0),
                width: Val::Px(300.0),
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.0, 0.6, 0.0, 0.9)).into(),
            z_index: ZIndex::Global(900),
            ..default()
        },
        TutorialUI,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            format!("✅ {}", message),
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

fn complete_tutorial(tutorial_state: &mut TutorialState, commands: &mut Commands) {
    tutorial_state.active = false;
    
    // Save tutorial completion
    let mut progress = TutorialProgress::load_or_default();
    progress.tutorial_completed = true;
    progress.times_played += 1;
    progress.save();
    
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::from(Srgba::new(0.0, 0.5, 0.0, 0.8)).into(),
            z_index: ZIndex::Global(1000),
            ..default()
        },
        TutorialUI,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "🎉 TUTORIAL COMPLETED! 🎉\n\n\
             You've learned the basics of Viticulture!\n\
             Continue playing to master advanced strategies.\n\n\
             Press SPACE to return to main menu\n\
             Press ESC to continue current game",
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
    
    info!("Tutorial completed successfully!");
}

pub fn tutorial_cleanup_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tutorial_state: ResMut<TutorialState>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    tutorial_ui: Query<Entity, With<TutorialUI>>,
) {
    if !tutorial_state.active {
        return;
    }
    
    // Clean up tutorial UI when ESC is pressed
    if keyboard.just_pressed(KeyCode::Escape) {
        tutorial_state.active = false;
        for entity in tutorial_ui.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
    
    // Return to main menu when tutorial is complete
    if keyboard.just_pressed(KeyCode::Space) && !tutorial_state.active {
        for entity in tutorial_ui.iter() {
            commands.entity(entity).despawn_recursive();
        }
        next_state.set(GameState::MainMenu);
    }
}