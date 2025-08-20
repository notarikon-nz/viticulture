use bevy::prelude::*;
use crate::components::*;
use crate::systems::*;

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
    animation_settings: Res<AnimationSettings>,
) {
    if ui_query.is_empty() {
        crate::systems::ui::setup_ui(&mut commands);
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
        
        // Animate workers returning to starting positions
        for mut worker in workers.iter_mut() {
            if worker.placed_at.is_some() {
                let start_pos = worker.position;
                let player_id = worker.owner.0;
                let y_offset = if worker.is_grande { -170.0 } else { -200.0 };
                let target_pos = Vec2::new(-500.0 + (player_id as f32 * 100.0), y_offset);
                
                // This would need the worker entity, so we'd need to restructure this
                // For now, just update position directly
                worker.position = target_pos;
            }
            
            worker.placed_at = None;
        }
        
        for mut space in action_spaces.iter_mut() {
            space.occupied_by = None;
            space.bonus_worker_slot = None;
        }
        
        let mut wake_up_assignments = Vec::new();
        for (i, player_id) in turn_order.players.iter().enumerate() {
            wake_up_assignments.push((*player_id, (i + 1) as u8));
        }
        turn_order.set_wake_up_order(wake_up_assignments);
        
        for (player_id, _) in &turn_order.wake_up_order {
            if let Some(bonus) = turn_order.get_wake_up_bonus(*player_id) {
                crate::systems::game_logic::apply_wake_up_bonus(*player_id, bonus, &mut hands, &mut players, &mut card_decks, &mut commands);
            }
        }
        
        turn_order.current_player = 0;
        
        // Trigger season transition
        trigger_season_transition(&mut commands, GameState::Spring, GameState::Summer, &animation_settings);
        
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
    trackers: &mut Query<&mut ResidualPaymentTracker>,
    structures: &Query<&Structure>, 
    audio_assets: &Res<AudioAssets>,
    audio_settings: &Res<AudioSettings>,
    animation_settings: &Res<AnimationSettings>,
) {
    let mut hand = hands.iter_mut().find(|h| h.owner == player_id);
    let mut vineyard = vineyards.iter_mut().find(|v| v.owner == player_id);
    let mut player = players.iter_mut().find(|p| p.id == player_id);
    
    // Calculate player position for effects
    let player_pos = Vec2::new(-400.0 + (player_id.0 as f32 * 200.0), 0.0);
    
    match action {
        ActionSpace::DrawVine => {
            if let (Some(hand), Some(card)) = (hand.as_mut(), card_decks.draw_vine_card()) {
                hand.vine_cards.push(card);
                
                // Animate card draw
                let target_pos = Vec2::new(player_pos.x - 100.0, -200.0);
                animate_card_draw(commands, CardType::Vine, target_pos, animation_settings);
                
                crate::systems::audio::play_sfx(commands, audio_assets, audio_settings, AudioType::CardDraw);
                crate::systems::animations::spawn_animated_text(commands, player_id, "+Vine", Color::from(Srgba::new(0.2, 0.8, 0.2, 1.0)));
            }
        }
        ActionSpace::DrawWineOrder => {
            if let (Some(hand), Some(card)) = (hand.as_mut(), card_decks.draw_wine_order_card()) {
                hand.wine_order_cards.push(card);
                
                // Animate card draw
                let target_pos = Vec2::new(player_pos.x + 100.0, -200.0);
                animate_card_draw(commands, CardType::WineOrder, target_pos, animation_settings);
                
                crate::systems::audio::play_sfx(commands, audio_assets, audio_settings, AudioType::CardDraw);
                crate::systems::animations::spawn_animated_text(commands, player_id, "+Order", Color::from(Srgba::new(0.6, 0.2, 0.8, 1.0)));
            }
        }
        ActionSpace::PlantVine => {
            if let (Some(hand), Some(vineyard)) = (hand.as_mut(), vineyard.as_mut()) {
                if !hand.vine_cards.is_empty() {
                    let vine_card = &hand.vine_cards[0];
                    let player_structures: Vec<_> = structures.iter()
                        .filter(|s| s.owner == player_id)
                        .cloned()
                        .collect();
                    
                    // Find first suitable field with structure requirements
                    for i in 0..9 {
                        if vineyard.can_plant_vine_with_requirements(i, vine_card, &player_structures) {
                            let vine_card = hand.vine_cards.remove(0);
                            vineyard.fields[i].vine = Some(vine_card.vine_type);
                            vineyard.lira -= vine_card.cost;
                            
                            let field_pos = calculate_field_position(player_id, i);
                            spawn_construction_particles(commands, field_pos, animation_settings);
                            crate::systems::animations::spawn_animated_text(commands, player_id, "Planted!", Color::from(Srgba::new(0.4, 0.8, 0.4, 1.0)));
                            break;
                        }
                    }
                }
            }
        }
        ActionSpace::Harvest => {
            if let Some(vineyard) = vineyard.as_mut() {
                let structures = Vec::new();
                let gained = vineyard.harvest_grapes(&structures);
                if gained > 0 {
                    // Harvest particle effects
                    spawn_harvest_particles(commands, player_pos, gained, animation_settings);
                    
                    crate::systems::audio::play_sfx(commands, audio_assets, audio_settings, AudioType::Harvest);
                    crate::systems::animations::spawn_animated_text(commands, player_id, &format!("+{} Grapes", gained), Color::from(Srgba::new(0.8, 0.4, 0.8, 1.0)));
                }
            }
        }
        ActionSpace::MakeWine => {
            if let Some(vineyard) = vineyard.as_mut() {
                let red_available = vineyard.red_grapes;
                let white_available = vineyard.white_grapes;
                
                if red_available >= 2 && white_available >= 2 {
                    vineyard.red_grapes -= 1;
                    vineyard.white_grapes -= 1;
                    vineyard.red_wine += 2;
                    
                    // Wine pouring effect
                    spawn_wine_pouring_effect(commands, player_pos, animation_settings);
                    
                    crate::systems::audio::play_sfx(commands, audio_assets, audio_settings, AudioType::WineMake);
                    crate::systems::animations::spawn_animated_text(commands, player_id, "+Sparkling Wine", Color::from(Srgba::new(0.9, 0.7, 0.2, 1.0)));
                } else if red_available >= 1 && white_available >= 1 {
                    vineyard.red_grapes -= 1;
                    vineyard.white_grapes -= 1;
                    vineyard.white_wine += 1;
                    
                    spawn_wine_pouring_effect(commands, player_pos, animation_settings);
                    
                    crate::systems::audio::play_sfx(commands, audio_assets, audio_settings, AudioType::WineMake);
                    crate::systems::animations::spawn_animated_text(commands, player_id, "+Blush Wine", Color::from(Srgba::new(0.9, 0.5, 0.6, 1.0)));
                } else {
                    let red_to_use = if red_available > 0 { 1 } else { 0 };
                    let white_to_use = if white_available > 0 { 1 } else { 0 };
                    
                    if vineyard.make_wine(red_to_use, white_to_use) {
                        let total_wine = red_to_use + white_to_use;
                        if total_wine > 0 {
                            spawn_wine_pouring_effect(commands, player_pos, animation_settings);
                            
                            crate::systems::audio::play_sfx(commands, audio_assets, audio_settings, AudioType::WineMake);
                            crate::systems::animations::spawn_animated_text(commands, player_id, &format!("+{} Wine", total_wine), Color::from(Srgba::new(0.7, 0.2, 0.2, 1.0)));
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
                        vineyard.red_wine -= order.red_wine_needed;
                        vineyard.white_wine -= order.white_wine_needed;
                        
                        // Apply immediate rewards
                        player.gain_victory_points(order.victory_points);
                        player.gain_lira(order.immediate_payout());
                        
                        // Advance residual payment tracker
                        if let Some(mut tracker) = trackers.iter_mut().find(|t| t.owner == player_id) {
                            tracker.advance(order.residual_payment());
                        }
                        
                        // Existing particle effects...
                        spawn_victory_point_particles(commands, player_pos, order.victory_points, animation_settings);
                        if order.immediate_payout() > 0 {
                            spawn_lira_particles(commands, player_pos + Vec2::new(50.0, 0.0), order.immediate_payout(), animation_settings);
                        }
                        
                        crate::systems::audio::play_sfx(commands, audio_assets, audio_settings, AudioType::VictoryPoint);
                        crate::systems::animations::spawn_animated_text(commands, player_id, &format!("+{} VP", order.victory_points), Color::from(Srgba::new(1.0, 1.0, 0.0, 1.0)));
                    }
                }
            }
        }
        ActionSpace::GiveTour => {
            if let Some(player) = player.as_mut() {
                let bonus_lira = 2;
                player.gain_lira(bonus_lira);
                
                // Lira gain particles
                spawn_lira_particles(commands, player_pos, bonus_lira, animation_settings);
                
                crate::systems::audio::play_sfx(commands, audio_assets, audio_settings, AudioType::LiraGain);
                crate::systems::animations::spawn_animated_text(commands, player_id, &format!("+{} Lira", bonus_lira), Color::from(Srgba::new(1.0, 0.84, 0.0, 1.0)));
            }
        }
        ActionSpace::SellGrapes => {
            if let (Some(vineyard), Some(player)) = (vineyard.as_mut(), player.as_mut()) {
                let grapes_sold = vineyard.red_grapes + vineyard.white_grapes;
                if grapes_sold > 0 {
                    player.gain_lira(grapes_sold);
                    vineyard.red_grapes = 0;
                    vineyard.white_grapes = 0;
                    
                    // Lira gain particles
                    spawn_lira_particles(commands, player_pos, grapes_sold, animation_settings);
                    
                    crate::systems::audio::play_sfx(commands, audio_assets, audio_settings, AudioType::LiraGain);
                    crate::systems::animations::spawn_animated_text(commands, player_id, &format!("+{} Lira", grapes_sold), Color::from(Srgba::new(1.0, 0.84, 0.0, 1.0)));
                }
            }
        }
        ActionSpace::TrainWorker => {
            if let Some(player) = player.as_mut() {
                if player.lira >= 4 {
                    player.lira -= 4;
                    player.workers += 1;
                    
                    // Construction particles for training
                    spawn_construction_particles(commands, player_pos, animation_settings);
                    
                    crate::systems::animations::spawn_animated_text(commands, player_id, "+Worker", Color::from(Srgba::new(0.5, 0.8, 1.0, 1.0)));
                }
            }
        }
        ActionSpace::BuildStructure => {
            if let Some(vineyard) = vineyard.as_mut() {
                if vineyard.can_build_structure(StructureType::Trellis) {
                    if vineyard.build_structure(StructureType::Trellis) {
                        // Construction particles
                        spawn_construction_particles(commands, player_pos, animation_settings);
                        
                        crate::systems::animations::spawn_animated_text(commands, player_id, "+Structure", Color::from(Srgba::new(0.8, 0.8, 0.2, 1.0)));
                    }
                }
            }
        }
    }
}

pub fn fall_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut vineyards: Query<&mut Vineyard>,
    mut commands: Commands,
    text_query: Query<Entity, (With<Text>, Without<UIPanel>)>,
    animation_settings: Res<AnimationSettings>,
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
            let gained = vineyard.harvest_grapes(&structures);
            if gained > 0 {
                // Harvest particles for automatic harvest
                let player_pos = Vec2::new(-400.0 + (vineyard.owner.0 as f32 * 200.0), 0.0);
                spawn_harvest_particles(&mut commands, player_pos, gained, &animation_settings);
            }
        }
        
        // Trigger season transition
        trigger_season_transition(&mut commands, GameState::Fall, GameState::Winter, &animation_settings);
        
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

// Utility functions
fn calculate_field_position(player_id: PlayerId, field_index: usize) -> Vec2 {
    let field_x = -200.0 + ((field_index % 3) as f32 * 40.0);
    let field_y = 100.0 - ((field_index / 3) as f32 * 40.0);
    Vec2::new(field_x + (player_id.0 as f32 * 200.0), field_y)
}

fn spawn_construction_particles(
    commands: &mut Commands,
    position: Vec2,
    settings: &AnimationSettings,
) {
    let particle_count = (15.0 * settings.particle_density) as usize;
    let particles = create_construction_particles(position, particle_count);
    
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(position.extend(3.0)),
            ..default()
        },
        ParticleEffect {
            particles,
            effect_type: ParticleType::Construction,
            timer: Timer::from_seconds(1.2, TimerMode::Once),
        },
    ));
}

