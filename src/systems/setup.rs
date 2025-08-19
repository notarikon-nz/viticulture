use bevy::prelude::*;
use crate::components::*;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = GameAssets {
        worker_texture: asset_server.load("worker.png"),
        vine_card_texture: asset_server.load("vine_card.png"),
        wine_order_card_texture: asset_server.load("wine_order.png"),
        field_texture: asset_server.load("field.png"),
    };
    commands.insert_resource(assets);
}

pub fn cleanup_entities_system(
    mut commands: Commands,
    entities: Query<Entity, (Without<Camera>, Without<Window>)>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn setup_game_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    config: Res<GameConfig>,
    mut turn_order: ResMut<TurnOrder>,
    text_query: Query<Entity, With<Text>>,
) {
    for entity in text_query.iter() {
        commands.entity(entity).despawn();
    }
    
    for i in 0..config.player_count {
        let player = Player::new(i, format!("Player {}", i + 1));
        let vineyard = Vineyard::new(PlayerId(i));
        let hand = Hand::new(PlayerId(i));
        
        commands.spawn(player);
        commands.spawn(vineyard);
        commands.spawn(hand);
        
        // Create regular workers + grande worker
        for w in 0..2 {
            let worker_pos = Vec2::new(-500.0 + (i as f32 * 100.0), -200.0 + (w as f32 * 30.0));
            commands.spawn((
                Worker::new(PlayerId(i), false, worker_pos),
                Clickable { size: Vec2::new(20.0, 20.0) },
            ));
        }
        
        // Add grande worker
        let grande_pos = Vec2::new(-500.0 + (i as f32 * 100.0), -170.0);
        commands.spawn((
            Worker::new(PlayerId(i), true, grande_pos),
            Clickable { size: Vec2::new(25.0, 25.0) },
        ));
        
        turn_order.players.push(PlayerId(i));
    }
    
    let action_board = ActionBoard::new();
    for space in action_board.spaces.clone() {
        commands.spawn((
            space,
            Clickable { size: Vec2::new(60.0, 30.0) },
        ));
    }
    commands.spawn(action_board);
    
    next_state.set(GameState::Spring);
}