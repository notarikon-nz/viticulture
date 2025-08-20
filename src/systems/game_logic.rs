use bevy::prelude::*;
use crate::components::*;
use crate::systems::*;
use crate::systems::ui::setup_ui;
use crate::systems::animations::spawn_animated_text;

const YELLOW: Srgba = Srgba::new(1.0, 1.0, 0.0, 1.0);
const GOLD: Srgba = Srgba::new(1.0, 0.84, 0.0, 1.0);

pub fn spring_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut turn_order: ResMut<TurnOrder>,
    mut workers: Query<&mut Worker>,
    mut action_spaces: Query<&mut ActionSpaceSlot>,
    config: ResMut<GameConfig>,
    mut commands: Commands,
    text_query: Query<Entity, (With<Text>, Without<UIPanel>)>,
    ui_query: Query<Entity, With<UIPanel>>,
    mut hands: Query<&mut Hand>,
    mut players: Query<&mut Player>,
    mut card_decks: ResMut<CardDecks>,
) {
    if ui_query.is_empty() {
        setup_ui(&mut commands);
    }
    
    if text_query.is_empty() {
        commands.spawn(Text2dBundle {
            text: Text::from_section(
                format!("SPRING PHASE - YEAR {}\nChoose wake-up times (1-7)\nPress SPACE to auto-assign and continue", config.current_year),
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 200.0, 1000.0)),
            ..default()
        });
    }
    
    if keyboard.just_pressed(KeyCode::Space) {
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        
        let mut wake_up_assignments = Vec::new();
        for (i, player_id) in turn_order.players.iter().enumerate() {
            wake_up_assignments.push((*player_id, (i + 1) as u8));
        }
        turn_order.set_wake_up_order(wake_up_assignments);
        
        for (player_id, _) in &turn_order.wake_up_order {
            if let Some(bonus) = turn_order.get_wake_up_bonus(*player_id) {
                apply_wake_up_bonus(*player_id, bonus, &mut hands, &mut players, &mut card_decks, &mut commands);
            }
        }
        
        for mut worker in workers.iter_mut() {
            worker.placed_at = None;
            let player_id = worker.owner.0;
            let y_offset = if worker.is_grande { -170.0 } else { -200.0 };
            worker.position = Vec2::new(-500.0 + (player_id as f32 * 100.0), y_offset);
        }
        
        for mut space in action_spaces.iter_mut() {
            space.occupied_by = None;
            space.bonus_worker_slot = None;
        }
        
        turn_order.current_player = 0;
        next_state.set(GameState::Summer);
    }
}

fn apply_wake_up_bonus(
    player_id: PlayerId,
    bonus: WakeUpBonus,
    hands: &mut Query<&mut Hand>,
    players: &mut Query<&mut Player>,
    card_decks: &mut ResMut<CardDecks>,
    commands: &mut Commands,
) {
    match bonus {
        WakeUpBonus::DrawVineCard => {
            if let Some(mut hand) = hands.iter_mut().find(|h| h.owner == player_id) {
                if let Some(card) = card_decks.draw_vine_card() {
                    hand.vine_cards.push(card);
                    spawn_animated_text(commands, player_id, "Wake-up: +Vine", Color::from(Srgba::new(0.2, 0.8, 0.2, 1.0)));
                }
            }
        }
        WakeUpBonus::GainLira(amount) => {
            if let Some(mut player) = players.iter_mut().find(|p| p.id == player_id) {
                player.gain_lira(amount);
                spawn_animated_text(commands, player_id, &format!("Wake-up: +{} Lira", amount), Color::from(GOLD));
            }
        }
        WakeUpBonus::GainVictoryPoint => {
            if let Some(mut player) = players.iter_mut().find(|p| p.id == player_id) {
                player.gain_victory_points(1);
                spawn_animated_text(commands, player_id, "Wake-up: +1 VP", Color::from(YELLOW));
            }
        }
        WakeUpBonus::DrawWineOrderCard => {
            if let Some(mut hand) = hands.iter_mut().find(|h| h.owner == player_id) {
                if let Some(card) = card_decks.draw_wine_order_card() {
                    hand.wine_order_cards.push(card);
                    spawn_animated_text(commands, player_id, "Wake-up: +Order", Color::from(Srgba::new(0.6, 0.2, 0.8, 1.0)));
                }
            }
        }
        _ => {}
    }
}