fn spawn_victory_point_particles(
    commands: &mut Commands,
    position: Vec2,
    vp_amount: u8,
    settings: &AnimationSettings,
) {
    let particle_count = (vp_amount as f32 * 3.0 * settings.particle_density) as usize;
    let particles = create_victory_point_particles(position, particle_count);
    
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(position.extend(3.0)),
            ..default()
        },
        ParticleEffect {
            particles,
            effect_type: ParticleType::VictoryPoints,
            timer: Timer::from_seconds(1.5, TimerMode::Once),
        },
    ));
}

fn create_construction_particles(center: Vec2, count: usize) -> Vec<Particle> {
    use rand::Rng;
    let mut rng = rand::rng();
    
    (0..count)
        .map(|_| {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let speed = rng.random_range(30.0..80.0);
            let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
            
            Particle {
                position: Vec2::new(
                    rng.random_range(-5.0..5.0),
                    rng.random_range(-5.0..5.0),
                ),
                velocity,
                life: rng.random_range(0.8..1.5),
                max_life: 1.5,
                size: rng.random_range(1.0..2.5),
                color: Color::from(Srgba::new(0.8, 0.8, 0.8, 1.0)), // Gray dust
            }
        })
        .collect()
}

fn create_victory_point_particles(center: Vec2, count: usize) -> Vec<Particle> {
    use rand::Rng;
    let mut rng = rand::rng();
    
    (0..count)
        .map(|_| {
            let angle = rng.random_range(-std::f32::consts::PI / 4.0..std::f32::consts::PI / 4.0);
            let speed = rng.random_range(40.0..100.0);
            let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
            
            Particle {
                position: Vec2::new(rng.random_range(-8.0..8.0), 0.0),
                velocity,
                life: rng.random_range(1.2..2.0),
                max_life: 2.0,
                size: rng.random_range(2.0..4.0),
                color: Color::from(Srgba::new(1.0, 1.0, 0.0, 1.0)), // Yellow stars
            }
        })
        .collect()
}

