# GameForge2026 — Game Design Document (GDD)

> **Version :** v0.2 — *document mis à jour pour refléter le jeu réellement implémenté (vertical slice complète et jouable).*
> **Statut :** prototype jouable de bout en bout. Les valeurs chiffrées ci-dessous sont celles **réellement codées** et servent de base d'équilibrage. Les points encore ouverts sont listés au §17.
> **Genre :** Roguelite d'action top-down, visée manuelle, sur le thème de la **vitesse**.
> **Tech :** Rust + [Bevy](https://bevyengine.org/) `0.18` (voir [README](../README.md)).
> **Plateforme de jeu :** **build native Windows** (le rendu via WSLg n'a pas de GPU exploitable → on compile et on lance sous Windows). Le code reste développé sous WSL/Ubuntu.

---

## 0. Compte rendu — où en est le jeu

**Tout le cœur du jeu tourne.** Une partie complète est jouable :

```
CABANON (hub)
  → établi (choix de 2 outils) · bousier (boutique) · porte Jardin · porte Terrasse (verrouillée au départ)
  → LE JARDIN (run) : 5 biomes enchaînés
        chaque biome = 1 à 3 salles (combat / élite / trésor) puis 1 salle de BOSS (gauntlet 3 vagues)
        après chaque boss : 1 augment (choix 3→1) puis choix du biome suivant (2 options)
  → après le 5e biome : LA TERRASSE (survie chronométrée sans fin)
  → à la mort : écran « réveil au cabanon » avec une excuse bidon aléatoire
```

### Ce qui est implémenté
- **Mécanique signature vitesse→dégâts** (modèle « flat », voir §3.1).
- **Déplacement** à inertie (magnitude + direction séparées) et **dash** court avec i-frames.
- **6 armes** sur **2 slots** (clic gauche / clic droit), sprites d'armes séparés du corps.
- **8 types d'ennemis** (3 IA : poursuite, charge, distance) + **3 boss à patterns**.
- **3 biomes** (palette, bestiaire, boss dédiés) + scaling de difficulté par profondeur de run.
- **Boucle de salles** complète : combat, élite, trésor, gauntlet de boss, portes, choix de biome.
- **18 augments** avec synergies + keystones, choix 3→1 après boss, salles trésor.
- **Méta-progression** : monnaie unique (Pattes) **sauvegardée sur disque** (`save.ron`), déblocage d'armes par accomplissements + rachat chez le bousier (cap 2/run), upgrades permanents.
- **Mode Terrasse** : survie infinie qui monte en puissance, boss surprises, record sauvegardé.
- **HUD** complet, pause, écran de mort, toasts d'annonce, feedback visuel de vitesse.

### Ce qui n'est PAS encore là
- Pas d'**audio** (ni musique ni SFX).
- Pas de **pixel art** : tout est en formes/rectangles colorés (placeholders lisibles).
- Pas de **salles secrètes**, pas de manette, pas de leaderboard en ligne.

---

## 1. Vision & piliers de design

**Pitch d'une ligne :** un jardinier rétréci traverse son jardin devenu hostile, où **plus il va vite, plus il frappe fort**, pour récupérer ses outils volés par un bousier et rejoindre sa terrasse.

### Les piliers (ce qui ne doit jamais être trahi)

1. **La vitesse, c'est l'arme.** Le mouvement n'est pas un confort, c'est la source des dégâts. Rester immobile = dégâts plancher.
2. **Glass cannon nerveux.** PV faibles, esquive reine. On survit par le placement, pas par le tank.
3. **Runs courtes, builds variés.** ~12-15 min par run, augments tranchants et peu nombreux.
4. **Synergies > stats brutes.** La rejouabilité naît des combinaisons (armes × augments).
5. **Comique & cosy.** Ton léger, running gags, un jardin attachant.

---

## 2. Lore, ton & dialogues

### 2.1 Le déclencheur (running gag)

Le jardinier range ses outils dans son **cabanon**, fait une sieste, et se réveille **minuscule**. Aucune explication — c'est la blague. Un **bousier farceur** en a profité pour lui **voler ses outils** (miniaturisés eux aussi) et refuse de les rendre, sauf **contre des pattes d'insectes** : sa monnaie, qu'il amasse pour rouler une énorme **boule de pattes** censée intimider tout le jardin.

### 2.2 Le bousier — répliques au cabanon (implémentées)

À chaque visite du cabanon, le bousier sort une réplique aléatoire affichée sous lui :

1. « Tes outils ? Quels outils ? Hé hé. »
2. « Encore des pattes ! Ma boule sera MAGNIFIQUE. »
3. « Rétréci ? Moi je te trouve très bien comme ça. »
4. « Reviens avec des pattes. Beaucoup. De. Pattes. »
5. « Un jour, ma boule de pattes terrifiera tout le jardin. »
6. « Je ne suis pas un voleur, je suis un collectionneur. »

Répliques contextuelles du bousier :
- Tenter la terrasse verrouillée : *« Le bousier ricane : "La terrasse ? Faut la MÉRITER." »*
- Achat refusé (cap 2 outils/run atteint) : *« Le bousier refuse : max 2 outils par run — reviens après une run. »*
- Pas assez de pattes : *« Pas assez de pattes. Le bousier soupire. »*
- Outil récupéré : *« {Outil} récupéré ! Passe à l'établi pour l'équiper. »*
- Upgrade acheté : *« Upgrade acheté. Le bousier compte ses pattes. »*

### 2.3 La boucle de la mort — excuses bidon (implémentées)

À chaque mort, le joueur « se réveille au cabanon » avec une excuse tirée au hasard parmi 14 :

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

L'écran de mort affiche : *« Tu te réveilles au cabanon. »* + l'excuse + un récap (insectes dézingués, pattes ramassées, temps).

### 2.4 Autres annonces (toasts) en jeu

- Entrée en salle de combat : *« {Biome} — salle {n}/{total} »*
- Salle d'élite : *« SALLE D'ÉLITE — grosse bête, grosse récompense »*
- Salle trésor : *« Salle au trésor — choisis UN cadeau (E) »*
- Antre du boss : *« ANTRE DU BOSS — vague 1/3 ({Biome}) »* → *« Vague 2/3 — ça grossit… »* → *« Vague 3/3 — les gros calibres. »* → *« {Boss} entre en scène ! »*
- Récompenses trésor : *« +20 PV. Ça repousse. »* / *« Une pluie de pattes ! Le bousier serait fier. »*
- Entrée terrasse : *« LA TERRASSE. Ce n'était pas un refuge. Tiens bon ! »*
- Boss surprise en terrasse : *« {Boss} s'invite sur la terrasse ! »*
- Augment ramassé (trésor) : *« Augment trouvé : {nom} ! »*

### 2.5 L'objectif & le twist

Le jardinier veut **rejoindre sa terrasse**. **Plot twist :** ce n'est pas un refuge — c'est l'épreuve ultime, le **mode survie chronométré sans fin** (§10).

---

## 3. Game feel & mécaniques cœur *(valeurs implémentées)*

### 3.1 Vitesse → dégâts — modèle « flat » *(mécanique signature)*

**Décision v0.2 :** on a abandonné le `lerp(MULT_MIN, MULT_MAX, ratio)` au profit d'un modèle **basé sur la vitesse absolue** :

```
mult_vitesse = max( vitesse_actuelle / SPEED_PER_MULT , DMG_MULT_MIN )
dégâts = dégâts_base_arme × mult_vitesse × dmg_mult(augments) × (1 + momentum) × (1.5 si burst de dash)
```

- `SPEED_PER_MULT = 100` → **100 px/s = ×1.0**, **250 px/s = ×2.5**.
- `DMG_MULT_MIN = 0.4` → plancher quand on est (quasi) immobile.
- La vitesse effective est plafonnée à `max_speed` pour le calcul (le dash ne donne pas un mult absurde, il donne pile le mult de pointe).
- **Conséquence clé voulue :** comme le calcul part de la vitesse absolue, **chaque bonus de vitesse relève aussi le plafond de dégâts**. Un augment +15 % de vitesse → 287 px/s → ×2.87.

### 3.2 Déplacement — inertie magnitude + direction

**Décision v0.2 :** la vélocité gère séparément sa **magnitude** (norme) et sa **direction**.

- `max_speed` de base = **250 px/s**.
- `accel` = **850 px/s²** (×1.4 avec l'augment Caféine) → ~0.3 s pour atteindre la pointe.
- **Freinage** (touches relâchées) = `accel × 2.0`, en conservant la direction → s'arrêter fait chuter la vitesse (donc les dégâts) plus vite qu'on ne l'a gagnée.
- **Pivot** : quand on tourne, la direction pivote vers l'input (lerp facteur `14 × dt`) **sans perdre la magnitude** → on garde son élan en changeant d'angle (correctif majeur de game feel).
- Sur-vitesse (sortie de dash) résorbée vers `max_speed` à `accel × 2.5`.

### 3.3 Dash

- **Dash court et net** : `DASH_SPEED = 640 px/s` pendant `DASH_DURATION = 0.16 s` → **~100 px** de distance fixe (une esquive, pas un long burst).
- **I-frames** : `0.16 + 0.02 s` de base (≈ couvre le dash), `+0.15 s` avec l'augment Esquive féline.
- **Cooldown** : `1.25 s` (réduit de 10 % par rang d'upgrade « dash »). **1 charge** de base, **2** avec Double détente.
- **On ne peut pas attaquer pendant le dash**, sauf augment **Dash offensif** (qui ouvre aussi une fenêtre de **burst ×1.5** de 0.8 s en sortie).

### 3.4 Survie (glass cannon)

- **PV de départ : 50** (+8 par rang d'upgrade PV, +15 par augment Carapace).
- À la prise d'un coup : **0.6 s d'i-frames** + léger recul. Les projectiles ennemis donnent 0.5 s d'i-frames.
- Pas de soin entre salles par défaut ; soin uniquement via salle trésor (+20), augment Rosée (+12 après boss), upgrade PV, ou Photosynthèse.

### 3.5 Visée & contrôle

- **Visée manuelle** à la souris, pas d'auto-aim. Chaque arme se déclenche soi-même (clic G = slot 1, clic D = slot 2).
- Armes **Maintien** (arrosoir, karcher) : effet continu tant que le bouton est tenu.

---

## 4. Armes *(6 implémentées, valeurs réelles)*

Le perso porte **2 armes max**, sprites séparés du corps, orientés vers la visée. Profils : **Frappe** (clic = 1 coup), **Maintien** (tenu = continu), **Utilitaire** (cooldown).

| Arme | Profil | Dégâts | Cooldown | Portée/Rayon | Recul | Effet |
|------|--------|--------|----------|--------------|-------|-------|
| 👊 **Poings** | Frappe | 6 | 0.25 s | 28 / 24 | 70 | Arme de base, courte portée. |
| 🌱 **Petite pelle** | Frappe | 9 | 0.28 s | 40 / 28 | 100 | Mêlée rapide, DPS régulier. |
| ⛏️ **Pelle** | Frappe | 24 | 0.75 s | 48 / 38 | 300 | Lourde, gros coup, gros recul. |
| 🍴 **Râteau** | Utilitaire | 4 | 2.2 s | rayon 230 | — | **Attire** tous les ennemis proches vers le joueur. |
| 💧 **Arrosoir** | Maintien | 14 (DPS poison) | dépôt 0.09 s | rayon 26 | — | Pose une **traînée de pesticide** au sol. |
| 🔫 **Karcher** | Maintien | 32 (DPS) | tick 0.07 s | ligne 235 / largeur 16 | 60 | Jet haute pression + recul continu. |

> Tous les dégâts ci-dessus sont **avant** le multiplicateur de vitesse et les augments.

### Mécaniques détaillées
- **Arrosoir / poison :** la flaque vit **3.2 s** ; un ennemi dans le poison subit la trame (`Poisoned`, dégâts toutes les 0.4 s, durée 2.5 s) ; **rester dans le pesticide rafraîchit son timer** → un ennemi immobilisé dans le poison fond. Les ennemis « mous » (limace) prennent **×1.6** de dégâts de poison.
- **Râteau :** tire les ennemis dans un rayon vers le joueur (combo canonique : râteau → arrosoir). Avec l'augment Râteau aimanté : rayon ×1.4 + ralentissement 1.5 s.
- **Karcher :** dégâts en ligne (cône fin), recul poussant les ennemis (utile pour les enfoncer dans le poison).

---

## 5. Augments & synergies *(18 implémentés)*

**Acquisition :** après chaque boss, **3 augments proposés → 1 choisi**. Salles trésor : 1 augment aléatoire bonus. Pool resserré, ~5-7 augments/run. Le tirage exclut les augments uniques déjà pris ; trois augments sont **cumulables** (Jambes de criquet, Aiguillon, Carapace).

| Catégorie | Augment | Effet exact |
|-----------|---------|-------------|
| **Vitesse** | Jambes de criquet | +15 % vitesse max **par stack** (donc +plafond de dégâts). |
| | Café du bousier | +40 % d'accélération. |
| | Adrénaline | Sous 30 % de PV : +25 % de vitesse. |
| **Dash / burst** | Dash offensif | Attaque pendant le dash + **burst ×1.5** de 0.8 s en sortie. |
| | Double détente | +1 charge de dash. |
| | Esquive féline | I-frames du dash +0.15 s. |
| | Sortie explosive | Explosion AoE en fin de dash (∝ vitesse, rayon 95). |
| **Mods d'arme** | Pesticide concentré | Poison ×1.6. |
| | Râteau aimanté | Râteau : rayon +40 % et ralentit 1.5 s. |
| | Buse haute pression | Karcher : +25 % dégâts, recul ×1.6. |
| | Pelle élargie | Armes de frappe : zone ×1.35. |
| | Aiguillon | +20 % de dégâts de base **par stack**. |
| **Momentum** | Momentum | +2 %/s de dégâts en mouvement, max +40 %. |
| | Photosynthèse | Régénère 2 PV/s au-dessus de 70 % de vitesse. |
| **Keystones** | Épidémie | Le poison **se propage** aux ennemis proches à la mort (rayon 110). |
| | Traînée toxique | Le **dash laisse une traînée de pesticide**. |
| **Défensif** | Carapace de fortune | +15 PV max **par stack**. |
| | Rosée du matin | Soigne 12 PV après chaque boss. |

### Synergies cultivées
- **Râteau + Arrosoir** : on agglutine puis on noie le paquet.
- **Dash offensif + vitesse** : hit-and-run dévastateur.
- **Momentum + Épidémie** : on ne s'arrête plus, le jardin fond derrière soi.
- **Karcher + poison** : on repousse les ennemis dans nos flaques.

---

## 6. Structure d'un run *(implémentée)*

```
RUN ( ~12-15 min )
└── 5 BIOMES enchaînés
    └── BIOME
        ├── 1 à 3 salles (tirage aléatoire 1..=3)
        │     1re salle : toujours COMBAT
        │     suivantes : 70 % Combat · 15 % Élite · 15 % Trésor
        └── 1 salle de BOSS (gauntlet 3 vagues)
              → récompense : 1 augment (choix 3→1)
              → puis choix du biome suivant (2 options) — sauf après le 5e biome
→ après les 5 biomes : TERRASSE (survie / score)
→ à la mort : retour au cabanon (excuse bidon)
```

### Scaling par profondeur (`biome_index` 0→4)
- PV ennemis : `× (1 + 0.22 × biome_index)`.
- Dégâts ennemis : `× (1 + 0.15 × biome_index)`.
- Densité de spawn en salle de combat : petits `4-6 + profondeur`, moyens `1-2 + profondeur/2`, gros `0-1` (à partir du 3e biome).

### Salle de boss — gauntlet 3 vagues
| Vague | Contenu |
|-------|---------|
| 1 | `7 + biome_index` petits sbires (tier 0 du biome). |
| 2 | `4 + biome_index/2` sbires moyens (tier 1). |
| 3 | `2 + biome_index/2` gros mobs (tier 2). |
| Boss | Apparaît une fois la vague 3 nettoyée. |

### Salles spéciales
- **Élite :** 1 ennemi renforcé (×3 PV, ×1.5 dégâts, ×1.5 taille, ×6 pattes, teinte magenta) + 3 petits. Récompense : **1 augment**.
- **Trésor :** 3 piédestaux, on en prend **un** (E) : **Soin +20 PV** / **+35 Pattes** / **Augment mystère**.

### Choix de biome (pool à 3, déterministe)
Après un boss, on propose les **2 autres** biomes (jamais le courant). Règle limpide tant que le pool est à 3.

---

## 7. Biomes *(3 implémentés)*

| Biome | Ambiance (couleurs) | Bestiaire (tiers nuée / moyen / gros) | Boss |
|-------|---------------------|----------------------------------------|------|
| **La Plaine** *(départ)* | Vert, herbe rase, lumineux | Puceron, Fourmi / Araignée, Moustique / Scarabée | Mémé Mygale (araignée) |
| **La Savane** | Terreux, sec, doré | Fourmi, Puceron / Guêpe, Araignée / Scarabée, Escargot | Roger le Scorpion |
| **La Jungle** | Sombre, mousseux, humide | Moustique, Puceron / Limace, Guêpe / Escargot, Scarabée | Grompaud (crapaud) |

Une run traverse **5 biomes**, donc certains se répètent (jamais deux fois d'affilée) — c'est voulu, la difficulté monte avec la profondeur.

---

## 8. Bestiaire *(8 ennemis + 3 boss implémentés)*

### 8.1 Ennemis (stats de base, avant scaling)

| Ennemi | PV | Vitesse | Dégâts | Rayon | Pattes | IA | Vuln. poison |
|--------|----|---------|--------|-------|--------|-----|-----|
| Puceron | 8 | 115 | 4 | 8 | 1 | Poursuite | ×1.0 |
| Fourmi | 14 | 150 | 5 | 9 | 1 | Poursuite | ×1.0 |
| Araignée | 26 | 170 | 8 | 11 | 3 | **Charge** (bond quand proche) | ×1.0 |
| Moustique | 10 | 175 | 4 | 8 | 2 | **Distance** (150-240, tir 2 s) | ×1.0 |
| Guêpe | 22 | 145 | 7 | 11 | 3 | **Distance** (170-260, tir 1.6 s) | ×1.0 |
| Scarabée | 48 | 85 | 9 | 14 | 3 | Poursuite (tank) | ×1.0 |
| Escargot | 85 | 38 | 12 | 17 | 4 | Poursuite (gros tank) | ×1.0 |
| Limace | 38 | 48 | 6 | 13 | 2 | Poursuite (mou) | **×1.6** |

Comportements : **Poursuite** (fonce), **Charge** (strafe puis bond rapide), **Distance** (garde sa distance, strafe, tire des projectiles). Une **séparation** empêche l'empilement. Les empoisonnés prennent une teinte verte.

### 8.2 Boss *(noms + patterns implémentés)*

- **Mémé Mygale** (Plaine, araignée) — 360 PV. Skitter saccadé vers le joueur, puis alterne : **bond AoE** (télégraphe au sol, intouchable en l'air, éclaboussure de toile à l'atterrissage), **jet de toile radial** (12 projectiles en cercle) et **invocation** de 2-3 araignéeaux.
- **Roger le Scorpion** (Savane) — 360 PV. Orbite autour du joueur, alterne **charge de pinces télégraphée** (vire rouge, fonce en ligne) et **salves de dard venimeux** (3 salves de 3 projectiles verts en éventail).
- **Grompaud** (Jungle, crapaud — clin d'œil au *Gromp* de League of Legends, le crapaud de jungle) — 470 PV. Alterne **bond AoE** (télégraphe au sol, intouchable en l'air, crachats toxiques qui laissent des flaques à l'atterrissage) et **coup de langue** en ligne télégraphé.

Chaque boss est précédé de son gauntlet de 3 vagues thématisées. Une barre de vie de boss s'affiche en haut de l'écran.

### 8.3 PNJ — Le Bousier
Vendeur du cabanon (§11). Sa **boule de pattes** (visible à côté de lui) grossit avec ta fortune accumulée.

---

## 9. Équilibrage *(état actuel — en cours de tuning)*

Valeurs de combat déjà citées (§3, §4, §8). Tuning en cours côté **game feel** (vitesse/accel/dash/courbe de dégâts), itéré en live. Les leviers principaux exposés :

| Réglage | Valeur actuelle |
|---------|-----------------|
| Vitesse de pointe | 250 px/s |
| Accélération | 850 px/s² |
| Freinage | accel ×2.0 |
| Vitesse de pivot | lerp 14 |
| Dash | 640 px/s × 0.16 s (~100 px) |
| px par ×1.0 de dégât | 100 |
| Plancher de dégâts | ×0.4 |
| PV de départ | 50 |

> Astuce dev : `GF_WIN=960x540` règle la taille de fenêtre, `GF_FPS=1` affiche le FPS.

---

## 10. Mode Terrasse *(implémenté)*

- Atteint après les **5 biomes** (ou en accès direct depuis le cabanon une fois débloqué — dans ce cas la run repart « à nu », sans le build).
- Arène plus grande (1240×680). **Survie infinie** : ennemis en continu, montée en puissance avec le temps :
  - Intervalle de spawn : `1.6 - temps×0.015 s`, plancher 0.45 s.
  - Nombre par vague : `1 + temps/25`, max 4.
  - Chance d'élite : `temps/600`, max 15 %.
  - Échelle de stats : `1 + temps/45`.
  - **Boss surprise** toutes les 75 s (à partir de 60 s), échelle `1 + temps/90`.
- **Score** : temps tenu + kills. **Meilleur temps sauvegardé** (`best_terrasse`) et affiché au cabanon.
- Atteindre la terrasse **via une run** débloque l'accomplissement du Karcher.

---

## 11. Méta-progression & économie *(implémentée)*

### 11.1 Le hub — le cabanon
Petit lieu à l'échelle mini. Interactions (touche E) : **Établi** (choix des 2 outils parmi les débloqués), **Bousier** (boutique), **porte Jardin** (run), **porte Terrasse** (verrouillée tant que non atteinte). HUD du hub : Pattes, record terrasse, nombre de runs, nombre de morts.

### 11.2 La monnaie — les Pattes
**Monnaie unique**, gagnée en tuant des insectes (chaque ennemi lâche ses pattes au sol, aimantées vers le joueur), **conservée entre les runs** (sauvegarde `save.ron`). Gain augmentable de +15 %/rang via upgrade.

### 11.3 Déblocage des armes — accomplissements + rachat
Un accomplissement rend l'outil **récupérable** ; on le **rachète** ensuite au bousier (cap **2 outils/run**).

| Outil | Accomplissement (condition codée) | Prix |
|-------|-----------------------------------|------|
| Poings | Déjà débloqué (arme de départ) | 0 |
| Petite pelle | Battre ton **premier boss** | 60 |
| Arrosoir | **100 insectes** tués (cumulé) | 100 |
| Râteau | Battre **2 boss dans la même run** | 140 |
| Pelle | Battre les boss des **3 biomes** (cumulé) | 180 |
| Karcher | **Atteindre la Terrasse** | 220 |

### 11.4 Upgrades permanents *(achetés au bousier, en Pattes)*

| Upgrade | Effet par rang | Prix par rang |
|---------|----------------|---------------|
| PV max | +8 PV | 40 / 90 / 160 |
| Vitesse | +5 % vitesse max | 50 / 110 / 190 |
| Recharge dash | -10 % de cooldown | 80 / 160 |
| Gain de pattes | +15 % | 60 / 130 |

### 11.5 Sauvegarde
Fichier `save.ron` à la racine : pattes, kills cumulés, armes débloquées/récupérables, accomplissements, boss battus, rangs d'upgrade, terrasse débloquée, meilleur temps, runs, morts.

---

## 12. Contrôles *(implémentés)*

| Action | Touche |
|--------|--------|
| Déplacement | `ZQSD` / `WASD` / flèches |
| Visée | Souris |
| Arme 1 | Clic gauche |
| Arme 2 | Clic droit |
| Dash | `Espace` / `Shift` |
| Interagir | `E` |
| Pause | `Échap` |
| Choix (augment/biome) | `1` / `2` / `3` |
| Boutique / établi | `1`–`9`, `Échap` ferme |

---

## 13. Direction artistique & audio *(à faire)*

- **Actuel :** placeholders géométriques colorés, lisibles. Feedback de vitesse déjà en place : **teinte du perso qui chauffe** (vert → orange) + **traînée de fantômes** au-delà de 55 % de vitesse. Texte via police **DejaVu Sans embarquée** (accents + symboles).
- **Cible :** **pixel art** (sprite perso/armes séparés s'y prête). Audio à définir (musique montant avec l'intensité, SFX punchy dash/kills).

---

## 14. Architecture technique (Bevy 0.18) *(implémentée)*

12 modules Rust :

| Module | Rôle |
|--------|------|
| `common` | États (`AppState`, `RunPhase`), composants/messages partagés, dégâts, poison, pattes, mort, excuses, nettoyage. |
| `player` | Stats effectives, déplacement (inertie), dash, vitesse→dégâts, feedback visuel. |
| `weapons` | 6 armes, 2 slots, visée, poison/flaques, sprites d'armes séparés. |
| `enemies` | Bestiaire (8 types), IA (poursuite/charge/distance), projectiles, flaques dangereuses. |
| `boss` | 3 boss à machines à états (patterns). |
| `biomes` | 3 biomes : palette, bestiaire par tier, boss, règle de choix. |
| `rooms` | Boucle de run : construction de salles, gauntlet, portes, trésor, choix de biome. |
| `augments` | 18 augments, tirage 3→1, UI de choix. |
| `meta` | Sauvegarde RON, accomplissements, déblocages. |
| `cabanon` | Hub, bousier, boutique, établi, portes. |
| `terrasse` | Mode survie. |
| `ui` | HUD, pause, game over, toasts. |

- **États** : `Cabanon · EnRun · Terrasse · GameOver`, et sous-phases de run `Fighting · DoorOpen · Augment · BiomeChoice`.
- **Ordonnancement** : sets `Input → Ai → Move → Combat → Post`.
- **Collisions** : tout en cercles (rayon), simple et suffisant.

---

## 15. Lancement & build

- **Jeu (Windows natif, GPU réel)** : `C:\GameForge2026\Jouer.bat` ou `target\release\GameForge2026.exe`.
- **Dev (WSL)** : édition du code sous `~/GameForge2026`, build `cargo build --release`. Le rendu WSLg n'a pas de GPU exploitable → on rejoue côté Windows.
- Le projet existe en deux endroits : source de référence (WSL) + copie compilable Windows (`C:\GameForge2026`).

---

## 16. Roadmap restante

- [x] M0-M6 : socle, hook, armes, salles, run complète, méta, terrasse.
- [ ] **Audio** (musique + SFX).
- [ ] **Pixel art** (remplacer les placeholders).
- [ ] **Équilibrage** approfondi (passe complète une fois le feel validé).
- [ ] Contenu : 4e+ biome, plus d'armes/augments, salles secrètes.
- [ ] Manette, leaderboard.

---

## 17. Partis pris à valider *(décisions prises pendant le dev, à confirmer)*

Ces choix ont été faits pour rendre le jeu jouable ; ils ne sont pas gravés :

- ✅ **Modèle de dégâts flat** (`vitesse/100`, plancher ×0.4) — *validé.*
- ✅ **Identités des boss** : Mémé Mygale (araignée) / Roger le Scorpion / Grompaud (crapaud, réf. Gromp de LoL) — *validé.*
- ✅ **Mapping des déblocages d'armes** (§11.3) — *validé.*
- ✅ **Upgrades permanents** : on garde les 4 axes (PV/vitesse/dash/pattes) — *validé, le +PV reste modéré.*

Restent ouverts :
1. **Probabilités de salles** (70/15/15) et **1-3 salles par biome**.
2. **Terrasse en accès direct** : repart « à nu » (sans le build de run).
3. **PV de départ 50**, i-frames 0.6 s après un coup.
4. **Noms des petits ennemis** et d'éventuels exploits absurdes pour les déblocages (lore §2.1).

---

*Document maintenu au fil du projet. Les sections « à valider » (§17) attendent tes arbitrages — on les tranche au fur et à mesure.*
