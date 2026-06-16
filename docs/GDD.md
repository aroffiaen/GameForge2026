# GameForge2026 — Game Design Document (GDD)

> **Version :** v0.3 — **refonte de design** suite au brainstorm Équipe Reims.
> **Changement majeur :** la mécanique « plus vite = plus de dégâts » quitte le cœur du jeu. Le **thème vitesse** devient le **moteur de progression** via un système de **Stats-Up chronométré** (salles à battre vite pour gagner des stats). Voir §3.
> **Genre :** Roguelite d'action top-down, visée manuelle, sur le thème de la **vitesse**.
> **Tech :** Rust + [Bevy](https://bevyengine.org/) `0.18`. **Build/jouée nativement sous Windows** (GPU NVIDIA/Vulkan) ; dev sous WSL. Lancement : `./play-windows.sh` (depuis WSL) ou `C:\GameForge2026\Jouer.bat`.
> **Inspirations notées :** Hades, Subway Surfers, « QTE / doors rapides ».

> ⚠️ **Statut de la refonte :** ce GDD décrit la **cible v0.3**. Le code actuel implémente encore la v0.2 (vitesse→dégâts au cœur, 8 mobs, gauntlet de vagues, etc.). La **checklist §18** liste tout ce qu'il faut migrer. On attaque après le merge du travail en cours.

---

## 0. Le pitch & la boucle

Un jardinier rétréci traverse son jardin devenu hostile pour récupérer ses outils volés par un bousier et rejoindre sa terrasse. **Le nerf du jeu : aller vite.** Pas pour taper plus fort, mais parce que **nettoyer chaque salle sous le chrono fait monter tes stats** — et les rater te les fait perdre.

```
RUN ( ~10 min )
└── 5 BIOMES (tirés parmi 6), chacun = 5 salles + 1 boss   → 25 salles + 5 boss
    • Salle 1 du run : NORMALE (pas de chrono).
    • Ensuite, après CHAQUE salle (hors boss) : 3 PORTES, une stat au-dessus de chacune.
        → tu choisis une porte = tu MISES cette stat.
        → la salle derrière est CHRONOMÉTRÉE :
              réussie sous le temps cible → tu GAGNES la stat (∝ avance)
              ratée                       → tu PERDS la stat misée (∝ retard)
    • Salle de BOSS : pas de chrono, pas de vagues — juste le boss (3 patterns).
        → après le boss : choix d'AUGMENT (3→1), PUIS les 3 portes habituelles.
        → on enchaîne sur un BIOME suivant ALÉATOIRE (non encore vu dans la run).
→ après les 5 biomes : TERRASSE (survie chronométrée sans fin, pas de chrono-stat).
→ à la mort : réveil au cabanon avec une excuse bidon aléatoire.
```

---

## 1. Piliers de design

1. **La vitesse, c'est la progression.** On ne tape pas plus fort en allant vite : on **devient plus fort** en finissant vite. Le chrono est partout (hors boss/terrasse) et pousse à l'agressivité permanente.
2. **Risque / récompense à chaque salle.** Miser une stat et la doubler — ou la perdre. Chaque porte est une décision.
3. **Glass cannon nerveux.** PV faibles, esquive reine, plus de dégâts de collision : on encaisse des **attaques** (qu'on peut lire et éviter), pas de la pénalité passive.
4. **Synergies > stats brutes.** Armes × augments × build de stats.
5. **Comique & cosy.** Ton léger, running gags, jardin attachant.

---

## 2. Lore, ton & dialogues *(inchangés v0.2)*

### 2.1 Déclencheur
Le jardinier range ses outils dans son **cabanon**, fait une sieste, se réveille **minuscule**. Un **bousier farceur** lui a volé ses outils miniaturisés et ne les rend que contre des **pattes d'insectes** (sa monnaie, qu'il amasse pour rouler une boule de pattes censée intimider le jardin).

### 2.2 Répliques du bousier (cabanon, tirées au hasard)
1. « Tes outils ? Quels outils ? Hé hé. »
2. « Encore des pattes ! Ma boule sera MAGNIFIQUE. »
3. « Rétréci ? Moi je te trouve très bien comme ça. »
4. « Reviens avec des pattes. Beaucoup. De. Pattes. »
5. « Un jour, ma boule de pattes terrifiera tout le jardin. »
6. « Je ne suis pas un voleur, je suis un collectionneur. »

Contextuelles : terrasse verrouillée → *« La terrasse ? Faut la MÉRITER. »* · cap d'outils → *« max 2 outils par run — reviens après une run »* · pas assez de pattes → *« Pas assez de pattes. Le bousier soupire. »*

### 2.3 Excuses de mort (réveil au cabanon, 14, tirées au hasard)
1. « Un ruissellement d'eau de pluie t'a charrié jusqu'au cabanon. »
2. « Un insecte super sympa t'a ramené sur son dos. »
3. « Tu t'es réveillé. C'était un rêve. Enfin… presque. »
4. « Une bourrasque t'a déposé pile devant la porte. Pratique. »
5. « Le bousier jure qu'il n'y est pour rien. Il rigole, pourtant. »
6. « Tu as glissé sur une limace. Longtemps. Très longtemps. »
7. « Un escargot livreur t'a raccompagné. Délai non garanti. »
8. « Tu t'es évanoui d'avoir trop couru. L'ironie ne t'échappe pas. »
9. « Une fourmi t'a confondu avec une miette et t'a rapporté ici. »
10. « Le tuyau d'arrosage a eu un hoquet. Te voilà rincé, et rentré. »
11. « Tu as pris un pétale dans la figure. Un GROS pétale. »
12. « Quelqu'un a crié "apéro" et tes jambes ont décidé seules. »
13. « Une taupe t'a poliment montré la sortie. Par en dessous. »
14. « Le jardin a demandé une pause. Toi aussi, apparemment. »

---

## 3. Mécanique signature — Stats-Up chronométré

### 3.1 Les 3 portes
Après **chaque salle** (sauf salle de boss et terrasse), 3 portes s'affichent, **une stat différente au-dessus de chacune** (tirées parmi les stats du §3.3). Le joueur **choisit une porte = il MISE cette stat**.

### 3.2 La salle chronométrée derrière la porte
La salle derrière la porte choisie a un **chrono cible** :
- **Réussie sous le temps cible** → **+2 points** de la stat misée **par seconde d'avance** (sur le temps cible).
- **Ratée (temps dépassé)** → **−1 point** de la stat misée **par seconde de retard**, plafonné à **−15 points**.
- Asymétrie volontaire : on gagne plus vite qu'on ne perd, mais l'échec pique.

**Règles :**
- Seule la **salle 1 du run** est normale (pas de chrono).
- **Salles de boss** : jamais de chrono.
- **Salles élite** : chrono **plus permissif** (seuil plus large).
- **Temps cible** : adapté à la difficulté de la salle (nombre/type d'ennemis, taille). *Les valeurs moyennes seront tunées après playtest de la refonte.*

### 3.3 Les stats *(7 stats)*
Chaque stat est un **pourcentage**, **base 100 %**, **plancher 25 %**, **PAS de cap haut** (le snowball doit pouvoir partir loin — c'est le scaling de la Terrasse qui finit par arrêter la run). Un « point de stat » = **1 %**.

| Stat | Effet (base à 100 %) | Formule |
|------|----------------------|---------|
| **PV** | PV max | `50 × PV%/100` |
| **Régén PV** | régén passive | `1.0 HP/s × Régén%/100` |
| **DMG** | dégâts d'arme | `base × DMG%/100` |
| **Résistance** | dégâts subis | `dégâts × 100/Rési%` (200 %→½ dégâts, 25 %→×4) |
| **Move Speed** | vitesse | `250 px/s × MS%/100` |
| **Attack Speed** | cadence | intervalle `= base_cd × 100/AS%` (200 %→2× plus rapide) |
| **Dash CD** | cooldown dash | `1.25 s × 100/DashCD%` (200 %→0.62 s) |

> Toutes démarrent à 100 %, plancher 25 %, sans plafond. *(Valeurs de base à confirmer au playtest.)*

### 3.4 HUD
Affiche le **chrono en cours vs cible**, la **stat misée**, et le **panneau de stats** (les 7 valeurs en %). Feedback fort à la réussite/échec (gain/perte chiffré en toast).

---

## 4. Mobilité, dash & survie

- **Déplacement** à inertie : magnitude + direction séparées (on garde son élan en tournant), montée en régime progressive, freinage plus vif. Move Speed de base **250 px/s** × (MS%/100).
- **Dash** court et net (~100 px, i-frames couvrant le dash). Cooldown réduit par la stat **Dash CD**. Augments : 2e charge, i-frames+, dash offensif…
- **PV de départ : 50** × (PV%/100).
- **Plus de dégâts de collision.** On ne subit que des **attaques** télégraphiées (voir §8). I-frames courtes après un coup encaissé.

### 4.x Augment « Élan » — l'ancienne mécanique signature *(optionnelle)*
La vitesse→dégâts revient en **augment** avec des ratios doux : **×0.8 à l'arrêt → ×1.5 à pleine vitesse** (au lieu de ×0.4→×2.5). Pour les joueurs qui veulent un style « momentum ».

---

## 5. Armes

- **Dual wielding** : le perso porte **2 armes en même temps**, **pas de doublon** (2 armes différentes). Sprite des armes séparé du corps (couche bras).
- **Visée manuelle** (clic gauche = arme 1, clic droit = arme 2).
- Dégâts = `base_arme × (DMG%/100) × multiplicateurs(augments)`. Cadence = `base × (AS%/100)`.
- **⚠️ Plus aucun knockback** sur quelque arme que ce soit (retrait global, important).
- **Déclenchement** : *Frappe* (clic = 1 coup), *Maintien* (bouton tenu = effet continu, **sans coût** ni cooldown qui tourne — hold-to-shoot).

### Roster retenu (10 armes)

| Arme | Déclench. | Forme / portée | Comportement |
|------|-----------|----------------|--------------|
| **Pesticide** | **Maintien** | traînée au sol | Tant que tenu : pose une traînée de poison (DoT, timer reset au contact). |
| **Pelle** | Frappe | **AoE cercle** autour du perso | Coup de zone façon **Q de Darius** (anneau). 1 main. |
| **Râteau** | Frappe | cône **devant** (midrange) | **Attire** les ennemis vers le perso, **devant uniquement** (plus tout autour). |
| **Karcher** | **Maintien** | **spray 60°** (midrange) | Jet en éventail continu, **pas de kb**. |
| **Tronçonneuse** | **Maintien** | **ligne** mid/long | Dégâts soutenus en continu. **Tant qu'active : perso ralenti + arme 2 inutilisable.** |
| **Pioche** | Frappe | frappe de **zone** ~midrange | Impact AoE à distance moyenne. |
| **Faux** | Frappe | **cône ~50°** long range | Grand balayage à distance. |
| **Hache** | Frappe | **lancée**, très longue portée | Va jusqu'au mur. Gros dégâts, **long CD**. |
| **Serpe** | Frappe | **AoE ~300°** short/mid | Balaie presque tout autour, **rapide**. |
| **Pic de vigne** | Frappe | **lance** CaC→longue portée | Estoc qui s'allonge (comme une lance). |

> *(Dégâts/cd/portée chiffrés : valeurs de départ proposées à l'implémentation, à tuner — cf. §3 pour l'équilibrage des stats.)*

---

## 6. Structure d'un run

```
RUN = 5 BIOMES (tirés parmi 6, sans répétition), chacun :
  ├── 5 salles : la 1re du run est NORMALE ; toutes les autres sont CHRONO
  │     (atteintes via le choix d'une des 3 portes-stat)
  └── 1 salle de BOSS : pas de chrono, pas de vagues, juste le boss (3 patterns)
        → AUGMENT (3 → 1)   [important, conservé]
        → 3 PORTES-STAT
        → BIOME suivant = aléatoire parmi ceux pas encore vus dans la run
→ TERRASSE après les 5 biomes (survie, voir §10)
```

### Salles spéciales
- **Élite** : porte signalée par un **losange violet**. La salle contient un **mob élite** (mob random agrandi avec **une stat boostée**) + du menu fretin. Chrono **plus permissif**. Grosse récompense.
- **Pas de salles trésor ni secrètes** : supprimées (le système de portes-stat remplace ces respirations).

### Peuplement & scaling
- **Le nombre d'ennemis n'est plus aléatoire** (fixé par salle/profondeur) ; **les types présents le sont** (tirés dans le bestiaire du biome).
- **Scaling linéaire** des ennemis avec la profondeur (au minimum), avec une **petite modulation selon la réussite/échec de la salle précédente** (si tu enchaînes les réussites, ça monte un peu plus).

---

## 7. Biomes

**6 biomes dans le jeu ; une run en traverse 5** (aléatoires, sans répétition), avec leurs **bosses associés**. Puis la **Terrasse** (zone finale, §10).

| Biome | Boss |
|-------|------|
| **Jardin** | Araignée |
| **Gravier** | Mille-Pattes |
| **Boue** | Grompaud (crapaud, réf. *Gromp* de LoL) |
| **Terre Sèche** | Roger le Scorpion |
| **Potager** | Méga-Limace |
| **Dalles** | Araignée *(géante — la « 2.5k PV » ?)* |
| *(finale)* | **Terrasse** — survie sans fin (§10) |

> ⚠️ **À confirmer :** les biomes **Jardin** et **Dalles** ont tous deux « Araignée » comme boss. Volontaire (une petite araignée tôt, la géante 2.5k PV sur les Dalles tard) ? Ou l'un des deux doit changer ?

---

## 8. Bestiaire

**3 archétypes × 2 types = 6 mobs.** Plus de dégâts de collision : chaque archétype a sa **propre attaque**.

| Archétype | Types | Attaque |
|-----------|-------|---------|
| **Chase** (poursuite) | **Fourmi**, **Escargot** | fonce au contact et **frappe au corps-à-corps** (windup + coup). |
| **Lunge** (charge) | **Araignée**, **Criquet** | **ruée** : inflige des dégâts **pendant la charge**. |
| **Range** (distance) | **Guêpe**, **Cigale** | tire une **boule rouge** (projectile). |

- **Élites** : un mob random, **plus gros**, avec **une stat boostée** (PV, vitesse, dégâts… selon le tirage).
- **Scaling** : linéaire en profondeur (+ modulation réussite/échec, §6).
- **PNJ** : le **Bousier** (vendeur du cabanon, sprite : scarabée poussant sa boule).

### Boss
6 prédateurs du jardin, **1 boss = 3 patterns**, salle de boss **sans vagues** : **Araignée** (Jardin), **Mille-Pattes** (Gravier), **Grompaud** (Boue), **Roger le Scorpion** (Terre Sèche), **Méga-Limace** (Potager), **Araignée géante** (Dalles). Cf. §7.

---

## 9. Augments & méta

- **Augment après chaque boss (3 → 1)** — conservé, important.
- **Augments ≠ stats.** Les augments sont des **effets de gameplay / keystones** (modifs de mécaniques : dash, poison qui se propage, mods d'arme, effets en mouvement…), **indépendants** du système de stats-up. Les bonus de **% bruts** (PV, DMG, vitesse…) relèvent **uniquement** des stats-up — donc pas d'augment « +X % DMG ». Idées tableau : **Dash**, **HolyCross**, **Élan** (§4.x).
- **Méta-progression** *(inchangée v0.2)* : monnaie unique **Pattes** (sauvegarde `save.ron`), déblocage d'armes par accomplissements + rachat au bousier (cap 2/run), upgrades permanents, cabanon hub.

---

## 10. Mode Terrasse *(inchangé v0.2)*
Atteint après les 5 biomes (ou accès direct depuis le cabanon une fois débloqué). **Survie infinie** qui monte en puissance, boss surprises, **meilleur temps sauvegardé**. **Pas de chrono-stat** ici.

---

## 11. Contrôles
ZQSD/WASD déplacement · souris visée · clic G/D armes · Espace/Shift dash · E interagir · Échap pause · 1/2/3 choix (portes/augments).

---

## 12. Direction artistique
**Pixel art**, perso en **couches séparées** : jambes (orientées déplacement) · bras (orientés visée, tiennent les armes) · chapeau (au-dessus, teinté : bleu i-frames de dash, rouge dégâts). Feedback de vitesse (traînée). Police DejaVu Sans embarquée. *(Sprites en cours d'intégration.)*

---

## 13. Architecture technique *(cible)*
Modules Bevy : `common` (états, stats, dégâts), `player` (déplacement, dash, couches sprites), `stats` *(nouveau : valeurs de stats + Stats-Up + chrono)*, `weapons`, `enemies` (3 archétypes, attaques, pas de collision), `boss`, `biomes`, `rooms` (5×5, portes, chrono, élites), `augments`, `meta`, `cabanon`, `terrasse`, `ui`.

---

## 18. Refonte v0.3 — Checklist d'implémentation

### A. Système de stats (nouveau cœur)
- [ ] Ressource `Stats` : 7 stats en % (PV, RégénPV, DMG, Résistance, MoveSpeed, AttackSpeed, DashCD), base 100 %, plancher 25 %.
- [ ] Brancher chaque stat sur son effet (PV max, régén, dégâts, réduction, vitesse, cadence, cooldown dash).
- [ ] Panneau de stats dans le HUD (7 valeurs).

### B. Stats-Up chronométré
- [ ] Système de **3 portes** après chaque salle (hors boss/terrasse), 1 stat random distincte par porte.
- [ ] Choix d'une porte = **mise** de la stat ; la salle suivante devient chrono.
- [ ] **Chrono** par salle (seuil adapté à la difficulté ; permissif si élite).
- [ ] Réussite → **+2 pts/s d'avance** sur la stat misée. Échec → **−1 pt/s de retard, cap −15**.
- [ ] Toasts de gain/perte ; chrono visible dans le HUD.
- [ ] 1re salle du run = normale (pas de chrono).

### C. Retrait / déplacement de la vitesse→dégâts
- [ ] Retirer le `mult` vitesse→dégâts du calcul de dégâts cœur.
- [ ] Le reproposer en **augment « Élan »** : ×0.8 immobile → ×1.5 vitesse max.

### D. Refonte ennemis
- [x] Nouveau roster **6 mobs** (Fourmi, Escargot / Araignée, Criquet / Guêpe, Cigale).
- [x] **Supprimer les dégâts de collision.** *(mobs ; le boss garde les siens)*
- [x] Attaques par archétype : chase = **mêlée télégraphée**, lunge = **dégâts pendant la ruée**, range = **boule rouge**.
- [x] **Nombre d'ennemis fixe** par salle ; **types aléatoires** (pool du biome).
- [x] **Scaling linéaire** + modulation selon réussite/échec de la salle précédente *(via `momentum`)*.
- [x] **Élites** : mob random agrandi + 1 stat boostée ; porte marquée d'un **losange violet**.

### E. Structure de run
- [x] **5 biomes / run** (tirés parmi 6, sans répétition), **5 salles + 1 boss** chacun (25 salles). *(La run démarre toujours au Jardin, puis 4 biomes aléatoires non vus — choix de design, pour la courbe de difficulté.)*
- [x] **Supprimer le gauntlet de vagues** au boss (salle de boss = juste le boss).
- [x] Après boss : **augment (3→1)** puis **3 portes-stat**, puis **biome aléatoire** non encore vu.
- [x] Retirer l'écran de **choix de biome** (la transition se fait via les portes-stat post-boss).

### F. Armes
- [x] **Retrait global du knockback** (toutes armes). *(champ kb et kb_mult supprimés)*
- [x] Roster **10 armes** avec leurs comportements (§5) : Pesticide (Maintien), Pelle (AoE anneau), Râteau (pull cône devant), Karcher (spray 60°, Maintien), Tronçonneuse (ligne, Maintien, ralentit + bloque arme 2), Pioche (zone midrange), Faux (cône 50°), Hache (lancée jusqu'au mur, gros CD), Serpe (AoE 300° rapide), Pic de vigne (estoc). *(valeurs chiffrées à tuner au playtest)*
- [x] Mécanique **Maintien** (hold-to-shoot, sans coût ni CD continu) pour Pesticide / Karcher / Tronçonneuse.
- [x] **Dual wield** : 2 armes différentes, pas de doublon. *(garanti par le toggle de l'établi)*

### G. Biomes / contenu
- [x] **6 biomes** : Jardin, Gravier, Boue, Terre Sèche, Potager, Dalles (+ Terrasse finale).
- [x] Bosses : Araignée, **Mille-Pattes** *(neuf : corps segmenté, ruée en ligne, salve, bave)*, Grompaud, Scorpion, Méga-Limace, **Araignée géante** *(réutilise l'IA Araignée, gros PV — placeholder §19)*.
- [x] Adapter palettes / bestiaires par biome.

### H. Nettoyage
- [x] **Supprimer salles trésor & secrètes.** *(fait en B)*
- [x] Trier le **pool d'augments** : enlever les augments de **% brut** (doublons des stats), garder les **mécaniques**. *(Jambes de criquet, Aiguillon, Carapace retirés ; Café du bousier conservé car « accélération » ≠ stat.)*

---

## 19. Restant à trancher
- ✅ **6 biomes + bosses** : figés (§7).
- ✅ **Profils des 10 armes** : comportements figés (§5) ; valeurs chiffrées à tuner.
- ✅ **Formules de stats** : posées (§3.3), à affiner au playtest. 1 pt = 1 %.
- ✅ **Salles trésor/secrètes** : supprimées.
- ✅ **Augments vs stats** : indépendants (augments = mécaniques, pas de % brut) (§9).
- ✅ **Cap haut des stats** : aucun (snowball libre, la Terrasse borne la run).

- ✅ **Jardin & Dalles = même boss (Araignée)** : assumé (placeholder manque d'inspi), différenciable plus tard.
- ✅ **Armes continues = Maintien (hold-to-shoot)**, sans coût ni cooldown continu.

Encore ouvert :
- Valeurs chiffrées des armes (dégâts/cd/portée) — à poser puis tuner au playtest.

---

*Document maintenu au fil du projet. La v0.3 est la cible ; le code migre selon la checklist §18 après le merge en cours.*
