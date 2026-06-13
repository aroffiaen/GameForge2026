//! Les biomes du jardin : pool de départ à 3 (GDD §7).

use bevy::prelude::*;

use crate::boss::BossKind;
use crate::enemies::EnemyKind;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Biome {
    Plaine,
    Savane,
    Jungle,
    /// Le Potager : terre grasse et gluante, royaume de la Méga-Limace (GDD §7).
    Potager,
}

pub const ALL_BIOMES: [Biome; 4] =
    [Biome::Plaine, Biome::Savane, Biome::Jungle, Biome::Potager];

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Plaine => "La Plaine",
            Biome::Savane => "La Savane",
            Biome::Jungle => "La Jungle",
            Biome::Potager => "Le Potager",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Biome::Plaine => "Herbe rase, espace ouvert. Insectes tout-venant.",
            Biome::Savane => "Herbes sèches et carapaces. Ça pique et ça tanke.",
            Biome::Jungle => "Mousse, humidité, trucs gluants. Glissant.",
            Biome::Potager => "Terre grasse, légumes et bave. Lent, mou, collant.",
        }
    }

    /// Couleur de fond (hors arène).
    pub fn clear_color(self) -> Color {
        match self {
            Biome::Plaine => Color::srgb(0.16, 0.22, 0.10),
            Biome::Savane => Color::srgb(0.24, 0.19, 0.08),
            Biome::Jungle => Color::srgb(0.07, 0.14, 0.10),
            Biome::Potager => Color::srgb(0.12, 0.16, 0.07),
        }
    }

    /// Couleur du sol de l'arène.
    pub fn ground_color(self) -> Color {
        match self {
            Biome::Plaine => Color::srgb(0.30, 0.42, 0.18),
            Biome::Savane => Color::srgb(0.55, 0.45, 0.20),
            Biome::Jungle => Color::srgb(0.13, 0.28, 0.18),
            Biome::Potager => Color::srgb(0.34, 0.30, 0.16),
        }
    }

    /// Couleur du décor (touffes, cailloux…).
    pub fn accent_color(self) -> Color {
        match self {
            Biome::Plaine => Color::srgb(0.38, 0.55, 0.22),
            Biome::Savane => Color::srgb(0.70, 0.60, 0.30),
            Biome::Jungle => Color::srgb(0.20, 0.40, 0.25),
            Biome::Potager => Color::srgb(0.55, 0.50, 0.25),
        }
    }

    /// Bestiaire par « tiers » du gauntlet : nuée / moyens / gros (GDD §6.3).
    pub fn tier(self, t: u8) -> &'static [EnemyKind] {
        match (self, t) {
            (Biome::Plaine, 0) => &[EnemyKind::Puceron, EnemyKind::Fourmi],
            (Biome::Plaine, 1) => &[EnemyKind::Araignee, EnemyKind::Moustique],
            (Biome::Plaine, _) => &[EnemyKind::Scarabee],
            (Biome::Savane, 0) => &[EnemyKind::Fourmi, EnemyKind::Puceron],
            (Biome::Savane, 1) => &[EnemyKind::Guepe, EnemyKind::Araignee],
            (Biome::Savane, _) => &[EnemyKind::Scarabee, EnemyKind::Escargot],
            (Biome::Jungle, 0) => &[EnemyKind::Moustique, EnemyKind::Puceron],
            (Biome::Jungle, 1) => &[EnemyKind::Limace, EnemyKind::Guepe],
            (Biome::Jungle, _) => &[EnemyKind::Escargot, EnemyKind::Scarabee],
            (Biome::Potager, 0) => &[EnemyKind::Limace, EnemyKind::Puceron],
            (Biome::Potager, 1) => &[EnemyKind::Escargot, EnemyKind::Moustique],
            (Biome::Potager, _) => &[EnemyKind::Escargot, EnemyKind::Scarabee],
        }
    }

    pub fn boss(self) -> BossKind {
        match self {
            Biome::Plaine => BossKind::Araignee,
            Biome::Savane => BossKind::Scorpion,
            Biome::Jungle => BossKind::Gromp,
            Biome::Potager => BossKind::MegaLimace,
        }
    }

    /// Identifiant stable pour la sauvegarde.
    pub fn id(self) -> &'static str {
        match self {
            Biome::Plaine => "plaine",
            Biome::Savane => "savane",
            Biome::Jungle => "jungle",
            Biome::Potager => "potager",
        }
    }

    /// Les 2 options proposées après un boss. Pool à 4 : rotation déterministe
    /// (les 2 biomes suivants dans le cycle), de sorte que chaque biome — dont
    /// le Potager et sa Méga-Limace — reste atteignable. La refonte v0.3
    /// (5 biomes parmi 6, sans répétition) remplacera cette règle.
    pub fn next_choices(self) -> [Biome; 2] {
        match self {
            Biome::Plaine => [Biome::Savane, Biome::Jungle],
            Biome::Savane => [Biome::Jungle, Biome::Potager],
            Biome::Jungle => [Biome::Potager, Biome::Plaine],
            Biome::Potager => [Biome::Plaine, Biome::Savane],
        }
    }
}
