//! Méta-progression : Pattes, accomplissements, déblocages chez le bousier
//! (GDD §11). Sauvegarde RON locale (`save.ron`).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::weapons::WeaponKind;

pub const SAVE_PATH: &str = "save.ron";

// ---------------------------------------------------------------------------
// Sauvegarde
// ---------------------------------------------------------------------------

#[derive(Resource, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MetaSave {
    /// La monnaie unique : des pattes d'insectes (GDD §11.2).
    pub pattes: u64,
    pub total_kills: u64,
    /// Outils récupérés, donc jouables.
    pub unlocked: Vec<WeaponKind>,
    /// Accomplissement validé : l'outil attend d'être racheté au bousier.
    pub claimable: Vec<WeaponKind>,
    pub achievements: Vec<String>,
    /// Biomes dont le boss a été battu au moins une fois.
    pub bosses_beaten: Vec<String>,
    // Upgrades permanents (rangs achetés).
    pub up_hp: u8,
    pub up_speed: u8,
    pub up_dash: u8,
    pub up_pattes: u8,
    pub terrasse_unlocked: bool,
    pub best_terrasse: f32,
    pub runs: u32,
    pub deaths: u32,
    /// Throttle : max 2 outils récupérés par run (GDD §11.3).
    pub tools_bought_this_cycle: u8,
    /// Touches configurables, persistées par libellé (Settings).
    pub keybinds: KeybindsSave,
}

/// Représentation sérialisable des keybinds (libellés AZERTY).
#[derive(Resource, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct KeybindsSave {
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub dash: String,
    pub interact: String,
}

impl Default for KeybindsSave {
    fn default() -> Self {
        Self::from_binds(&Keybinds::default())
    }
}

impl KeybindsSave {
    pub fn from_binds(kb: &Keybinds) -> Self {
        Self {
            up: key_label(kb.up).to_string(),
            down: key_label(kb.down).to_string(),
            left: key_label(kb.left).to_string(),
            right: key_label(kb.right).to_string(),
            dash: key_label(kb.dash).to_string(),
            interact: key_label(kb.interact).to_string(),
        }
    }

    pub fn to_binds(&self) -> Keybinds {
        let d = Keybinds::default();
        Keybinds {
            up: key_from_label(&self.up).unwrap_or(d.up),
            down: key_from_label(&self.down).unwrap_or(d.down),
            left: key_from_label(&self.left).unwrap_or(d.left),
            right: key_from_label(&self.right).unwrap_or(d.right),
            dash: key_from_label(&self.dash).unwrap_or(d.dash),
            interact: key_from_label(&self.interact).unwrap_or(d.interact),
        }
    }
}

impl Default for MetaSave {
    fn default() -> Self {
        Self {
            pattes: 0,
            total_kills: 0,
            unlocked: vec![WeaponKind::Pelle],
            // Les 6 armes de la refonte (Lot 2) sont rachetables d'emblée
            // chez le bousier (pas de gate d'accomplissement dédiée).
            claimable: vec![
                WeaponKind::Pioche,
                WeaponKind::Serpe,
                WeaponKind::Faux,
                WeaponKind::PicDeVigne,
                WeaponKind::Hache,
                WeaponKind::Tronconneuse,
            ],
            achievements: Vec::new(),
            bosses_beaten: Vec::new(),
            up_hp: 0,
            up_speed: 0,
            up_dash: 0,
            up_pattes: 0,
            terrasse_unlocked: false,
            best_terrasse: 0.0,
            runs: 0,
            deaths: 0,
            tools_bought_this_cycle: 0,
            keybinds: KeybindsSave::default(),
        }
    }
}

impl MetaSave {
    pub fn has_ach(&self, id: &str) -> bool {
        self.achievements.iter().any(|a| a == id)
    }
    pub fn add_ach(&mut self, id: &str) {
        if !self.has_ach(id) {
            self.achievements.push(id.to_string());
        }
    }
    pub fn is_unlocked(&self, w: WeaponKind) -> bool {
        self.unlocked.contains(&w)
    }
    pub fn is_claimable(&self, w: WeaponKind) -> bool {
        self.claimable.contains(&w)
    }
    pub fn make_claimable(&mut self, w: WeaponKind) -> bool {
        if self.is_unlocked(w) || self.is_claimable(w) {
            return false;
        }
        self.claimable.push(w);
        true
    }
}

pub fn load_meta() -> MetaSave {
    match std::fs::read_to_string(SAVE_PATH) {
        Ok(content) => ron::from_str(&content).unwrap_or_else(|err| {
            warn!("save.ron illisible ({err}), on repart de zéro");
            MetaSave::default()
        }),
        Err(_) => MetaSave::default(),
    }
}

