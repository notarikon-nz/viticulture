use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::*;
use crate::systems::audio::*;

#[derive(Serialize, Deserialize, Resource, Clone)]
pub struct UserSettings {
    pub audio_enabled: bool,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub auto_save_enabled: bool,
    pub show_tooltips: bool,
    pub performance_mode: bool,
    pub ai_difficulty: u8, // 1 = Beginner, 2 = Intermediate
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            audio_enabled: true,
            sfx_volume: 0.7,
            music_volume: 0.3,
            auto_save_enabled: true,
            show_tooltips: true,
            performance_mode: false,
            ai_difficulty: 1,
        }
    }
}

impl UserSettings {
    pub fn load_or_default() -> Self {
        match std::fs::read_to_string("viticulture_settings.json") {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
    
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("viticulture_settings.json", json);
        }
    }
}

#[derive(Component)]
pub struct SettingsPanel;

#[derive(Component)]
pub struct SettingsButton {
    pub setting_type: SettingType,
}

#[derive(Clone, Copy)]
pub enum SettingType {
    ToggleAudio,
    SfxVolumeUp,
    SfxVolumeDown,
    MusicVolumeUp,
    MusicVolumeDown,
    ToggleAutoSave,
    ToggleTooltips,
    TogglePerformance,
    AiDifficultyUp,
    AiDifficultyDown,
    ResetSettings,
    CloseSettings,
}

pub fn initialize_settings_system(mut commands: Commands) {
    let settings = UserSettings::load_or_default();
    commands.insert_resource(settings);
}

pub fn settings_menu_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    settings: Res<UserSettings>,
    existing_settings: Query<Entity, With<SettingsPanel>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if existing_settings.is_empty() {
            show_settings_menu(&mut commands, &settings);
        } else {
            hide_settings_menu(&mut commands, existing_settings);
        }
    }
}

fn show_settings_menu(commands: &mut Commands, settings: &UserSettings) {
    // Background overlay
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::from(Srgba::new(0.0, 0.0, 0.0, 0.8)).into(),
            z_index: ZIndex::Global(200),
            ..default()
        },
        SettingsPanel,
    )).with_children(|parent| {
        // Settings panel
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(400.0),
                height: Val::Px(500.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            background_color: Color::from(Srgba::new(0.1, 0.1, 0.1, 0.95)).into(),
            ..default()
        }).with_children(|panel| {
            // Title
            panel.spawn(TextBundle::from_section(
                "⚙️ SETTINGS",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            }));
            
            // Audio Section
            create_setting_row(panel, "🔊 Audio", &format!("{}", if settings.audio_enabled { "ON" } else { "OFF" }), SettingType::ToggleAudio);
            create_volume_row(panel, "🎵 SFX Volume", settings.sfx_volume, SettingType::SfxVolumeDown, SettingType::SfxVolumeUp);
            create_volume_row(panel, "🎼 Music Volume", settings.music_volume, SettingType::MusicVolumeDown, SettingType::MusicVolumeUp);
            
            // Game Section
            create_setting_row(panel, "💾 Auto-Save", &format!("{}", if settings.auto_save_enabled { "ON" } else { "OFF" }), SettingType::ToggleAutoSave);
            create_setting_row(panel, "💡 Tooltips", &format!("{}", if settings.show_tooltips { "ON" } else { "OFF" }), SettingType::ToggleTooltips);
            create_setting_row(panel, "⚡ Performance Mode", &format!("{}", if settings.performance_mode { "ON" } else { "OFF" }), SettingType::TogglePerformance);
            
            // AI Section
            create_difficulty_row(panel, "🤖 AI Difficulty", settings.ai_difficulty);
            
            // Action Buttons
            panel.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                ..default()
            }).with_children(|actions| {
                create_action_button(actions, "Reset", SettingType::ResetSettings, Color::from(Srgba::new(0.8, 0.3, 0.3, 1.0)));
                create_action_button(actions, "Close", SettingType::CloseSettings, Color::from(Srgba::new(0.3, 0.8, 0.3, 1.0)));
            });
            
            // Controls help
            panel.spawn(TextBundle::from_section(
                "\nControls:\nESC - Settings\nTAB - Statistics\nF5 - Save Game\nF9 - Load Game\nF10 - Balance Test\nF12 - Emergency Exit",
                TextStyle {
                    font_size: 12.0,
                    color: Color::from(Srgba::new(0.7, 0.7, 0.7, 1.0)),
                    ..default()
                },
            ));
        });
    });
}

