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
        // Core game resources
        .insert_resource(TurnOrder::default())
        .insert_resource(GameConfig::default())
        .insert_resource(GameSettings::default())
        .insert_resource(CardDecks::new())
        .insert_resource(AISettings::default())
        .insert_resource(GameValidation::default())
        // Performance resources
        .insert_resource(PerformanceSettings::default())
        .insert_resource(FrameCache::default())
        // Game state resources
        .insert_resource(EndGameScoring::default())
        .insert_resource(BalanceTestResults::default())
        .insert_resource(AutoTestConfig::default())
        .insert_resource(SaveManager::default())
        .insert_resource(UndoSystem::default())
        .insert_resource(AnimationSettings::default())
        // Expansion resources (create them conditionally)
        .insert_resource(ExpansionSettings::default())
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, (
            setup_camera, 
            load_assets, 
            load_audio_assets, 
            initialize_settings_system, 
            initialize_session_system,
            setup_tooltips_system,
            initialize_expansion_content_system,
            initialize_achievements_system,
            initialize_onboarding_system,
            initialize_tutorial_system,
        ))
        .add_systems(
            Update, (
                main_menu_system.run_if(in_state(GameState::MainMenu)),
                (setup_game_system, setup_ai_players, setup_residual_payment_system).run_if(in_state(GameState::Setup)),
                (spring_system, start_background_music).run_if(in_state(GameState::Spring)),
                mouse_input_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                worker_placement_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),

                // Conditional AI systems - use proper run conditions
                ai_decision_system.run_if(
                    in_state(GameState::Summer)
                        .or_else(in_state(GameState::Winter))
                        .and_then(not(testing_mode_enabled))
                ),
                
                // Fast AI for testing
                fast_ai_decision_system.run_if(
                    in_state(GameState::Summer)
                        .or_else(in_state(GameState::Winter))
                        .and_then(testing_mode_enabled)
                ),
                                
                ai_decision_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                update_audio_volume.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                fall_system.run_if(in_state(GameState::Fall)),
                // Victory check runs during ALL gameplay states to detect wins immediately
                check_victory_system.run_if(
                    in_state(GameState::Spring)
                        .or_else(in_state(GameState::Summer))
                        .or_else(in_state(GameState::Fall))
                        .or_else(in_state(GameState::Winter))
                ),
                // Final scoring only runs when GameOver
                calculate_final_scores.run_if(in_state(GameState::GameOver)),
                ui_button_system.run_if(in_state(GameState::Summer).or_else(in_state(GameState::Winter))),
                
                //cached_ui_update_system,
                //culled_sprite_system,
                update_sprites_system,
                update_ui_system,

                animate_text_system,
                ui_game_over_system,
                main_menu_cleanup_system,

            ))
        .add_systems(Update, (
                apply_residual_income_system,
                apply_mama_abilities_system,
                display_player_cards_system,

                // Persistence & QoL systems
                save_game_system.run_if(not(in_state(GameState::MainMenu).or_else(in_state(GameState::GameOver)))),                
                load_game_system,
                track_session_system,
                balance::track_action_usage_system,
                update_statistics_on_game_end_system,
                display_statistics_system,
                settings_menu_system,
                handle_settings_interaction_system,
                create_snapshot_system,
                undo_action_system,
                display_undo_status_system,
            ))
        .add_systems(Update, (
                // Expansion systems
                setup_tuscany_expansion_system,
                handle_visitor_cards_system,
                setup_advanced_vineyards_system,
                apply_board_bonuses_system,
                expansion_toggle_system,
                trigger_season_event_system,
                // Tooltip systems
                tooltip_hover_system,
                tooltip_display_system,
                contextual_help_system,
                quick_reference_system,
                card_tooltip_system,
            ))
        .add_systems(Update, (
                // Bug fixes and maintenance
                fix_worker_state_system,
                fix_card_deck_system,
                fix_resource_overflow_system,
                fix_turn_order_system,
                fix_action_space_consistency_system,
                validate_game_state_system,
                emergency_recovery_system,
             ))
        .add_systems(Update, (
               // Balance testing systems
                auto_balance_test_system,
                ui_protection_system.run_if(testing_mode_enabled),
                fast_test_mode_system.run_if(testing_mode_enabled),
                unstuck_system.run_if(testing_mode_enabled),
                protected_setup_system.run_if(in_state(GameState::Setup).and_then(testing_mode_enabled)),
                
                debug_ai_setup_system.run_if(testing_mode_enabled),
                
                // Regular balance systems
                statistics::track_action_usage_system,
                dynamic_difficulty_system,
                apply_balance_tweaks,
                game_length_tracking_system,
                performance_monitor_system,
                
            ))
        .add_systems(Update, (
                year_end_aging_system,
                enforce_hand_limit_system,
                assign_temporary_worker_system,
                fall_visitor_system.run_if(in_state(GameState::Fall)),
            ))            
        .add_systems(PostUpdate, (
            despawn_marked_entities,
        ),
        )
        .run();
}

// Custom run condition functions
fn testing_mode_enabled(test_config: Res<AutoTestConfig>) -> bool {
    test_config.enabled
}

fn not_testing_mode_enabled(test_config: Res<AutoTestConfig>) -> bool {
    !test_config.enabled
}


pub fn despawn_marked_entities(
    mut commands: Commands,
    query: Query<Entity, With<MarkedForDespawn>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn(); // ← Now safe
    }
}

// Debug system to verify AI setup
pub fn debug_ai_setup_system(
    ai_players: Query<&AIPlayer>,
    players: Query<&Player>,
    test_config: Res<AutoTestConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Press F11 during testing to debug AI setup
    if test_config.enabled && keyboard.just_pressed(KeyCode::F11) {
        let total_players = players.iter().count();
        let ai_entities = ai_players.iter().count();
        let human_players = players.iter().filter(|p| !p.is_ai).count();
        let ai_players_marked = players.iter().filter(|p| p.is_ai).count();
        
        info!("🔍 AI DEBUG:");
        info!("  Total Players: {}", total_players);
        info!("  Players marked as AI: {}", ai_players_marked);
        info!("  AI entities: {}", ai_entities);
        info!("  Human players: {}", human_players);
        info!("  Expected AI: {}", test_config.ai_count);
        
        for ai in ai_players.iter() {
            info!("  🤖 AI Entity: Player {:?} (difficulty: {:?})", ai.player_id, ai.difficulty);
        }
        
        for player in players.iter() {
            info!("  👤 Player {}: {} (AI: {})", player.id.0 + 1, player.name, player.is_ai);
        }
    }
}