// Apply residual income at the start of each spring
pub fn apply_residual_income_system(
    mut players: Query<&mut Player>,
    residual_incomes: Query<&ResidualIncome>,
    current_state: Res<State<GameState>>,
) {
    // Only apply at the start of spring phase
    if current_state.is_changed() && matches!(current_state.get(), GameState::Spring) {
        for income in residual_incomes.iter() {
            if let Some(mut player) = players.iter_mut().find(|p| p.id == income.owner) {
                player.gain_lira(income.amount);
                info!("Player {:?} gained {} lira from residual income: {}", 
                      income.owner, income.amount, income.source);
            }
        }
    }
}

// Apply residual payments each spring
pub fn apply_residual_payments_system(
    mut players: Query<&mut Player>,
    trackers: Query<&ResidualPaymentTracker>,
    current_state: Res<State<GameState>>,
) {
    if current_state.is_changed() && matches!(current_state.get(), GameState::Spring) {
        for tracker in trackers.iter() {
            if let Some(mut player) = players.iter_mut().find(|p| p.id == tracker.owner) {
                let income = tracker.annual_income();
                if income > 0 {
                    player.gain_lira(income);
                    info!("Player {:?} gained {} lira from residual payments", tracker.owner, income);
                }
            }
        }
    }
}

// Apply Mama card special abilities when actions are performed
pub fn apply_mama_abilities_system(
    mama_cards: Query<&MamaCard>,
    mut players: Query<&mut Player>,
    mut vineyards: Query<&mut Vineyard>,
    workers: Query<&Worker, Changed<Worker>>,
    mut commands: Commands,
) {
    for worker in workers.iter() {
        // Only apply abilities when worker is newly placed
        if let Some(action) = worker.placed_at {
            // Find the player's mama card
            if let Some(mama) = mama_cards.iter().find(|m| m.id == worker.owner.0) {
                match (&mama.special_ability, action) {
                    (Some(MamaAbility::BonusHarvest), ActionSpace::Harvest) => {
                        if let Some(mut vineyard) = vineyards.iter_mut().find(|v| v.owner == worker.owner) {
                            vineyard.red_grapes += 1; // Bonus harvest grape
                            info!("Mama ability: {} got bonus harvest grape", mama.name);
                        }
                    },
                    (Some(MamaAbility::DiscountedStructures), ActionSpace::BuildStructure) => {
                        if let Some(mut vineyard) = vineyards.iter_mut().find(|v| v.owner == worker.owner) {
                            vineyard.lira += 1; // Refund 1 lira (structure discount)
                            info!("Mama ability: {} got structure discount", mama.name);
                        }
                    },
                    (Some(MamaAbility::FreeVinePlanting), ActionSpace::PlantVine) => {
                        if let Some(mut vineyard) = vineyards.iter_mut().find(|v| v.owner == worker.owner) {
                            vineyard.lira += 1; // Refund vine planting cost
                            info!("Mama ability: {} got free vine planting", mama.name);
                        }
                    },
                    _ => {} // No ability or doesn't match action
                }
            }
        }
    }
}