fn create_setting_row(parent: &mut ChildBuilder, label: &str, value: &str, toggle_type: SettingType) {
    parent.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        },
        ..default()
    }).with_children(|row| {
        row.spawn(TextBundle::from_section(
            label,
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        
        row.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(60.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::from(Srgba::new(0.3, 0.3, 0.3, 1.0)).into(),
                ..default()
            },
            SettingsButton { setting_type: toggle_type },
        )).with_children(|button| {
            button.spawn(TextBundle::from_section(
                value,
                TextStyle {
                    font_size: 14.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
    });
}

fn create_volume_row(parent: &mut ChildBuilder, label: &str, volume: f32, down_type: SettingType, up_type: SettingType) {
    parent.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        },
        ..default()
    }).with_children(|row| {
        row.spawn(TextBundle::from_section(
            label,
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        
        row.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).with_children(|controls| {
            // Decrease button
            controls.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::right(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: Color::from(Srgba::new(0.5, 0.2, 0.2, 1.0)).into(),
                    ..default()
                },
                SettingsButton { setting_type: down_type },
            )).with_children(|button| {
                button.spawn(TextBundle::from_section("-", TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                }));
            });
            
            // Volume display
            controls.spawn(TextBundle::from_section(
                &format!("{:.0}%", volume * 100.0),
                TextStyle {
                    font_size: 14.0,
                    color: Color::WHITE,
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::horizontal(Val::Px(10.0)),
                ..default()
            }));
            
            // Increase button
            controls.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::left(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: Color::from(Srgba::new(0.2, 0.5, 0.2, 1.0)).into(),
                    ..default()
                },
                SettingsButton { setting_type: up_type },
            )).with_children(|button| {
                button.spawn(TextBundle::from_section("+", TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                }));
            });
        });
    });
}

fn create_difficulty_row(parent: &mut ChildBuilder, label: &str, difficulty: u8) {
    let difficulty_text = match difficulty {
        1 => "Beginner",
        2 => "Intermediate",
        _ => "Unknown",
    };
    
    create_volume_row(parent, label, difficulty as f32, SettingType::AiDifficultyDown, SettingType::AiDifficultyUp);
}

fn create_action_button(parent: &mut ChildBuilder, text: &str, setting_type: SettingType, color: Color) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(80.0),
                height: Val::Px(35.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: color.into(),
            ..default()
        },
        SettingsButton { setting_type },
    )).with_children(|button| {
        button.spawn(TextBundle::from_section(
            text,
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

fn hide_settings_menu(commands: &mut Commands, existing_settings: Query<Entity, With<SettingsPanel>>) {
    for entity in existing_settings.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_settings_interaction_system(
    mut interaction_query: Query<(&Interaction, &SettingsButton, &mut BackgroundColor)>,
    mut settings: ResMut<UserSettings>,
    mut commands: Commands,
    existing_settings: Query<Entity, With<SettingsPanel>>,
    mut audio_settings: ResMut<AudioSettings>,
) {
    let mut should_refresh = false;
    let mut should_close = false;

    for (interaction, settings_button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match settings_button.setting_type {
                    SettingType::ToggleAudio => {
                        settings.audio_enabled = !settings.audio_enabled;
                        audio_settings.enabled = settings.audio_enabled;
                    }
                    SettingType::SfxVolumeUp => {
                        settings.sfx_volume = (settings.sfx_volume + 0.1).min(1.0);
                        audio_settings.sfx_volume = settings.sfx_volume;
                    }
                    SettingType::SfxVolumeDown => {
                        settings.sfx_volume = (settings.sfx_volume - 0.1).max(0.0);
                        audio_settings.sfx_volume = settings.sfx_volume;
                    }
                    SettingType::MusicVolumeUp => {
                        settings.music_volume = (settings.music_volume + 0.1).min(1.0);
                        audio_settings.music_volume = settings.music_volume;
                    }
                    SettingType::MusicVolumeDown => {
                        settings.music_volume = (settings.music_volume - 0.1).max(0.0);
                        audio_settings.music_volume = settings.music_volume;
                    }
                    SettingType::ToggleAutoSave => {
                        settings.auto_save_enabled = !settings.auto_save_enabled;
                    }
                    SettingType::ToggleTooltips => {
                        settings.show_tooltips = !settings.show_tooltips;
                    }
                    SettingType::TogglePerformance => {
                        settings.performance_mode = !settings.performance_mode;
                    }
                    SettingType::AiDifficultyUp => {
                        settings.ai_difficulty = (settings.ai_difficulty + 1).min(2);
                    }
                    SettingType::AiDifficultyDown => {
                        settings.ai_difficulty = (settings.ai_difficulty - 1).max(1);
                    }
                    SettingType::ResetSettings => {
                        *settings = UserSettings::default();
                        audio_settings.enabled = settings.audio_enabled;
                        audio_settings.sfx_volume = settings.sfx_volume;
                        audio_settings.music_volume = settings.music_volume;
                    }
                    SettingType::CloseSettings => {
                        should_close = true;
                    }
                }
                
                // Save settings after any change
                settings.save();
                
                // Mark for refresh if not closing
                if !matches!(settings_button.setting_type, SettingType::CloseSettings) {
                    should_refresh = true;
                }
            }
            Interaction::Hovered => {
                *color = Color::from(Srgba::new(0.9, 0.9, 0.9, 1.0)).into();
            }
            Interaction::None => {
                *color = Color::from(Srgba::new(0.3, 0.3, 0.3, 1.0)).into();
            }
        }
    }
    
    // Handle cleanup and refresh outside the loop
    if should_close {
        hide_settings_menu(&mut commands, existing_settings);
    } else if should_refresh {
        hide_settings_menu(&mut commands, existing_settings);
        show_settings_menu(&mut commands, &settings);
    }
}