pub fn save_meta(meta: &MetaSave) {
    match ron::ser::to_string_pretty(meta, ron::ser::PrettyConfig::default()) {
        Ok(s) => {
            if let Err(err) = std::fs::write(SAVE_PATH, s) {
                warn!("Impossible d'écrire {SAVE_PATH}: {err}");
            }
        }
        Err(err) => warn!("Sérialisation de la sauvegarde impossible: {err}"),
    }
}

/// Prix de rachat des outils chez le bousier (GDD §11.3).
pub fn tool_price(w: WeaponKind) -> u64 {
    match w {
        WeaponKind::Pelle => 0, // outil de départ
        WeaponKind::Pesticide => 100,
        WeaponKind::Rateau => 140,
        WeaponKind::Karcher => 220,
        WeaponKind::Pioche => 160,
        WeaponKind::Serpe => 180,
        WeaponKind::Faux => 200,
        WeaponKind::PicDeVigne => 240,
        WeaponKind::Hache => 280,
        WeaponKind::Tronconneuse => 300,
    }
}

/// Condition d'accomplissement qui rend l'outil récupérable.
pub fn tool_condition(w: WeaponKind) -> &'static str {
    match w {
        WeaponKind::Pelle => "Ton outil de départ.",
        WeaponKind::Pesticide => "Dézingue 100 insectes (en cumulé).",
        WeaponKind::Rateau => "Bats ton premier boss.",
        WeaponKind::Karcher => "Atteins la Terrasse.",
        WeaponKind::Pioche
        | WeaponKind::Serpe
        | WeaponKind::Faux
        | WeaponKind::PicDeVigne
        | WeaponKind::Hache
        | WeaponKind::Tronconneuse => "Disponible au rachat chez le bousier.",
    }
}

// ---------------------------------------------------------------------------
// Plugin : moteur d'accomplissements
// ---------------------------------------------------------------------------

pub struct MetaPlugin;

impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load_meta())
            .init_resource::<Keybinds>()
            .add_systems(Startup, sync_keybinds_from_save)
            .add_systems(Update, achievements_engine)
            .add_systems(OnEnter(AppState::Terrasse), terrasse_reached);
    }
}

/// Au démarrage, construit la ressource `Keybinds` depuis la sauvegarde.
fn sync_keybinds_from_save(meta: Res<MetaSave>, mut kb: ResMut<Keybinds>) {
    *kb = meta.keybinds.to_binds();
}

/// Écoute les morts d'ennemis et déverrouille la « récupérabilité » des
/// outils (modèle accomplissements + achats, GDD §11.3).
fn achievements_engine(
    mut died: MessageReader<EnemyDied>,
    mut meta: ResMut<MetaSave>,
    run: Res<crate::rooms::RunState>,
    mut toasts: MessageWriter<ToastMsg>,
) {
    let mut changed = false;
    for msg in died.read() {
        meta.total_kills += 1;

        // Pesticide : 100 insectes au compteur.
        if meta.total_kills >= 100 && !meta.has_ach("kills_100") {
            meta.add_ach("kills_100");
            if meta.make_claimable(WeaponKind::Pesticide) {
                toasts.write(ToastMsg(
                    "100 insectes ! Le bousier veut bien te revendre le PESTICIDE.".into(),
                ));
            }
            changed = true;
        }

        if msg.was_boss {
            // Râteau : premier boss.
            if !meta.has_ach("first_boss") {
                meta.add_ach("first_boss");
                if meta.make_claimable(WeaponKind::Rateau) {
                    toasts.write(ToastMsg(
                        "Premier boss ! Le RÂTEAU est récupérable au cabanon.".into(),
                    ));
                }
            }
            // Cumul des boss battus par biome (réservé aux déblocages du Lot 2).
            let biome_id = run.biome.id().to_string();
            if !meta.bosses_beaten.contains(&biome_id) {
                meta.bosses_beaten.push(biome_id);
            }
            changed = true;
        }
    }
    if changed {
        save_meta(&meta);
    }
}

/// Atteindre la terrasse via une run : accomplissement du Karcher.
fn terrasse_reached(
    run: Res<crate::rooms::RunState>,
    mut meta: ResMut<MetaSave>,
    mut toasts: MessageWriter<ToastMsg>,
) {
    if !run.came_from_run {
        return;
    }
    if !meta.has_ach("terrasse") {
        meta.add_ach("terrasse");
        if meta.make_claimable(WeaponKind::Karcher) {
            toasts.write(ToastMsg(
                "LA TERRASSE ! Le KARCHER est récupérable au cabanon.".into(),
            ));
        }
        save_meta(&meta);
    }
}