// Enhanced wine making that considers Papa card abilities
pub fn enhanced_make_wine_action(
    player_id: PlayerId,
    vineyards: &mut Query<&mut Vineyard>,
    papa_cards: &Query<&PapaCard>,
) -> u8 {
    if let Some(mut vineyard) = vineyards.iter_mut().find(|v| v.owner == player_id) {
        // Check if player has wine expertise papa ability
        let has_wine_expertise = papa_cards.iter()
            .any(|p| p.id == player_id.0 && 
                 matches!(p.special_ability, Some(PapaAbility::WineExpertise)));
        
        let red_available = vineyard.red_grapes;
        let white_available = vineyard.white_grapes;
        let mut wine_made = 0;
        
        // Enhanced wine making with multiple options
        if red_available >= 1 && white_available >= 1 {
            // Blush wine: 1 red + 1 white grape → wine (more efficient with expertise)
            let blush_efficiency = if has_wine_expertise { 2 } else { 1 };
            vineyard.red_grapes -= 1;
            vineyard.white_grapes -= 1;
            vineyard.white_wine += blush_efficiency; // Store blush as white wine
            wine_made += blush_efficiency;
            info!("Made blush wine (efficiency: {})", blush_efficiency);
        } else if red_available >= 2 && white_available >= 2 {
            // Sparkling wine: 2 red + 2 white → 3 wine (premium option)
            vineyard.red_grapes -= 2;
            vineyard.white_grapes -= 2;
            vineyard.red_wine += 3; // Sparkling gives bonus wine
            wine_made += 3;
            info!("Made sparkling wine");
        } else {
            // Regular wine making: 1 grape → 1 wine
            let red_to_use = red_available.min(2);
            let white_to_use = white_available.min(2);
            vineyard.red_grapes -= red_to_use;
            vineyard.white_grapes -= white_to_use;
            vineyard.red_wine += red_to_use;
            vineyard.white_wine += white_to_use;
            wine_made += red_to_use + white_to_use;
            info!("Made regular wine: {} red, {} white", red_to_use, white_to_use);
        }
        
        wine_made
    } else {
        0
    }
}


