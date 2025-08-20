use bevy::prelude::*;
use bevy::audio::Volume;

#[derive(Resource)]
pub struct AudioAssets {
    pub worker_place: Handle<AudioSource>,
    pub card_draw: Handle<AudioSource>,
    pub harvest: Handle<AudioSource>,
    pub wine_make: Handle<AudioSource>,
    pub victory_point: Handle<AudioSource>,
    pub lira_gain: Handle<AudioSource>,
    pub error: Handle<AudioSource>,
    pub phase_change: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct AudioSettings {
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub enabled: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sfx_volume: 0.7,
            music_volume: 0.3,
            enabled: true,
        }
    }
}

#[derive(Component)]
pub struct BackgroundMusic;

pub fn load_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let audio_assets = AudioAssets {
        worker_place: asset_server.load("audio/worker_place.ogg"),
        card_draw: asset_server.load("audio/card_draw.ogg"),
        harvest: asset_server.load("audio/harvest.ogg"),
        wine_make: asset_server.load("audio/wine_make.ogg"),
        victory_point: asset_server.load("audio/victory_point.ogg"),
        lira_gain: asset_server.load("audio/lira_gain.ogg"),
        error: asset_server.load("audio/error.ogg"),
        phase_change: asset_server.load("audio/phase_change.ogg"),
    };
    
    commands.insert_resource(audio_assets);
    commands.insert_resource(AudioSettings::default());
}

pub fn play_sfx(
    commands: &mut Commands,
    audio_assets: &Res<AudioAssets>,
    settings: &Res<AudioSettings>,
    sound: AudioType,
) {
    if !settings.enabled || settings.sfx_volume <= 0.0 {
        return;
    }
    
    let source = match sound {
        AudioType::WorkerPlace => &audio_assets.worker_place,
        AudioType::CardDraw => &audio_assets.card_draw,
        AudioType::Harvest => &audio_assets.harvest,
        AudioType::WineMake => &audio_assets.wine_make,
        AudioType::VictoryPoint => &audio_assets.victory_point,
        AudioType::LiraGain => &audio_assets.lira_gain,
        AudioType::Error => &audio_assets.error,
        AudioType::PhaseChange => &audio_assets.phase_change,
    };
    
    commands.spawn(AudioBundle {
        source: source.clone(),
        settings: PlaybackSettings {
            volume: Volume::new(settings.sfx_volume),
            mode: bevy::audio::PlaybackMode::Despawn,
            ..default()
        },
    });
}

#[derive(Clone, Copy)]
pub enum AudioType {
    WorkerPlace,
    CardDraw,
    Harvest,
    WineMake,
    VictoryPoint,
    LiraGain,
    Error,
    PhaseChange,
}

pub fn start_background_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
    music_query: Query<Entity, With<BackgroundMusic>>,
) {
    if music_query.is_empty() && settings.enabled && settings.music_volume > 0.0 {
        commands.spawn((
            AudioBundle {
                source: asset_server.load("audio/background_music.ogg"),
                settings: PlaybackSettings {
                    volume: Volume::new(settings.music_volume),
                    mode: bevy::audio::PlaybackMode::Loop,
                    ..default()
                },
            },
            BackgroundMusic,
        ));
    }
}

pub fn update_audio_volume(
    settings: Res<AudioSettings>,
    mut music_query: Query<&mut AudioSink, With<BackgroundMusic>>,
) {
    if settings.is_changed() {
        for sink in music_query.iter_mut() {
            if settings.enabled {
                sink.set_volume(settings.music_volume);
            } else {
                sink.set_volume(0.0);
            }
        }
    }
}
