use bevy::prelude::*;

mod components;
mod systems;

use components::*;
use systems::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(
                WindowPlugin {
                    primary_window: Some(Window {
                        title: "Viticulture".into(),
                        resolution: (1200.0, 800.0).into(),
                        ..default()
                    }),
                    ..default()
                }
            )
        )
        .init_state::<GameState>()
        .insert_resource(TurnOrder::default())
        .insert_resource(GameConfig::default())
        .insert_resource(CardDecks::new())
        .add_systems(Startup, (setup_camera, load_assets))
        .add_systems(
            Update, (
                main_menu_system.run_if(in_state(GameState::MainMenu)),
                (setup_game_system, cleanup_entities_system).run_if(in_state(GameState::Setup)),
                spring_system.run_if(in_state(GameState::Spring)),
                mouse_input_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                worker_placement_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                fall_system.run_if(in_state(GameState::Fall)),
                check_victory_system,
                ui_button_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                update_ui_system,
                update_sprites_system,
                animate_text_system,
                ui_game_over_system,
            ),
        )
        .run();
}