// 3. Add aging system
pub fn year_end_aging_system(
    mut vineyards: Query<&mut Vineyard>,
    current_state: Res<State<GameState>>,
) {
    if current_state.is_changed() && matches!(current_state.get(), GameState::Spring) {
        for mut vineyard in vineyards.iter_mut() {
            // Age grapes (max 9)
            vineyard.red_grapes = (vineyard.red_grapes + 1).min(9);
            vineyard.white_grapes = (vineyard.white_grapes + 1).min(9);
            
            // Age wines (max 9)
            vineyard.red_wine = (vineyard.red_wine + 1).min(9);
            vineyard.white_wine = (vineyard.white_wine + 1).min(9);
        }
    }
}

// 4. Add hand limit enforcement
pub fn enforce_hand_limit_system(
    mut hands: Query<&mut Hand>,
    current_state: Res<State<GameState>>,
) {
    if current_state.is_changed() && matches!(current_state.get(), GameState::Spring) {
        for mut hand in hands.iter_mut() {
            let total_cards = hand.vine_cards.len() + hand.wine_order_cards.len();
            if total_cards > 7 {
                let excess = total_cards - 7;
                // Simple implementation: remove vine cards first
                for _ in 0..excess {
                    if !hand.vine_cards.is_empty() {
                        hand.vine_cards.remove(0);
                    } else if !hand.wine_order_cards.is_empty() {
                        hand.wine_order_cards.remove(0);
                    }
                }
            }
        }
    }
}

// 5. Add temporary worker support
#[derive(Component)]
pub struct TemporaryWorker {
    pub owner: PlayerId,
    pub expires_end_of_year: bool,
}

