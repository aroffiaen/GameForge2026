# LittleFastGardener

Un **roguelike d'action top-down** sur le thème de la **vitesse**, où l'on incarne un
jardinier mystérieusement rétréci dans son propre jardin, qui doit survivre salle après
salle pour regagner sa terrasse.

---

## 🌱 Le pitch

Vous êtes un **jardinier rétréci**. Coincé au ras du sol dans votre jardin devenu une jungle
géante, vous vous frayez un chemin à travers une succession de salles,
infestées d'ennemis de toutes tailles et de tous types, pour remonter jusqu'à votre
**terrasse** — votre supposée zone de sécurité.

## ⚡ Le hook : la vitesse, c'est la **progression**

Le pilier central qui distingue le jeu :

> **Plus tu nettoies les salles vite, plus tu deviens fort.**

La vitesse n'est pas un bonus de dégâts : c'est le **moteur de progression**. Chaque salle (hors
boss/terrasse) est **chronométrée**. Avant d'y entrer, tu passes une des **3 portes**, chacune
surmontée d'une **stat** : franchir une porte, c'est **miser** cette stat. Boucle la salle **sous
le temps cible** et tu **gagnes** la stat (proportionnel à ton avance) ; dépasse le chrono et tu la
**perds**. On gagne plus vite qu'on ne perd — mais l'échec pique.

Résultat : on est en mouvement permanent — *kite*, *dash*, on traverse, on optimise chaque seconde.

> 💨 Pour les nostalgiques, l'ancienne mécanique « plus vite = plus de dégâts » revient en **augment
> optionnel** (« Élan ») avec des ratios doux.

## 🎮 Gameplay

- **Vue :** top-down (caméra au-dessus du personnage).
- **Visée :** manuelle — le joueur vise et déclenche ses attaques (skill-based, façon *Enter the Gungeon*).
- **Mobilité au cœur du jeu :** déplacements nerveux (à inertie), dash avec i-frames, perso rapide et fragile (*glass cannon*). On encaisse des **attaques télégraphiées** (lisibles, esquivables) — **plus de dégâts de collision**.
- **7 stats en %** (PV, Régén, Dégâts, Résistance, Vitesse, Cadence, Dash CD) qui montent/descendent via le **Stats-Up chronométré** (cf. le hook). Base 100 %, plancher 25 %, **pas de plafond** (le snowball peut partir loin).
- **Structure « biomes » :** un run = **5 biomes** (tirés parmi 6), chacun = **5 salles + 1 boss**. Mobs variés (6 mobs, 3 archétypes), élites, boss à patterns.

### 🛠️ Armes (matériel de jardin)

Le personnage peut porter **jusqu'à 2 armes simultanément**. Le sprite du perso est
**séparé** de celui des armes (pour pouvoir les combiner librement et les animer
indépendamment).

Les armes (10 au roster) sont inspirées de l'outillage de jardinage. **Aucun knockback**, et trois
d'entre elles sont en **Maintien** (hold-to-shoot, sans coût ni cooldown continu) :

