//! Les 7 stats du joueur — cœur de la refonte v0.3 (GDD §3.3).
//!
//! Chaque stat est un **pourcentage** : base 100 %, plancher 25 %, **sans cap**
//! (le snowball doit pouvoir partir loin). Un « point de stat » = 1 %.
//! Le système de Stats-Up chronométré (portes + chrono, GDD §3.1) fait monter
//! ou descendre ces valeurs ; ici on ne définit que la ressource et le câblage
//! des effets (« brancher chaque stat sur son effet »).

use bevy::prelude::*;

/// Plancher commun à toutes les stats (GDD §3.3).
pub const STAT_FLOOR: f32 = 25.0;
/// Valeur de base de toute stat au début d'une run.
pub const STAT_BASE: f32 = 100.0;

/// Les 7 stats. L'ordre fixe la position dans le tableau de `Stats`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Stat {
    Pv,
    Regen,
    Dmg,
    Resistance,
    MoveSpeed,
    AttackSpeed,
    DashCd,
}

impl Stat {
    pub const ALL: [Stat; 7] = [
        Stat::Pv,
        Stat::Regen,
        Stat::Dmg,
        Stat::Resistance,
        Stat::MoveSpeed,
        Stat::AttackSpeed,
        Stat::DashCd,
    ];

    /// Libellé court affiché au HUD et au-dessus des portes.
    pub fn label(self) -> &'static str {
        match self {
            Stat::Pv => "PV",
            Stat::Regen => "Régén",
            Stat::Dmg => "Dégâts",
            Stat::Resistance => "Résist.",
            Stat::MoveSpeed => "Vitesse",
            Stat::AttackSpeed => "Cadence",
            Stat::DashCd => "Dash CD",
        }
    }
}

/// Les valeurs courantes des 7 stats (en %), indexées par `Stat as usize`.
#[derive(Resource, Clone)]
pub struct Stats {
    values: [f32; 7],
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            values: [STAT_BASE; 7],
        }
    }
}

impl Stats {
    /// Valeur (%) d'une stat.
    pub fn get(&self, s: Stat) -> f32 {
        self.values[s as usize]
    }

    /// Ajoute `delta` points (%) à une stat, en respectant le plancher.
    /// `delta` peut être négatif (mise perdue).
    pub fn add(&mut self, s: Stat, delta: f32) {
        let v = &mut self.values[s as usize];
        *v = (*v + delta).max(STAT_FLOOR);
    }

    /// Remet toutes les stats à la base (début de run / retour cabanon).
    pub fn reset(&mut self) {
        self.values = [STAT_BASE; 7];
    }

    // --- Multiplicateurs prêts à l'emploi (neutres à 100 %) ---

    /// PV max : ×PV%/100.
    pub fn pv_mult(&self) -> f32 {
        self.get(Stat::Pv) / 100.0
    }
    /// Régén passive en HP/s : 1.0 × Régén%/100.
    pub fn regen_hps(&self) -> f32 {
        1.0 * self.get(Stat::Regen) / 100.0
    }
    /// Dégâts d'arme : ×DMG%/100.
    pub fn dmg_mult(&self) -> f32 {
        self.get(Stat::Dmg) / 100.0
    }
    /// Réduction des dégâts subis : dégâts ×100/Rési% (200 %→½, 25 %→×4).
    pub fn incoming_mult(&self) -> f32 {
        100.0 / self.get(Stat::Resistance)
    }
    /// Vitesse de déplacement : ×MS%/100.
    pub fn move_mult(&self) -> f32 {
        self.get(Stat::MoveSpeed) / 100.0
    }
    /// Cadence d'attaque : intervalle ×100/AS% (200 %→2× plus rapide).
    pub fn attack_cd_mult(&self) -> f32 {
        100.0 / self.get(Stat::AttackSpeed)
    }
    /// Cooldown de dash : ×100/DashCD% (200 %→0.62 s).
    pub fn dash_cd_mult(&self) -> f32 {
        100.0 / self.get(Stat::DashCd)
    }
}

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stats>();
    }
}