pub fn assign_temporary_worker_system(
    mut commands: Commands,
    turn_order: Res<TurnOrder>,
    existing_temp: Query<Entity, With<TemporaryWorker>>,
) {
    // Clean up old temp workers
    for entity in existing_temp.iter() {
        commands.entity(entity).despawn();
    }
    
    // Find player who chose position 7 (last wake-up)
    if let Some((player_id, time)) = turn_order.wake_up_order.iter().find(|(_, t)| *t == 7) {
        let worker_pos = Vec2::new(-500.0 + (player_id.0 as f32 * 120.0), -230.0);
        commands.spawn((
            Worker::new(*player_id, false, worker_pos),
            TemporaryWorker { owner: *player_id, expires_end_of_year: true },
            Clickable { size: Vec2::new(20.0, 20.0) },
        ));
    }
}

// 7. Add Fall phase for visitor cards
pub fn fall_visitor_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut hands: Query<&mut Hand>,
    mut card_decks: ResMut<CardDecks>,
    turn_order: Res<TurnOrder>,
    mut commands: Commands,
    text_query: Query<Entity, (With<Text>, Without<UIPanel>)>,
) {
    if text_query.is_empty() {
        commands.spawn(Text2dBundle {
            text: Text::from_section(
                "FALL PHASE\nEach player draws a visitor card\n\nPress SPACE to continue",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 200.0, 1000.0)),
            ..default()
        });
        
        // Each player draws a visitor card (simplified: give summer visitor)
        for player_id in &turn_order.players {
            if let Some(mut hand) = hands.iter_mut().find(|h| h.owner == *player_id) {
                // Simplified: treat as vine card for now
                if let Some(card) = card_decks.draw_vine_card() {
                    hand.vine_cards.push(card);
                }
            }
        }
    }
    
    if keyboard.just_pressed(KeyCode::Space) {
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::Winter);
    }
}

pub fn fall_draw_visitors_system(
    mut hands: Query<&mut Hand>,
    turn_order: Res<TurnOrder>,
    structures: Query<&Structure>,
    mut visitor_deck: Option<ResMut<VisitorDeck>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    text_query: Query<Entity, (With<Text>, Without<UIPanel>)>,
    expansion_settings: Res<ExpansionSettings>,
) {
    // Only run if Tuscany expansion is enabled (where visitor cards exist)
    if !expansion_settings.tuscany_enabled {
        // Skip visitor cards, just advance to winter
        if keyboard.just_pressed(KeyCode::Space) {
            next_state.set(GameState::Winter);
        }
        return;
    }
    
    let Some(mut visitor_deck) = visitor_deck else {
        return; // No visitor deck available
    };
    
    if text_query.is_empty() {
        commands.spawn(Text2dBundle {
            text: Text::from_section(
                "FALL PHASE\nEach player draws a visitor card\nPress SPACE to continue to Winter",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 200.0, 1000.0)),
            ..default()
        });
        
        // Draw visitor cards for each player in wake-up order
        for player_id in &turn_order.players {
            if let Some(mut hand) = hands.iter_mut().find(|h| h.owner == *player_id) {
                // Draw 1 summer visitor card (player's choice simplified to summer)
                if let Some(visitor) = visitor_deck.draw_summer_visitor() {
                    hand.add_visitor_card(visitor);
                }
                
                // Check if player has cottage for bonus visitor
                let has_cottage = structures.iter()
                    .any(|s| s.owner == *player_id && matches!(s.structure_type, StructureType::Cottage));
                
                if has_cottage {
                    // Draw bonus winter visitor
                    if let Some(bonus_visitor) = visitor_deck.draw_winter_visitor() {
                        hand.add_visitor_card(bonus_visitor);
                    }
                }
            }
        }
    }
    
    if keyboard.just_pressed(KeyCode::Space) {
        for entity in text_query.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::Winter);
    }
}

// Update wine order fulfillment to advance residual tracker
pub fn fulfill_order_with_residual(
    player_id: PlayerId,
    order: &WineOrderCard,
    vineyard: &mut Vineyard,
    player: &mut Player,
    trackers: &mut Query<&mut ResidualPaymentTracker>,
) -> bool {
    if vineyard.can_fulfill_order(order) {
        vineyard.red_wine -= order.red_wine_needed;
        vineyard.white_wine -= order.white_wine_needed;
        
        // Apply immediate rewards
        player.gain_victory_points(order.victory_points);
        player.gain_lira(order.immediate_payout());
        
        // Advance residual payment tracker
        if let Some(mut tracker) = trackers.iter_mut().find(|t| t.owner == player_id) {
            tracker.advance(order.residual_payment());
        }
        
        true
    } else {
        false
    }
}

