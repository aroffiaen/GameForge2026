//! Augments & synergies : peu nombreux, tranchants (GDD §5).

use bevy::prelude::*;
use rand::prelude::*;

use crate::common::*;

// Tri v0.3 (§18.H) : les augments de **% brut** (vitesse, dégâts, PV) ont été
// retirés — ils font désormais doublon avec le système de stats (partie A).
// Ne restent que des **mécaniques / keystones / mods d'arme** (GDD §9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Augment {
    // Vitesse / élan
    Cafeine,
    Adrenaline,
    /// L'ancienne mécanique signature, devenue optionnelle (GDD §4.x) :
    /// la vitesse redonne des dégâts, mais en doux (×0.8 → ×1.5).
    Elan,
    // Dash / burst
    DashOffensif,
    DoubleDetente,
    EsquiveFeline,
    SortieExplosive,
    // Modificateurs d'armes
    PesticideConcentre,
    RateauAimante,
    BuseHautePression,
    PelleElargie,
    // Momentum / on-the-move
    Momentum,
    Photosynthese,
    // Keystones
    Epidemie,
    TraineeToxique,
    // Défensif (mesuré)
    Rosee,
}

pub const POOL: &[Augment] = &[
    Augment::Cafeine,
    Augment::Adrenaline,
    Augment::Elan,
    Augment::DashOffensif,
    Augment::DoubleDetente,
    Augment::EsquiveFeline,
    Augment::SortieExplosive,
    Augment::PesticideConcentre,
    Augment::RateauAimante,
    Augment::BuseHautePression,
    Augment::PelleElargie,
    Augment::Momentum,
    Augment::Photosynthese,
    Augment::Epidemie,
    Augment::TraineeToxique,
    Augment::Rosee,
];

impl Augment {
    pub fn name(self) -> &'static str {
        match self {
            Augment::Cafeine => "Café du bousier",
            Augment::Adrenaline => "Adrénaline",
            Augment::Elan => "Élan",
            Augment::DashOffensif => "Dash offensif",
            Augment::DoubleDetente => "Double détente",
            Augment::EsquiveFeline => "Esquive féline",
            Augment::SortieExplosive => "Sortie explosive",
            Augment::PesticideConcentre => "Pesticide concentré",
            Augment::RateauAimante => "Râteau aimanté",
            Augment::BuseHautePression => "Buse haute pression",
            Augment::PelleElargie => "Pelle élargie",
            Augment::Momentum => "Momentum",
            Augment::Photosynthese => "Photosynthèse",
            Augment::Epidemie => "Épidémie",
            Augment::TraineeToxique => "Traînée toxique",
            Augment::Rosee => "Rosée du matin",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Augment::Cafeine => "+30 % d'accélération. Nerveux.",
            Augment::Adrenaline => "Sous 30 % de PV : +25 % de vitesse.",
            Augment::Elan => "L'élan paie : dégâts ×0,8 à l'arrêt → ×1,5 à pleine vitesse.",
            Augment::DashOffensif => "Attaque pendant le dash + burst ×1,5 en sortie.",
            Augment::DoubleDetente => "Une 2e charge de dash.",
            Augment::EsquiveFeline => "I-frames du dash allongées.",
            Augment::SortieExplosive => "Explosion en sortie de dash.",
            Augment::PesticideConcentre => "Poison +60 % de dégâts.",
            Augment::RateauAimante => "Râteau : rayon +40 % et ralentit 1,5 s.",
            Augment::BuseHautePression => "Karcher : +25 % de dégâts.",
            Augment::PelleElargie => "Armes de frappe : zone +35 %.",
            Augment::Momentum => "+2 %/s de dégâts en mouvement (max +40 %).",
            Augment::Photosynthese => "Régénère 2 PV/s au-dessus de 70 % de vitesse.",
            Augment::Epidemie => "KEYSTONE : le poison se propage à la mort.",
            Augment::TraineeToxique => "KEYSTONE : le dash laisse du pesticide.",
            Augment::Rosee => "Soigne 12 PV après chaque boss.",
        }
    }

    /// Depuis le tri v0.3, plus aucun augment ne se cumule : les seuls
    /// cumulables étaient les % bruts, partis dans le système de stats.
    /// Conservé pour `roll_offer` (qui exclut les augments déjà pris).
    pub fn stackable(self) -> bool {
        let _ = self;
        false
    }
}

// ---------------------------------------------------------------------------
// Ressources
// ---------------------------------------------------------------------------

/// Le build de la run en cours (GDD : ~5-7 augments/run).
#[derive(Resource, Default)]
pub struct Augments(pub Vec<Augment>);

impl Augments {
    pub fn has(&self, a: Augment) -> bool {
        self.0.contains(&a)
    }
}

/// L'offre courante (3 → 1, GDD §5.1).
#[derive(Resource, Default)]
pub struct AugmentOffer(pub Vec<Augment>);

