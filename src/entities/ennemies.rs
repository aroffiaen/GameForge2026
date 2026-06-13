use bevy::prelude::*;
use bevy::prelude::MessageWriter as BevyMessageWriter;
use bevy::prelude::MessageReader as BevyMessageReader;
// use rand::prelude::*; // pas utilisé dans ce snippet

use crate::common::*;
use crate::player::{Player, Iframes};

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
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(&EnemyKind, &Transform, &ContactDmg, &mut ShootCd), With<Enemy>>,
) {
    let Ok(player_tf) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    
    for (kind, tf, contact, mut cd) in &mut enemies {
        cd.0.tick(time.delta());
        let AiKind::Ranged { max, shoot_cd, .. } = def(*kind).ai else {
            continue;
        };
        let pos = tf.translation.truncate();
        if cd.0.is_finished() && pos.distance(player_pos) <= max + 40.0 {
            cd.0 = Timer::from_seconds(shoot_cd, TimerMode::Once);
            let dir = (player_pos - pos).normalize_or(Vec2::X);
            
            spawn_enemy_projectile(&mut commands, pos, dir * 250.0, contact.0, def(*kind).color);
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

//  projectile ennemie
pub fn enemy_projectiles(
    mut commands: Commands,
    arena: Res<Arena>,
    mut dmg: BevyMessageWriter<DamageMsg>,
    projectiles: Query<(Entity, &Transform, &EnemyProjectile)>,
    mut player: Query<(Entity, &Transform, &Radius, &mut Iframes), With<Player>>,
) {
    let Ok((player_e, player_tf, player_r, mut iframes)) = player.single_mut() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (e, tf, proj) in &projectiles {
        let pos = tf.translation.truncate();
        if pos.x.abs() > arena.half.x + 30.0 || pos.y.abs() > arena.half.y + 30.0 {
            commands.entity(e).despawn();
            continue;
        }
        if pos.distance(player_pos) <= player_r.0 + 5.0 {
            if iframes.0.is_finished() {
                dmg.write(DamageMsg {
                    target: player_e,
                    amount: proj.dmg,
                    kind: DamageKind::Hit,
                });
                iframes.0 = Timer::from_seconds(0.5, TimerMode::Once);
            }
            commands.entity(e).despawn();
        }
    }
}

// flaque de poison
pub fn hazard_puddles(
    time: Res<Time>,
    mut commands: Commands,
    mut dmg: BevyMessageWriter<DamageMsg>,
    mut puddles: Query<(Entity, &Transform, &mut HazardPuddle, &mut Sprite)>,
    player: Query<(Entity, &Transform, &Radius), With<Player>>,
) {
    let Ok((player_e, player_tf, player_r)) = player.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (e, tf, mut puddle, mut sprite) in &mut puddles {
        puddle.life.tick(time.delta());
        puddle.tick.tick(time.delta());
        let left = 1.0 - puddle.life.fraction();
        if left < 0.3 {
            sprite.color = sprite.color.with_alpha(0.45 * (left / 0.3));
        }
        if puddle.life.is_finished() {
            commands.entity(e).despawn();
            continue;
        }
        if puddle.tick.just_finished()
            && tf.translation.truncate().distance(player_pos) <= puddle.radius + player_r.0
        {
            dmg.write(DamageMsg {
                target: player_e,
                amount: puddle.dps * puddle.tick.duration().as_secs_f32(),
                kind: DamageKind::Poison,
            });
        }
    }
}

// dégâts par contact (mob touche joueur)
pub fn contact_damage(
    mut dmg: BevyMessageWriter<DamageMsg>,
    query_player: Query<(Entity, &Transform, &Radius, &Iframes), With<Player>>,
    query_mobs: Query<(&Transform, &ContactDmg), With<Enemy>>,
) {
    let Ok((player_e, player_tf, player_r, iframes)) = query_player.single() else {
        return;
    };

    if !iframes.0.is_finished() {
        return;
    }

    let player_pos = player_tf.translation.truncate();

    for (mob_tf, contact) in query_mobs.iter() {
        let mob_pos = mob_tf.translation.truncate();
        if mob_pos.distance(player_pos) < player_r.0 + 10.0 {
            dmg.write(DamageMsg {
                target: player_e,
                amount: contact.0,
                kind: DamageKind::Hit,
            });
            break; 
        }
    }
}

// système qui applique réellement les dégâts
pub fn handle_damage(
    mut messages: BevyMessageReader<DamageMsg>,
    mut query_health: Query<(&mut Health, Option<&mut Iframes>)>,
) {
    for msg in messages.read() {
        if let Ok((mut health, iframes)) = query_health.get_mut(msg.target) {
            health.hp -= msg.amount as i32;
            
            if let Some(mut iframes) = iframes {
                iframes.0 = Timer::from_seconds(0.5, TimerMode::Once);
            }
            info!("Entity {:?} took {} damage! HP left: {}", msg.target, msg.amount, health.hp);
        }
    }
}

// gestion de la mort (despawn à 0 HP)
pub fn death_system(
    mut commands: Commands,
    query: Query<(Entity, &Health)>,
) {
    for (entity, health) in query.iter() {
        if health.hp <= 0 {
            info!("Entity {:?} is dead!", entity);
            commands.entity(entity).despawn();
        }
    }
}

// système de débogage pour afficher les HP et positions
pub fn debug_logger_system(
    time: Res<Time>,
    mut timer: Local<Timer>,
    query_player: Query<(&Transform, &Health), With<Player>>,
    query_mobs: Query<(Entity, &Transform, &Health, Option<&EnemyKind>), With<Enemy>>,
) {
    // initialiser le timer s'il vient d'être créé
    if timer.duration() == std::time::Duration::ZERO {
        *timer = Timer::from_seconds(1.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());

    if timer.just_finished() {
        if let Ok((tf, health)) = query_player.single() {
            let pos = tf.translation.truncate();
            info!("PLAYER | HP: {} | Pos: X={:.1}, Y={:.1}", health.hp, pos.x, pos.y);
        }

        for (entity, tf, health, kind) in query_mobs.iter() {
            let pos = tf.translation.truncate();
            if let Some(k) = kind {
                info!("MOB {:?} ({:?}) | HP: {} | Pos: X={:.1}, Y={:.1}", entity, k, health.hp, pos.x, pos.y);
            } else {
                info!("MOB {:?} | HP: {} | Pos: X={:.1}, Y={:.1}", entity, health.hp, pos.x, pos.y);
            }
        }
    }
}

// afficher projectile ennemie

pub fn spawn_enemy_projectile(
    commands: &mut Commands,
    pos: Vec2,
    vel: Vec2,
    dmg: f32,
    color: Color,
) {
    commands.spawn((
        EnemyProjectile { dmg },
        Sprite {
            color: color.mix(&Color::WHITE, 0.3),
            custom_size: Some(Vec2::splat(7.0)),
            ..default()
        },
        Transform::from_translation(pos.extend(9.0)),
        Velocity(vel),
        Lifetime::secs(3.0),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;

    #[test]
    fn test_contact_damage_and_hp_loss() {
        let mut app = App::new();
        
        // preparer setup : minimal pour eviter les overheads
        app.add_plugins(MinimalPlugins);
        app.add_message::<DamageMsg>();
        app.add_systems(Update, (contact_damage, handle_damage).chain());
        
        // spawn joueur : 100 pv
        let player_id = app.world_mut().spawn((
            Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Radius(16.0),
            Health { hp: 100 },
            Iframes(Timer::from_seconds(0.1, TimerMode::Once)),
        )).id();

        // valider timer : s'assurer qu'il est fini pour le premier impact
        {
            let mut player_iframes = app.world_mut().get_mut::<Iframes>(player_id).unwrap();
            player_iframes.0.tick(std::time::Duration::from_millis(200));
        }

        // spawn ennemi : sur la meme position que le joueur
        app.world_mut().spawn((
            Enemy,
            ContactDmg(5.0),
            Transform::from_xyz(5.0, 0.0, 0.0),
        ));

        // executer impact : premiere collision
        app.update();

        // verifier hp : doit avoir perdu 5 pv
        let health = app.world().get::<Health>(player_id).unwrap();
        assert_eq!(health.hp, 95);

        // verifier iframes : doit etre invulnerable
        let iframes = app.world().get::<Iframes>(player_id).unwrap();
        assert!(!iframes.0.is_finished());

        // executer deuxieme impact : aucun degat attendu
        app.update();
        
        let health_after = app.world().get::<Health>(player_id).unwrap();
        assert_eq!(health_after.hp, 95, "Le joueur ne doit pas perdre de vie pendant les Iframes");
    }
}