// Enhanced plant vine action with structure requirements
pub fn plant_vine_with_requirements_system(
    player_id: PlayerId,
    hands: &mut Query<&mut Hand>,
    vineyards: &mut Query<&mut Vineyard>,
    structures: &Query<&Structure>,
    commands: &mut Commands,
) -> bool {
    let mut hand = hands.iter_mut().find(|h| h.owner == player_id);
    let mut vineyard = vineyards.iter_mut().find(|v| v.owner == player_id);
    
    if let (Some(hand), Some(vineyard)) = (hand.as_mut(), vineyard.as_mut()) {
        if !hand.vine_cards.is_empty() {
            let vine_card = &hand.vine_cards[0];
            let player_structures: Vec<_> = structures.iter()
                .filter(|s| s.owner == player_id)
                .cloned()
                .collect();
            
            // Find first suitable field
            for i in 0..9 {
                if vineyard.can_plant_vine_with_requirements(i, vine_card, &player_structures) {
                    let vine_card = hand.vine_cards.remove(0);
                    vineyard.fields[i].vine = Some(vine_card.vine_type);
                    vineyard.lira -= vine_card.cost;
                    
                    info!("Planted {:?} in field {} with structure requirements met", vine_card.vine_type, i);
                    return true;
                }
            }
            
            // If no suitable field found, show requirements
            let requirements = vine_card.requirements();
            if requirements.needs_trellis || requirements.needs_irrigation {
                let mut missing = Vec::new();
                if requirements.needs_trellis {
                    missing.push("Trellis");
                }
                if requirements.needs_irrigation {
                    missing.push("Irrigation");
                }
                info!("Cannot plant vine - missing structures: {}", missing.join(", "));
            }
        }
    }
    false
}

// Field selling/buying system
pub fn field_transaction_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut vineyards: Query<&mut Vineyard>,
    turn_order: Res<TurnOrder>,
    mut commands: Commands,
) {
    if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
        if let Some(mut vineyard) = vineyards.iter_mut().find(|v| v.owner == *current_player_id) {
            
            // Sell field with S key (for testing)
            if keyboard.just_pressed(KeyCode::KeyS) {
                for i in 0..9 {
                    if vineyard.fields[i].can_sell() {
                        if let Some(value) = vineyard.sell_field(i) {
                            info!("Sold field {} for {} lira", i, value);
                            break;
                        }
                    }
                }
            }
            
            // Buy back field with B key (for testing)
            if keyboard.just_pressed(KeyCode::KeyB) {
                for i in 0..9 {
                    if vineyard.buy_back_field(i) {
                        info!("Bought back field {} ", i);
                        break;
                    }
                }
            }
        }
    }
}

// Enhanced worker placement for grande workers
pub fn enhanced_worker_placement_system(
    mut workers: Query<&mut Worker>,
    mut action_spaces: Query<&mut ActionSpaceSlot>,
    turn_order: Res<TurnOrder>,
    current_state: Res<State<GameState>>,
) {
    if let Some(current_player_id) = turn_order.players.get(turn_order.current_player) {
        // Find available grande worker
        if let Some(mut grande_worker) = workers.iter_mut()
            .find(|w| w.owner == *current_player_id && w.is_grande && w.placed_at.is_none()) {
            
            // Find fully occupied spaces where grande worker could be placed
            for mut space in action_spaces.iter_mut() {
                let is_correct_season = match current_state.get() {
                    GameState::Summer => space.is_summer,
                    GameState::Winter => !space.is_summer,
                    _ => false,
                };
                
                if is_correct_season && space.occupied_by.is_some() {
                    // This space is occupied, but grande worker can still use it
                    // Place grande worker "on the action art"
                    if space.place_grande_on_occupied(*current_player_id) {
                        space.bonus_worker_slot = Some(*current_player_id);
                        info!("Grande worker placed on occupied action {:?}", space.action);
                    }
                }
            }
        }
    }
}

