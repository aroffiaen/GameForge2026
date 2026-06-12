# GameForge2026

> *Nom de travail — à renommer plus tard.*

Un **roguelike d'action top-down** sur le thème de la **vitesse**, où l'on incarne un
jardinier mystérieusement rétréci dans son propre jardin, qui doit survivre salle après
salle pour regagner sa terrasse.

---

## 🌱 Le pitch

Vous êtes un **jardinier rétréci**. Coincé au ras du sol dans votre jardin devenu une jungle
géante, vous vous frayez un chemin à travers une succession de salles (la *« tower »*),
infestées d'ennemis de toutes tailles et de tous types, pour remonter jusqu'à votre
**terrasse** — votre supposée zone de sécurité.

> ⚠️ **Plot twist :** la terrasse n'est pas un refuge. C'est l'épreuve finale — un **mode
> chronométré** où des ennemis affluent en continu, de plus en plus forts. Objectif : tenir
> le plus longtemps possible.

## ⚡ Le hook : la vitesse, c'est l'arme

Le pilier central qui distingue le jeu :

> **Plus le personnage va vite, plus il inflige de dégâts.**

La vitesse n'est pas un simple confort de déplacement : c'est le cœur du système de combat.
Rester immobile, c'est être faible. Le bon joueur est en mouvement permanent — il *kite*,
*dash*, traverse les salles, et frappe le plus fort quand il file le plus vite. Tout le
build tourne autour de l'entretien et de l'exploitation de cette vitesse.

## 🎮 Gameplay

- **Vue :** top-down (caméra au-dessus du personnage).
- **Visée :** manuelle — le joueur vise et déclenche ses attaques (skill-based, façon *Enter the Gungeon*).
- **Mobilité au cœur du jeu :** déplacements nerveux, dash, perso rapide. La mobilité *est* la mécanique principale (et la source des dégâts, cf. le hook ci-dessus).
- **Structure « tower » :** des salles qui s'enchaînent, peuplées de mobs variés (différentes tailles, différents types/comportements).

### 🛠️ Armes (matériel de jardin)

Le personnage peut porter **jusqu'à 2 armes simultanément**. Le sprite du perso est
**séparé** de celui des armes (pour pouvoir les combiner librement et les animer
indépendamment).

Les armes sont inspirées de l'outillage de jardinage, par ordre de progression :

| Arme | Style |
|------|-------|
| 👊 Poings | Arme de base |
| 🌱 Petite pelle | Mêlée rapide, courte portée |
| ⛏️ Pelle | Mêlée plus lourde |
| 🍴 Râteau | Mêlée à allonge / multi-cible |
| 💧 Arrosoir | Attaque à distance (jet) |
| 🔫 Karcher | Distance haute pression |
| … | *(à étendre)* |

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
- **Plateforme de dev :** Linux (testé sous **WSL2 / Ubuntu**, affichage via **WSLg**)

## 🚀 Installation & build

### 1. Rust

Le projet utilise **Rust** (edition 2024). Installe la toolchain via [rustup](https://rustup.rs/) :

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Dépendances système (Linux / Ubuntu / WSL)

Bevy a besoin de quelques bibliothèques système. Sur Ubuntu/Debian :

```bash
sudo apt update
sudo apt install -y libasound2-dev libudev-dev pkg-config libwayland-dev libxkbcommon-dev
```

| Paquet | Rôle |
|--------|------|
| `libasound2-dev` | Audio (ALSA) |
| `libudev-dev` | Détection des périphériques / manettes |
| `pkg-config` | Localisation des bibliothèques système |
| `libwayland-dev` + `libxkbcommon-dev` | Affichage de la fenêtre sous Wayland |

### 3. Compiler & lancer

```bash
cargo build      # compile (la 1re fois est longue : tout Bevy se compile)
cargo run        # compile puis lance le jeu
```

> Sous WSL2, la fenêtre s'ouvre automatiquement via **WSLg** (inclus dans Windows 11).

---

## 📍 État du projet

🚧 **Très tôt — squelette de projet.** Le code (`src/main.rs`) est encore un simple
*hello world* : le présent README fixe la **vision** et les **specs de base** avant
d'attaquer le développement.

📖 La conception détaillée (mécaniques, armes, augments, structure des runs, méta-progression,
lore…) vit dans le **[Game Design Document](docs/GDD.md)**.

### Roadmap (esquisse)

- [ ] Ouvrir une fenêtre Bevy + boucle de jeu de base
- [ ] Personnage déplaçable (top-down) avec mobilité/dash
- [ ] Système « vitesse → dégâts »
- [ ] Système d'armes (2 slots, sprite séparé) + poings
- [ ] Ennemis basiques + une salle
- [ ] Enchaînement de salles (structure « tower »)
- [ ] Système d'augments + premières synergies
- [ ] Méta-progression (déblocages permanents)
- [ ] Mode final « terrasse » (survie chronométrée)

## ❓ Questions ouvertes

- **Nom définitif** du jeu (actuellement *GameForge2026*, nom de projet).
