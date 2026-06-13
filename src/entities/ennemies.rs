use bevy::prelude::*;
// use rand::prelude::*; // pas utilisé dans ce snippet

use crate::common::*;
use crate::player::Player;

// Définitions

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum EnemyKind {
    Puceron,
    Fourmi,
    Araignee,
    Moustique,
    Guepe,
    Scarabee,
    Escargot,
    Limace,
}

#[allow(dead_code)]
pub struct EnemyDef {
    pub name: &'static str,
    pub hp: f32,
    pub speed: f32,
    pub dmg: f32,
    pub radius: f32,
    pub pattes: u32,
    pub color: Color,
    pub ai: AiKind,
    /// Multiplicateur des dégâts de poison subis (les « mous », GDD §8.1).
    pub poison_vuln: f32,
}

pub fn def(kind: EnemyKind) -> EnemyDef {
    match kind {
        EnemyKind::Puceron => EnemyDef {
            name: "Puceron",
            hp: 8.0,
            speed: 115.0,
            dmg: 4.0,
            radius: 16.0, // Taille 32x32
            pattes: 1,
            color: Color::srgb(0.55, 0.85, 0.45),
            ai: AiKind::Chase,
            poison_vuln: 1.0,
        },
        EnemyKind::Fourmi => EnemyDef {
            name: "Fourmi",
            hp: 14.0,
            speed: 150.0,
            dmg: 5.0,
            radius: 18.0, // Un peu plus grosse
            pattes: 1,
            color: Color::srgb(0.45, 0.25, 0.15),
            ai: AiKind::Chase,
            poison_vuln: 1.0,
        },
        EnemyKind::Araignee => EnemyDef {
            name: "Araignée",
            hp: 26.0,
            speed: 170.0,
            dmg: 8.0,
            radius: 22.0,
            pattes: 3,
            color: Color::srgb(0.25, 0.22, 0.3),
            ai: AiKind::Lunge,
            poison_vuln: 1.0,
        },
        EnemyKind::Moustique => EnemyDef {
            name: "Moustique",
            hp: 10.0,
            speed: 175.0,
            dmg: 4.0,
            radius: 16.0,
            pattes: 2,
            color: Color::srgb(0.6, 0.6, 0.7),
            ai: AiKind::Ranged { min: 150.0, max: 240.0, shoot_cd: 2.0 },
            poison_vuln: 1.0,
        },
        EnemyKind::Guepe => EnemyDef {
            name: "Guêpe",
            hp: 22.0,
            speed: 145.0,
            dmg: 7.0,
            radius: 22.0,
            pattes: 3,
            color: Color::srgb(0.95, 0.8, 0.1),
            ai: AiKind::Ranged { min: 170.0, max: 260.0, shoot_cd: 1.6 },
            poison_vuln: 1.0,
        },
        EnemyKind::Scarabee => EnemyDef {
            name: "Scarabée",
            hp: 48.0,
            speed: 85.0,
            dmg: 9.0,
            radius: 28.0, // Plus imposant
            pattes: 3,
            color: Color::srgb(0.3, 0.4, 0.5),
            ai: AiKind::Chase,
            poison_vuln: 1.0,
        },
        EnemyKind::Escargot => EnemyDef {
            name: "Escargot",
            hp: 85.0,
            speed: 38.0,
            dmg: 12.0,
            radius: 34.0,
            pattes: 4,
            color: Color::srgb(0.65, 0.55, 0.4),
            ai: AiKind::Chase,
            poison_vuln: 1.0,
        },
        EnemyKind::Limace => EnemyDef {
            name: "Limace",
            hp: 38.0,
            speed: 48.0,
            dmg: 6.0,
            radius: 26.0,
            pattes: 2,
            color: Color::srgb(0.8, 0.6, 0.2),
            ai: AiKind::Chase,
            poison_vuln: 1.6,
        },
    }
}

pub fn enemy_shoot(
    time: Res<Time>,
    _commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(&EnemyKind, &Transform, &mut ShootCd), With<Enemy>>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    
    for (kind, tf, mut cd) in &mut enemies {
        cd.0.tick(time.delta());
        let AiKind::Ranged { max, shoot_cd, .. } = def(*kind).ai else {
            continue;
        };
        let pos = tf.translation.truncate();
        if cd.0.is_finished() && pos.distance(player_pos) <= max + 40.0 {
            cd.0 = Timer::from_seconds(shoot_cd, TimerMode::Once);
            let dir = (player_pos - pos).normalize_or(Vec2::X);
            
            // 
            info!("Enemy {:?} shoots at {:?}", kind, dir);
        }
    }
}

/// Teinte verte sur les ennemis empoisonnés (lisibilité).
pub fn poison_tint(
    mut enemies: Query<
        (&mut Sprite, &BaseColor, Has<Poisoned>),
        (With<Enemy>, Without<HitFlash>),
    >,
) {
    for (mut sprite, base, poisoned) in &mut enemies {
        let target = if poisoned {
            base.0.mix(&Color::srgb(0.3, 1.0, 0.2), 0.45)
        } else {
            base.0
        };
        sprite.color = target;
    }
}