// Update card generation to include structure requirements
pub fn create_enhanced_vine_deck() -> Vec<VineCard> {
    let mut deck = Vec::new();
    
    // Basic vines (no requirements)
    for i in 0..8 {
        deck.push(VineCard {
            id: i,
            vine_type: if i % 2 == 0 { VineType::Red(1) } else { VineType::White(1) },
            cost: 1,
            art_style: if i % 2 == 0 { CardArt::BasicRed } else { CardArt::BasicWhite },
            special_ability: None,
        });
    }
    
    // Trellis-requiring vines
    for i in 10..16 {
        deck.push(VineCard {
            id: i,
            vine_type: if i % 2 == 0 { VineType::Red(3) } else { VineType::White(3) },
            cost: 2,
            art_style: if i % 2 == 0 { CardArt::PremiumRed } else { CardArt::PremiumWhite },
            special_ability: None,
        });
    }
    
    // Irrigation-requiring vines
    for i in 20..24 {
        deck.push(VineCard {
            id: i,
            vine_type: if i % 2 == 0 { VineType::Red(3) } else { VineType::White(3) },
            cost: 3,
            art_style: if i % 2 == 0 { CardArt::SpecialtyRed } else { CardArt::SpecialtyWhite },
            special_ability: Some(VineAbility::HighYield),
        });
    }
    
    // Premium vines (need both structures)
    for i in 30..34 {
        deck.push(VineCard {
            id: i,
            vine_type: if i % 2 == 0 { VineType::Red(4) } else { VineType::White(4) },
            cost: 4,
            art_style: if i % 2 == 0 { CardArt::SpecialtyRed } else { CardArt::SpecialtyWhite },
            special_ability: Some(VineAbility::DiseaseResistant),
        });
    }
    
    deck
}

// Enhanced wine orders with residual payments
pub fn create_wine_orders_with_residual() -> Vec<WineOrderCard> {
    vec![
        // Basic orders (no residual)
        WineOrderCard::new(100, 1, 0, 1, 1),
        WineOrderCard::new(101, 0, 1, 1, 1),
        WineOrderCard::new(102, 2, 0, 2, 2),
        
        // Orders with residual payments
        WineOrderCard::new_with_residual(200, 2, 1, 3, 2, 1), // 2 immediate + 1 residual
        WineOrderCard::new_with_residual(201, 1, 2, 3, 1, 2), // 1 immediate + 2 residual  
        WineOrderCard::new_with_residual(202, 3, 2, 5, 3, 2), // 3 immediate + 2 residual
        WineOrderCard::new_with_residual(203, 4, 3, 7, 2, 3), // 2 immediate + 3 residual (high residual)
        
        // Premium orders
        WineOrderCard::new_with_residual(300, 5, 2, 8, 4, 1), 
        WineOrderCard::new_with_residual(301, 3, 5, 9, 3, 2),
        WineOrderCard::new_with_residual(302, 6, 4, 12, 5, 3),
    ]
}

pub fn validate_actions_with_requirements(
    player_id: PlayerId,
    action: ActionSpace,
    players: &Query<&Player>,
    hands: &Query<&Hand>,
    vineyards: &Query<&Vineyard>,
    structures: &Query<&Structure>,
) -> ValidationResult {
    match action {
        ActionSpace::PlantVine => {
            let hand = hands.iter().find(|h| h.owner == player_id).unwrap();
            let vineyard = vineyards.iter().find(|v| v.owner == player_id).unwrap();
            
            if hand.vine_cards.is_empty() {
                return ValidationResult::Invalid("No vine cards to plant".to_string());
            }
            
            let vine_card = &hand.vine_cards[0];
            let player_structures: Vec<_> = structures.iter()
                .filter(|s| s.owner == player_id)
                .cloned()
                .collect();
            
            let can_plant = (0..9).any(|i| vineyard.can_plant_vine_with_requirements(i, vine_card, &player_structures));
            
            if !can_plant {
                let requirements = vine_card.requirements();
                if requirements.needs_trellis || requirements.needs_irrigation {
                    let mut missing = Vec::new();
                    if requirements.needs_trellis {
                        missing.push("Trellis");
                    }
                    if requirements.needs_irrigation {
                        missing.push("Irrigation");
                    }
                    return ValidationResult::Invalid(format!("Missing required structures: {}", missing.join(", ")));
                } else {
                    return ValidationResult::Invalid("No available fields or insufficient lira".to_string());
                }
            }
        }
        _ => return ValidationResult::Valid,
    }
    
    ValidationResult::Valid
}

pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }
}