pub fn execute_action(
    action: ActionSpace,
    player_id: PlayerId,
    hands: &mut Query<&mut Hand>,
    vineyards: &mut Query<&mut Vineyard>,
    players: &mut Query<&mut Player>,
    card_decks: &mut ResMut<CardDecks>,
    commands: &mut Commands,
    audio_assets: &Res<AudioAssets>,
    audio_settings: &Res<AudioSettings>,
) {
    let mut hand = hands.iter_mut().find(|h| h.owner == player_id);
    let mut vineyard = vineyards.iter_mut().find(|v| v.owner == player_id);
    let mut player = players.iter_mut().find(|p| p.id == player_id);
    
    match action {
        ActionSpace::DrawVine => {
            if let (Some(hand), Some(card)) = (hand.as_mut(), card_decks.draw_vine_card()) {
                hand.vine_cards.push(card);
                play_sfx(commands, audio_assets, audio_settings, AudioType::CardDraw);
                spawn_animated_text(commands, player_id, "+Vine", Color::from(Srgba::new(0.2, 0.8, 0.2, 1.0)));
            }
        }
        ActionSpace::DrawWineOrder => {
            if let (Some(hand), Some(card)) = (hand.as_mut(), card_decks.draw_wine_order_card()) {
                hand.wine_order_cards.push(card);
                play_sfx(commands, audio_assets, audio_settings, AudioType::CardDraw);
                spawn_animated_text(commands, player_id, "+Order", Color::from(Srgba::new(0.6, 0.2, 0.8, 1.0)));
            }
        }
        ActionSpace::PlantVine => {
            if let (Some(hand), Some(vineyard)) = (hand.as_mut(), vineyard.as_mut()) {
                if !hand.vine_cards.is_empty() {
                    let vine_card = hand.vine_cards.remove(0);
                    let structures = Vec::new();
                    for i in 0..9 {
                        if vineyard.can_plant_vine(i, &vine_card, &structures) {
                            if vineyard.plant_vine(i, vine_card.clone(), &structures) {
                                spawn_animated_text(commands, player_id, "Planted!", Color::from(Srgba::new(0.4, 0.8, 0.4, 1.0)));
                                break;
                            }
                        }
                    }
                }
            }
        }
        ActionSpace::BuildStructure => {
            if let Some(vineyard) = vineyard.as_mut() {
                if vineyard.can_build_structure(StructureType::Trellis) {
                    if vineyard.build_structure(StructureType::Trellis) {
                        spawn_animated_text(commands, player_id, "+Structure", Color::from(Srgba::new(0.8, 0.8, 0.2, 1.0)));
                    }
                }
            }
        }
        ActionSpace::Harvest => {
            if let Some(vineyard) = vineyard.as_mut() {
                let structures = Vec::new();
                let gained = vineyard.harvest_grapes(&structures);
                if gained > 0 {
                    play_sfx(commands, audio_assets, audio_settings, AudioType::Harvest);
                    spawn_animated_text(commands, player_id, &format!("+{} Grapes", gained), Color::from(Srgba::new(0.8, 0.4, 0.8, 1.0)));
                }
            }
        }
        ActionSpace::MakeWine => {
            if let Some(vineyard) = vineyard.as_mut() {
                // Enhanced wine making - can make blush and sparkling wine
                let red_available = vineyard.red_grapes;
                let white_available = vineyard.white_grapes;
                
                if red_available >= 2 && white_available >= 2 {
                    // Make sparkling wine (requires 1 red + 1 white, produces value 4 wine)
                    vineyard.red_grapes -= 1;
                    vineyard.white_grapes -= 1;
                    vineyard.red_wine += 2; // Sparkling wine stored as red wine for simplicity
                    play_sfx(commands, audio_assets, audio_settings, AudioType::WineMake);
                    spawn_animated_text(commands, player_id, "+Sparkling Wine", Color::from(Srgba::new(0.9, 0.7, 0.2, 1.0)));
                } else if red_available >= 1 && white_available >= 1 {
                    // Make blush wine (1 red + 1 white = 1 blush)
                    vineyard.red_grapes -= 1;
                    vineyard.white_grapes -= 1;
                    vineyard.white_wine += 1; // Blush stored as white wine for simplicity
                    play_sfx(commands, audio_assets, audio_settings, AudioType::WineMake);
                    spawn_animated_text(commands, player_id, "+Blush Wine", Color::from(Srgba::new(0.9, 0.5, 0.6, 1.0)));
                } else {
                    // Regular wine making
                    let red_to_use = if red_available > 0 { 1 } else { 0 };
                    let white_to_use = if white_available > 0 { 1 } else { 0 };
                    
                    if vineyard.make_wine(red_to_use, white_to_use) {
                        let total_wine = red_to_use + white_to_use;
                        if total_wine > 0 {
                            play_sfx(commands, audio_assets, audio_settings, AudioType::WineMake);
                            spawn_animated_text(commands, player_id, &format!("+{} Wine", total_wine), Color::from(Srgba::new(0.7, 0.2, 0.2, 1.0)));
                        }
                    }
                }
            }
        }
        ActionSpace::FillOrder => {
            if let (Some(hand), Some(vineyard), Some(player)) = (hand.as_mut(), vineyard.as_mut(), player.as_mut()) {
                if !hand.wine_order_cards.is_empty() {
                    let order = &hand.wine_order_cards[0];
                    if vineyard.can_fulfill_order(order) {
                        let order = hand.wine_order_cards.remove(0);
                        vineyard.fulfill_order(&order);
                        player.gain_victory_points(order.victory_points);
                        player.gain_lira(order.payout);
                        play_sfx(commands, audio_assets, audio_settings, AudioType::VictoryPoint);
                        spawn_animated_text(commands, player_id, &format!("+{} VP", order.victory_points), Color::from(YELLOW));
                        if order.payout > 0 {
                            play_sfx(commands, audio_assets, audio_settings, AudioType::LiraGain);
                            spawn_animated_text(commands, player_id, &format!("+{} Lira", order.payout), Color::from(GOLD));
                        }
                    }
                }
            }
        }
        ActionSpace::GiveTour => {
            if let Some(player) = player.as_mut() {
                let bonus_lira = 2;
                // Check for Tasting Room structure bonus
                // TODO: Query actual structures when implemented
                player.gain_lira(bonus_lira);
                play_sfx(commands, audio_assets, audio_settings, AudioType::LiraGain);
                spawn_animated_text(commands, player_id, &format!("+{} Lira", bonus_lira), Color::from(GOLD));
            }
        }
        ActionSpace::SellGrapes => {
            if let (Some(vineyard), Some(player)) = (vineyard.as_mut(), player.as_mut()) {
                let grapes_sold = vineyard.red_grapes + vineyard.white_grapes;
                if grapes_sold > 0 {
                    player.gain_lira(grapes_sold);
                    vineyard.red_grapes = 0;
                    vineyard.white_grapes = 0;
                    play_sfx(commands, audio_assets, audio_settings, AudioType::LiraGain);
                    spawn_animated_text(commands, player_id, &format!("+{} Lira", grapes_sold), Color::from(GOLD));
                }
            }
        }
        ActionSpace::TrainWorker => {
            if let Some(player) = player.as_mut() {
                if player.lira >= 4 {
                    player.lira -= 4;
                    player.workers += 1;
                    spawn_animated_text(commands, player_id, "+Worker", Color::from(Srgba::new(0.5, 0.8, 1.0, 1.0)));
                }
            }
        }
        _ => {}
    }
}