/// Ce qui se passe une fois l'augment choisi.
#[derive(Resource, Clone, Copy, PartialEq, Default)]
pub enum AfterAugment {
    /// Salle intermédiaire : on ouvre la porte.
    #[default]
    Door,
    /// Après un boss : choix de biome (ou terrasse après le 5e).
    PostBoss,
}

/// Tire `n` augments distincts du pool, en excluant les uniques déjà pris.
pub fn roll_offer(augments: &Augments, n: usize) -> Vec<Augment> {
    let mut rng = rand::rng();
    let mut available: Vec<Augment> = POOL
        .iter()
        .copied()
        .filter(|a| a.stackable() || !augments.has(*a))
        .collect();
    available.shuffle(&mut rng);
    available.truncate(n);
    available
}

// ---------------------------------------------------------------------------
// Plugin & UI de choix
// ---------------------------------------------------------------------------

pub struct AugmentsPlugin;

impl Plugin for AugmentsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Augments>()
            .init_resource::<AugmentOffer>()
            .init_resource::<AfterAugment>()
            .add_systems(OnEnter(RunPhase::Augment), open_augment_ui)
            .add_systems(
                Update,
                pick_augment.run_if(in_state(RunPhase::Augment)),
            );
    }
}

fn open_augment_ui(
    mut commands: Commands,
    augments: Res<Augments>,
    mut offer: ResMut<AugmentOffer>,
) {
    offer.0 = roll_offer(&augments, 3);

    commands
        .spawn((
            DespawnOnExit(RunPhase::Augment),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
            GlobalZIndex(20),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("— LE JARDIN TE PROPOSE —"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.9, 0.6)),
            ));
            if offer.0.is_empty() {
                parent.spawn((
                    Text::new("Plus rien à offrir ! (Espace pour continuer)"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            }
            for (i, augment) in offer.0.iter().enumerate() {
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            width: Val::Px(440.0),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.2, 0.1, 0.95)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(format!("[{}]  {}", i + 1, augment.name())),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 1.0, 0.5)),
                        ));
                        card.spawn((
                            Text::new(augment.desc()),
                            TextFont {
                                font_size: 15.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.85, 0.85, 0.8)),
                        ));
                    });
            }
            parent.spawn((
                Text::new("Choisis avec 1, 2 ou 3"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

fn pick_augment(
    keys: Res<ButtonInput<KeyCode>>,
    offer: Res<AugmentOffer>,
    mut augments: ResMut<Augments>,
    after: Res<AfterAugment>,
    mut toasts: MessageWriter<ToastMsg>,
    mut next_phase: ResMut<NextState<RunPhase>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut run: ResMut<crate::rooms::RunState>,
    mut meta: ResMut<crate::meta::MetaSave>,
) {
    let picked = if keys.just_pressed(KeyCode::Digit1) {
        offer.0.first().copied()
    } else if keys.just_pressed(KeyCode::Digit2) {
        offer.0.get(1).copied()
    } else if keys.just_pressed(KeyCode::Digit3) {
        offer.0.get(2).copied()
    } else if offer.0.is_empty() && keys.just_pressed(KeyCode::Space) {
        // Pool épuisé : on passe.
        advance(&after, &mut run, &mut meta, &mut next_phase, &mut next_state, &mut toasts);
        return;
    } else {
        None
    };
    let Some(augment) = picked else { return };
    augments.0.push(augment);
    toasts.write(ToastMsg(format!("Augment : {}", augment.name())));
    advance(&after, &mut run, &mut meta, &mut next_phase, &mut next_state, &mut toasts);
}

fn advance(
    after: &AfterAugment,
    run: &mut crate::rooms::RunState,
    meta: &mut crate::meta::MetaSave,
    next_phase: &mut NextState<RunPhase>,
    next_state: &mut NextState<AppState>,
    toasts: &mut MessageWriter<ToastMsg>,
) {
    match after {
        AfterAugment::Door => next_phase.set(RunPhase::DoorOpen),
        AfterAugment::PostBoss => {
            if run.biome_index >= 4 {
                // 5 biomes nettoyés : LA TERRASSE (GDD §10).
                run.came_from_run = true;
                if !meta.terrasse_unlocked {
                    meta.terrasse_unlocked = true;
                    toasts.write(ToastMsg(
                        "La Terrasse est désormais accessible depuis le cabanon !".into(),
                    ));
                }
                crate::meta::save_meta(meta);
                next_phase.set(RunPhase::None);
                next_state.set(AppState::Terrasse);
            } else {
                // Après le boss : l'augment est pris, on ouvre 3 portes-stat qui
                // mènent au biome suivant (aléatoire non vu, GDD §6).
                run.awaiting_biome = true;
                next_phase.set(RunPhase::DoorOpen);
            }
        }
    }
}
