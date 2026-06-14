//! Les biomes du jardin : 6 biomes (une run en traverse 5), GDD §7.

use bevy::prelude::*;

use crate::boss::BossKind;
use crate::enemies::EnemyKind;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Biome {
    /// Herbe rase, point de départ. Boss : Mémé Mygale.
    Jardin,
    /// Cailloux et poussière. Boss : Mille-Pattes.
    Gravier,
    /// Terre détrempée et gluante. Boss : Grompaud.
    Boue,
    /// Sol craquelé, chaud et sec. Boss : Roger le Scorpion.
    TerreSeche,
    /// Terre grasse, légumes et bave. Boss : Méga-Limace.
    Potager,
    /// Pierre et joints urbains. Boss : Araignée géante.
    Dalles,
}

pub const ALL_BIOMES: [Biome; 6] = [
    Biome::Jardin,
    Biome::Gravier,
    Biome::Boue,
    Biome::TerreSeche,
    Biome::Potager,
    Biome::Dalles,
];

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Jardin => "Le Jardin",
            Biome::Gravier => "Le Gravier",
            Biome::Boue => "La Boue",
            Biome::TerreSeche => "La Terre Sèche",
            Biome::Potager => "Le Potager",
            Biome::Dalles => "Les Dalles",
        }
    }

    #[allow(dead_code)] // flavor : réservé à un futur codex / carte de run
    pub fn desc(self) -> &'static str {
        match self {
            Biome::Jardin => "Herbe rase, espace ouvert. Insectes tout-venant.",
            Biome::Gravier => "Cailloux et poussière. Ça gratte et ça fonce.",
            Biome::Boue => "Terre détrempée, mousse et bave. Lent, collant.",
            Biome::TerreSeche => "Sol craquelé et brûlant. Ça pique et ça tanke.",
            Biome::Potager => "Terre grasse, légumes et bave. Lent, mou, collant.",
            Biome::Dalles => "Pierre et joints. Sec, dur, et ça grouille vite.",
        }
    }

    /// Couleur de fond (hors arène).
    pub fn clear_color(self) -> Color {
        match self {
            Biome::Jardin => Color::srgb(0.16, 0.22, 0.10),
            Biome::Gravier => Color::srgb(0.14, 0.14, 0.16),
            Biome::Boue => Color::srgb(0.10, 0.10, 0.07),
            Biome::TerreSeche => Color::srgb(0.24, 0.19, 0.08),
            Biome::Potager => Color::srgb(0.12, 0.16, 0.07),
            Biome::Dalles => Color::srgb(0.12, 0.13, 0.15),
        }
    }

    /// Couleur du sol de l'arène.
    pub fn ground_color(self) -> Color {
        match self {
            Biome::Jardin => Color::srgb(0.30, 0.42, 0.18),
            Biome::Gravier => Color::srgb(0.45, 0.45, 0.48),
            Biome::Boue => Color::srgb(0.28, 0.22, 0.13),
            Biome::TerreSeche => Color::srgb(0.55, 0.45, 0.20),
            Biome::Potager => Color::srgb(0.34, 0.30, 0.16),
            Biome::Dalles => Color::srgb(0.40, 0.42, 0.46),
        }
    }

    /// Couleur du décor (touffes, cailloux…).
    pub fn accent_color(self) -> Color {
        match self {
            Biome::Jardin => Color::srgb(0.38, 0.55, 0.22),
            Biome::Gravier => Color::srgb(0.62, 0.62, 0.66),
            Biome::Boue => Color::srgb(0.40, 0.32, 0.18),
            Biome::TerreSeche => Color::srgb(0.70, 0.60, 0.30),
            Biome::Potager => Color::srgb(0.55, 0.50, 0.25),
            Biome::Dalles => Color::srgb(0.55, 0.58, 0.62),
        }
    }

    /// Bestiaire par « tiers » : nuée / moyens / gros, tiré dans les 6 mobs de
    /// la refonte v0.3 (3 archétypes × 2 types, GDD §8). Chaque biome a sa
    /// teinte (ce qu'il met en avant), mais tous mélangent les archétypes.
    pub fn tier(self, t: u8) -> &'static [EnemyKind] {
        match (self, t) {
            // Jardin : tout-venant, mêlée et ruée.
            (Biome::Jardin, 0) => &[EnemyKind::Fourmi, EnemyKind::Criquet],
            (Biome::Jardin, 1) => &[EnemyKind::Araignee, EnemyKind::Guepe],
            (Biome::Jardin, _) => &[EnemyKind::Escargot],
            // Gravier : vif, ruée et tir lourd.
            (Biome::Gravier, 0) => &[EnemyKind::Fourmi, EnemyKind::Criquet],
            (Biome::Gravier, 1) => &[EnemyKind::Cigale, EnemyKind::Araignee],
            (Biome::Gravier, _) => &[EnemyKind::Escargot],
            // Boue : gluant, ruée et gros mous.
            (Biome::Boue, 0) => &[EnemyKind::Criquet, EnemyKind::Fourmi],
            (Biome::Boue, 1) => &[EnemyKind::Araignee, EnemyKind::Cigale],
            (Biome::Boue, _) => &[EnemyKind::Escargot],
            // Terre Sèche : ça pique (tir) et ça tanke.
            (Biome::TerreSeche, 0) => &[EnemyKind::Fourmi, EnemyKind::Criquet],
            (Biome::TerreSeche, 1) => &[EnemyKind::Guepe, EnemyKind::Cigale],
            (Biome::TerreSeche, _) => &[EnemyKind::Escargot, EnemyKind::Araignee],
            // Potager : lent, mou, collant.
            (Biome::Potager, 0) => &[EnemyKind::Fourmi, EnemyKind::Criquet],
            (Biome::Potager, 1) => &[EnemyKind::Cigale, EnemyKind::Guepe],
            (Biome::Potager, _) => &[EnemyKind::Escargot, EnemyKind::Araignee],
            // Dalles : sec et grouillant, mêlée et tir.
            (Biome::Dalles, 0) => &[EnemyKind::Criquet, EnemyKind::Fourmi],
            (Biome::Dalles, 1) => &[EnemyKind::Araignee, EnemyKind::Guepe],
            (Biome::Dalles, _) => &[EnemyKind::Escargot],
        }
    }

    pub fn boss(self) -> BossKind {
        match self {
            Biome::Jardin => BossKind::Araignee,
            Biome::Gravier => BossKind::MillePattes,
            Biome::Boue => BossKind::Gromp,
            Biome::TerreSeche => BossKind::Scorpion,
            Biome::Potager => BossKind::MegaLimace,
            Biome::Dalles => BossKind::AraigneeGeante,
        }
    }

    /// Identifiant stable pour la sauvegarde.
    pub fn id(self) -> &'static str {
        match self {
            Biome::Jardin => "jardin",
            Biome::Gravier => "gravier",
            Biome::Boue => "boue",
            Biome::TerreSeche => "terre_seche",
            Biome::Potager => "potager",
            Biome::Dalles => "dalles",
        }
    }

}