pub fn fall_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut vineyards: Query<&mut Vineyard>,
    mut commands: Commands,
    text_query: Query<Entity, (With<Text>, Without<UIPanel>)>,
) {
    if text_query.is_empty() {
        commands.spawn(Text2dBundle {
            text: Text::from_section(
                "FALL PHASE\nAutomatic harvest from planted vines\n\nPress SPACE to continue to Winter",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 200.0, 1000.0)),
            ..default()
        });
    }
    
    if keyboard.just_pressed(KeyCode::Space) {
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        
        let structures = Vec::new();
        for mut vineyard in vineyards.iter_mut() {
            vineyard.harvest_grapes(&structures);
        }
        next_state.set(GameState::Winter);
    }
}

pub fn check_victory_system(
    players: Query<&Player>,
    vineyards: Query<&Vineyard>,
    mut next_state: ResMut<NextState<GameState>>,
    config: Res<GameConfig>,
    mut commands: Commands,
    text_query: Query<Entity, With<Text>>,
) {
    let mut winner: Option<&Player> = None;
    let mut highest_vp = 0;
    
    for player in players.iter() {
        let mut total_vp = player.victory_points;
        
        // Add end-game bonuses
        if let Some(vineyard) = vineyards.iter().find(|v| v.owner == player.id) {
            let structures = Vec::new(); // TODO: Query actual structures
            total_vp += vineyard.get_end_game_bonus(&structures);
        }
        
        if total_vp >= config.target_victory_points {
            if total_vp > highest_vp {
                highest_vp = total_vp;
                winner = Some(player);
            }
        }
    }
    
    let year_limit_reached = config.current_year > config.max_years;
    
    if winner.is_some() || year_limit_reached {
        if winner.is_none() && year_limit_reached {
            for player in players.iter() {
                let mut total_vp = player.victory_points;
                if let Some(vineyard) = vineyards.iter().find(|v| v.owner == player.id) {
                    let structures = Vec::new();
                    total_vp += vineyard.get_end_game_bonus(&structures);
                }
                if total_vp > highest_vp {
                    highest_vp = total_vp;
                    winner = Some(player);
                }
            }
        }
        
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        
        if let Some(winning_player) = winner {
            commands.spawn(TextBundle::from_section(
                format!("GAME OVER!\n{} WINS with {} Victory Points!\n\nPress SPACE to play again", 
                        winning_player.name, highest_vp),
                TextStyle {
                    font_size: 32.0,
                    color: Color::from(GOLD),
                    ..default()
                },
            ).with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(200.0),
                left: Val::Px(100.0),
                ..default()
            }));
        }
        
        next_state.set(GameState::GameOver);
    }
}