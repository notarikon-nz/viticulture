use bevy::prelude::*;
use crate::components::*;

#[derive(Resource, Default)]
pub struct EndGameScoring {
    pub final_scores: Vec<(PlayerId, u8, String)>, // (player_id, final_vp, breakdown)
}

pub fn calculate_final_scores(
    mut scoring: ResMut<EndGameScoring>,
    players: Query<&Player>,
    vineyards: Query<&Vineyard>,
    structures: Query<&Structure>,
) {
    scoring.final_scores.clear();
    
    for player in players.iter() {
        let vineyard = vineyards.iter().find(|v| v.owner == player.id).unwrap();
        let player_structures: Vec<_> = structures.iter()
            .filter(|s| s.owner == player.id)
            .collect();
        
        let mut final_vp = player.victory_points;
        let mut breakdown = format!("Base VP: {}", player.victory_points);
        
        // Windmill bonus: +1 VP for every 7 lira
        if player_structures.iter().any(|s| matches!(s.structure_type, StructureType::Windmill)) {
            let windmill_bonus = vineyard.lira / 7;
            if windmill_bonus > 0 {
                final_vp += windmill_bonus;
                breakdown.push_str(&format!(" | Windmill: +{}", windmill_bonus));
            }
        }
        
        // Bonus VP for leftover resources (encourages efficiency)
        let leftover_bonus = calculate_leftover_bonus(vineyard);
        if leftover_bonus > 0 {
            final_vp += leftover_bonus;
            breakdown.push_str(&format!(" | Resources: +{}", leftover_bonus));
        }
        
        // Structure completion bonus
        let structure_bonus = calculate_structure_bonus(&player_structures);
        if structure_bonus > 0 {
            final_vp += structure_bonus;
            breakdown.push_str(&format!(" | Structures: +{}", structure_bonus));
        }
        
        scoring.final_scores.push((player.id, final_vp, breakdown));
    }
    
    // Sort by final VP (descending)
    scoring.final_scores.sort_by(|a, b| b.1.cmp(&a.1));
}

fn calculate_leftover_bonus(vineyard: &Vineyard) -> u8 {
    let total_wine = vineyard.red_wine + vineyard.white_wine;
    let total_grapes = vineyard.red_grapes + vineyard.white_grapes;
    
    // Small bonus for leftover resources (max 2 VP)
    let wine_bonus = (total_wine / 3).min(1); // 1 VP per 3 wine
    let grape_bonus = (total_grapes / 5).min(1); // 1 VP per 5 grapes
    
    wine_bonus + grape_bonus
}

fn calculate_structure_bonus(structures: &[&Structure]) -> u8 {
    let structure_count = structures.len() as u8;
    
    // Bonus for having multiple structures
    match structure_count {
        0..=1 => 0,
        2..=3 => 1, // Small bonus for diversification
        4..=5 => 2, // Medium bonus
        _ => 3,     // Large bonus for full development
    }
}

pub fn enhanced_tie_breaker(
    players: &Query<&Player>,
    vineyards: &Query<&Vineyard>,
    structures: &Query<&Structure>,
    scoring: &EndGameScoring,
) -> PlayerId {
    let top_score = scoring.final_scores[0].1;
    let tied_players: Vec<_> = scoring.final_scores.iter()
        .filter(|(_, vp, _)| *vp == top_score)
        .collect();
    
    if tied_players.len() == 1 {
        return tied_players[0].0;
    }
    
    // Enhanced tie-breaker: VP → Lira → Wine → Grapes → Structures
    let mut tie_break_data: Vec<_> = tied_players.iter()
        .map(|(player_id, vp, _)| {
            let vineyard = vineyards.iter().find(|v| v.owner == *player_id).unwrap();
            let structure_count = structures.iter()
                .filter(|s| s.owner == *player_id)
                .count();
            
            (*player_id, *vp, vineyard.lira, vineyard.red_wine + vineyard.white_wine, 
             vineyard.red_grapes + vineyard.white_grapes, structure_count)
        })
        .collect();
    
    tie_break_data.sort_by(|a, b| {
        b.1.cmp(&a.1) // VP
            .then(b.2.cmp(&a.2)) // Lira
            .then(b.3.cmp(&a.3)) // Wine
            .then(b.4.cmp(&a.4)) // Grapes
            .then(b.5.cmp(&a.5)) // Structures
    });
    
    tie_break_data[0].0
}

pub fn display_final_scores(
    mut commands: Commands,
    scoring: Res<EndGameScoring>,
    players: Query<&Player>,
) {
    if scoring.final_scores.is_empty() {
        return;
    }
    
    let winner_id = scoring.final_scores[0].0;
    let winner = players.iter().find(|p| p.id == winner_id).unwrap();
    let winner_vp = scoring.final_scores[0].1;
    
    // Display winner
    commands.spawn(TextBundle::from_section(
        format!("🏆 {} WINS! 🏆\nFinal Score: {} Victory Points", winner.name, winner_vp),
        TextStyle {
            font_size: 32.0,
            color: Color::from(Srgba::new(1.0, 0.84, 0.0, 1.0)),
            ..default()
        },
    ).with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(150.0),
        left: Val::Px(50.0),
        ..default()
    }));
    
    // Display all scores
    let mut score_text = String::new();
    for (i, (player_id, vp, breakdown)) in scoring.final_scores.iter().enumerate() {
        let player = players.iter().find(|p| p.id == *player_id).unwrap();
        score_text.push_str(&format!("{}. {}: {} VP\n   {}\n", 
                                    i + 1, player.name, vp, breakdown));
    }
    
    commands.spawn(TextBundle::from_section(
        score_text,
        TextStyle {
            font_size: 16.0,
            color: Color::WHITE,
            ..default()
        },
    ).with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(250.0),
        left: Val::Px(50.0),
        ..default()
    }));
}