| Arme | Déclenchement | Comportement |
|------|---------------|--------------|
| 🧴 Pesticide | Maintien | traînée de poison au sol (DoT) |
| ⛏️ Pelle | Frappe | AoE en anneau autour du perso |
| 🍴 Râteau | Frappe | attire les ennemis devant (cône) |
| 🔫 Karcher | Maintien | spray 60° à pression |
| 🪚 Tronçonneuse | Maintien | ligne continue (ralentit + bloque l'arme 2) |
| ⛏️ Pioche | Frappe | impact de zone à mi-portée |
| 🌾 Faux | Frappe | grand cône ~50° longue portée |
| 🪓 Hache | Frappe | lancée jusqu'au mur, gros dégâts, long CD |
| 🌙 Serpe | Frappe | balayage AoE ~300° rapide |
| 🌿 Pic de vigne | Frappe | estoc qui s'allonge (lance) |

> Valeurs (dégâts/portée/CD) en cours de tuning au playtest.

### 🧬 Augments & synergies

Au fil d'un run, le joueur récupère des **augments** qui enrichissent et transforment le
gameplay. Objectif : **maximiser la rejouabilité** en poussant à tester de multiples
**synergies** entre armes et augments. Chaque run encourage une *build* différente.

### 🔁 Boucle roguelike & progression permanente

- **Run :** on progresse de salle en salle jusqu'à mourir ou atteindre la terrasse.
- **Méta-progression :** accomplir certains objectifs débloque **de façon permanente** de
  nouvelles armes et des upgrades, disponibles dans les runs suivants.
- **Mode final « terrasse » :** mode **chronométré** — des ennemis arrivent en continu et
  montent en puissance ; objectif : survivre le plus longtemps possible.

---

## 🦀 Stack technique

- **Langage :** [Rust](https://www.rust-lang.org/) (edition 2024)
- **Moteur :** [Bevy](https://bevyengine.org/) `0.18`
- **Cible :** **Windows** (GPU, Vulkan). Le dev se fait sous **WSL2 / Ubuntu**, mais le jeu se
  **compile et se lance nativement sous Windows** — WSLg n'a pas de GPU exploitable pour Bevy.

## 🚀 Jouer

### Le plus simple : télécharger le build

➡️ **[Release `v0.3` (Windows)](https://github.com/aroffiaen/GameForge2026/releases/tag/v0.3)** —
décompresse le zip et lance `GameForge2026.exe` (garde le dossier `assets` à côté de l'exe).

> Au 1ᵉʳ lancement, Windows SmartScreen peut afficher « Éditeur inconnu » → *Informations
> complémentaires* → *Exécuter quand même* (normal pour un exe non signé).

## 🛠️ Build depuis les sources

1. **Rust** via [rustup](https://rustup.rs/) (edition 2024), avec la toolchain **Windows**
   (`x86_64-pc-windows-msvc`).
2. Depuis un terminal **WSL**, dans le dépôt :

   ```bash
   ./play-windows.sh
   ```

   Le script synchronise les sources vers `C:\GameForge2026`, compile en **release** avec la
   toolchain Windows, copie les `assets/` à côté de l'exe, puis ouvre la fenêtre du jeu.

> ⚠️ **Ne lance pas `cargo run` sous WSL** : Bevy plante (`WaylandError(NoCompositor)`), pas de GPU.
> `cargo build` sous WSL marche pour **juste vérifier que ça compile**.

---

## 📍 État du projet

✅ **v0.3 jouable** — refonte de design complète et build Windows publié en
[Release](https://github.com/aroffiaen/GameForge2026/releases/tag/v0.3).

📖 La conception détaillée (mécaniques, armes, augments, structure des runs, méta-progression,
lore…) vit dans le **[Game Design Document](docs/GDD.md)** — la **checklist §18** suit l'implémentation.

### Fait (v0.3)

- [x] Fenêtre Bevy + boucle de jeu, perso top-down à inertie + dash (i-frames)
- [x] **7 stats** branchées sur leurs effets + **Stats-Up chronométré** (3 portes, chrono, mise/gain/perte)
- [x] **10 armes** (2 slots, sprite séparé, Maintien/Frappe, pas de knockback)
- [x] **6 mobs** (3 archétypes, attaques télégraphiées, plus de collision) + **élites** + **6 boss** à patterns
- [x] Structure **5 biomes × (5 salles + 1 boss)**, augment 3→1 après boss
- [x] Augments (mécaniques/keystones), méta-progression (Pattes, déblocages, cabanon)
- [x] **Mode Terrasse** (survie chronométrée, record sauvegardé)
- [x] **Sprites mobs/boss**, **audio complet** (SFX + 3 bus de volume Mobs/Boss/Effets)
- [x] **Touches configurables**, **menus cliquables**, **bac à sable de dev** (F3)

### Reste à faire

- [ ] Tuning de playtest (valeurs d'armes, seuils de chrono, équilibrage des stats)
- [ ] Boss propre pour les **Dalles** (actuellement « Araignée géante » placeholder)
- [ ] **Nom définitif** du jeu (actuellement *GameForge2026*, nom de projet)
