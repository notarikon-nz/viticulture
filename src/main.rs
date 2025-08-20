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
        .insert_resource(GameSettings::default())
        .insert_resource(CardDecks::new())
        .insert_resource(AISettings::default())
        .insert_resource(GameValidation::default())
        .insert_resource(PerformanceSettings::default())
        .insert_resource(FrameCache::default())
        .insert_resource(EndGameScoring::default())
        .insert_resource(BalanceTestResults::default())
        .insert_resource(AutoTestConfig::default())
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, (setup_camera, load_assets, load_audio_assets))
        .add_systems(
            Update, (
                main_menu_system.run_if(in_state(GameState::MainMenu)),
                (setup_game_system, cleanup_entities_system, setup_ai_players).run_if(in_state(GameState::Setup)),
                (spring_system, start_background_music).run_if(in_state(GameState::Spring)),
                mouse_input_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                worker_placement_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                ai_decision_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                update_audio_volume.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                fall_system.run_if(in_state(GameState::Fall)),
                (check_victory_system, calculate_final_scores).run_if(in_state(GameState::GameOver)),
                ui_button_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                cached_ui_update_system,
                culled_sprite_system,
                animate_text_system,
                ui_game_over_system,
                // Bug fixes and maintenance
                fix_worker_state_system,
                fix_card_deck_system,
                fix_resource_overflow_system,
                fix_turn_order_system,
                fix_action_space_consistency_system,
                validate_game_state_system,
                emergency_recovery_system,
                // Balance testing
                auto_balance_test_system,
                track_action_usage_system,
                dynamic_difficulty_system,
                apply_balance_tweaks,
                fast_test_mode_system,
                game_length_tracking_system,
                performance_monitor_system,
            ),
        )
        .run();
}