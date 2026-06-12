# GameForge2026 — Game Design Document (GDD)

> **Version :** v0.1 (document vivant)
> **Statut :** vision + specs de base. Tout ce qui est marqué *(à valider)* est une proposition par défaut, pas une décision figée.
> **Genre :** Roguelite d'action top-down, visée manuelle, sur le thème de la **vitesse**.
> **Tech :** Rust + [Bevy](https://bevyengine.org/) `0.18` (voir [README](../README.md)).

---

## 1. Vision & piliers de design

**Pitch d'une ligne :** un jardinier rétréci traverse son jardin devenu hostile, où **plus il va vite, plus il frappe fort**, pour récupérer ses outils volés et rejoindre sa terrasse.

### Les piliers (ce qui ne doit jamais être trahi)

1. **La vitesse, c'est l'arme.** Le mouvement n'est pas un confort, c'est la source des dégâts. Le jeu doit toujours récompenser le joueur qui reste en mouvement et le pousser à prendre des risques pour aller plus vite.
2. **Glass cannon nerveux.** PV faibles, esquive reine. La tension vient de la fragilité : on survit par le skill de placement, pas par le tank.
3. **Runs courtes, builds variés.** ~12-15 min par run, des augments tranchants et peu nombreux, un parcours qui change à chaque partie → on relance « encore un run ».
4. **Synergies > stats brutes.** La rejouabilité naît des combinaisons (armes × augments), pas de l'empilement de chiffres. Chaque run raconte un build.
5. **Comique & cosy.** Ton léger, running gags, un jardin attachant. On sourit autant qu'on transpire.

---

## 2. Lore, ton & boucle narrative

### 2.1 Le déclencheur (running gag)

Un jour, dans son **cabanon**, le jardinier range ses outils. Épuisé, il se dit qu'une petite sieste — même dans un coin aussi peu confortable — ne serait pas de refus. Il se réveille… **minuscule**. Aucune explication. Le jeu n'en donnera jamais vraiment, et c'est la blague.

Pire : un **bousier farceur, malicieux et un brin malsain** a profité de son sommeil pour lui **voler ses outils** (eux aussi miniaturisés). Il refuse de les rendre — sauf **contre des pattes d'insectes**, sa monnaie à lui : il les amasse pour rouler une énorme **boule de pattes** censée **intimider tout le jardin**. Le jardinier les gagne donc en **dézinguant des insectes** (et en réalisant des **exploits ridicules**, ex. « bats le boss 1 avec une combinaison d'armes débiles »).

> Le bousier est donc à la fois le **comic relief** et le **moteur de la méta-progression** (voir §11).

### 2.2 Le ton

- **Comique / cosy.** Humour absurde, autodérision, petites bestioles attachantes même quand elles essaient de te tuer.
- **Running gag du rétrécissement** : jamais expliqué, souvent évoqué de travers.

### 2.3 La boucle de la mort (diégétique)

Quand le joueur meurt, il **se réveille dans son cabanon** (lui aussi rétréci). Chaque mort est « justifiée » par une **excuse bidon générée au hasard**, toujours différente :
- *« Un ruissellement d'eau de pluie t'a charrié jusqu'au cabanon. »*
- *« Un insecte super sympa t'a ramené sur son dos. »*
- *« Tu t'es réveillé. C'était un rêve. Enfin… presque. »*

→ Ces excuses sont une **liste de strings tirée aléatoirement** : peu coûteux à produire, gros rendement comique. *(À étoffer en continu.)*

### 2.4 L'objectif & le twist

Le jardinier veut **rejoindre sa terrasse**, sa supposée zone de sécurité. **Plot twist :** la terrasse n'est pas un refuge — c'est l'épreuve ultime, le **mode survie chronométré sans fin** (voir §10).

---

## 3. Game feel & mécaniques cœur

### 3.1 Vitesse instantanée → dégâts *(mécanique signature)*

Les dégâts dépendent de la **vélocité réelle du personnage à l'instant T**. Immobile = dégâts planchers ; à pleine vitesse = dégâts max.

**Formule proposée *(à valider / équilibrer)* :**

```
mult_vitesse = lerp(MULT_MIN, MULT_MAX, vitesse_actuelle / vitesse_max_courante)
dégâts = dégâts_base_arme × mult_vitesse × autres_multiplicateurs(augments…)
```

- `MULT_MIN ≈ 0.5` (immobile → on tape mou, c'est punitif et voulu)
- `MULT_MAX ≈ 2.5` (pleine vitesse → pic de dégâts) *(valeurs de départ, à tuner)*
- `vitesse_actuelle` = norme du vecteur vélocité du joueur ce frame.
- `vitesse_max_courante` = vitesse de pointe atteignable avec les stats/augments du moment (donc le ratio reste lisible même quand on build de la vitesse).

**Conséquences de design :**
- Rester planté = se saborder. Le kiting devient le mode de jeu naturel.
- Les augments de **vitesse de déplacement** sont aussi des augments de **dégâts** → double valeur, builds « vitesse » très désirables.
- Lisibilité : un **feedback visuel** d'intensité (aura/traînée/teinte d'arme qui s'intensifie avec la vitesse) est **indispensable** pour que le joueur ressente sa puissance. *(À designer.)*

### 3.2 Mobilité & dash

- **Mobilité au cœur du jeu** : perso nerveux, accélérations franches.
- **Dash** : esquive unique avec **i-frames** brèves + **cooldown**. *(Durée i-frames ~0,2 s, CD ~1-1,5 s — à valider.)*
- **On ne peut PAS attaquer pendant le dash** : l'esquive est un choix défensif qui coûte du DPS (et, le temps du dash, on n'exploite pas le pic de vitesse pour frapper). Décision risque/récompense à chaque dash.
- **Débloqué par augment — « dash offensif »** : un augment ouvre une **fenêtre de burst** en sortie de dash (ou autorise l'attaque pendant). Transforme le dash défensif en outil agressif → un axe de build à part entière.

### 3.3 Survie (glass cannon)

- **PV de départ : 50.** Assez pour offrir une **marge d'erreur douce en début de run** et laisser les mobs **scaler leurs dégâts** au fil de la partie — la marge se resserre, l'esprit glass cannon revient quand les ennemis montent en puissance.
- On survit par le **mouvement, l'esquive et le contrôle de l'espace**, pas par l'absorption.
- *(À valider :* soin entre salles ou non, i-frames après un coup reçu, courbe de scaling des dégâts ennemis.*)*

### 3.4 Visée & contrôle

- **Tout en visée manuelle**, pas d'auto-aim. Le joueur vise et déclenche **chaque** arme lui-même → skill intégral.
- *(Point réconcilié :* l'arrosoir « automatique » devient un **maintien** : on tient la gâchette, il pulvérise et **pose la traînée de pesticide le long du trajet**. La visée reste manuelle, c'est l'effet qui est continu.*)*

---

## 4. Armes

### 4.1 Système d'armes

- Le perso porte **jusqu'à 2 armes** simultanément.
- **Sprite du perso séparé du sprite des armes** → on combine librement n'importe quelle paire et on les anime indépendamment.
- **Tout en visée manuelle.** Chaque arme a son propre déclenchement (ex. clic gauche = arme 1, clic droit = arme 2).
- Les armes ont des **profils d'usage** différents :
  - **Frappe** (pression/clic = un coup visé) : poings, pelle…
  - **Maintien** (gâchette tenue = effet continu) : arrosoir, karcher…
  - **Utilitaire / contrôle** (souvent sur cooldown) : râteau (attire les ennemis)…

### 4.2 Roster (matériel de jardin)

| Arme | Profil | Mécanique | Statut |
|------|--------|-----------|--------|
| 👊 **Poings** | Frappe | Arme de départ, corps-à-corps rapide, courte portée. | Confirmé |
| 🌱 **Petite pelle** | Frappe | Mêlée rapide, faible portée, DPS régulier. | Confirmé |
| ⛏️ **Pelle** | Frappe | Mêlée lourde, plus lente, gros coup / knockback. | Confirmé |
| 🍴 **Râteau** | Utilitaire | **Attire les ennemis vers le joueur** (regroupement/CC). Pilier de synergie. | Confirmé |
| 💧 **Arrosoir** | Maintien | **Q de Singed** : pose une **traînée de pesticide** le long du trajet. Les ennemis dedans s'**empoisonnent** (DoT) ; **rester dans le poison reset son timer** → ils fondent s'ils stagnent. | Confirmé |
| 🔫 **Karcher** | Maintien | Jet haute pression à distance, **knockback** continu, DPS soutenu mais exige de tenir la ligne de mire. | Confirmé |
| ✂️ Sécateur | Frappe | Coups rapides, applique un saignement ? | Idée *(à valider)* |
| 🌿 Tuyau / lance à eau | Maintien | Poussée de zone, contrôle de foule. | Idée *(à valider)* |
| 🧂 Sel (anti-limace) | Frappe/lancer | Dégâts de zone, fort contre les ennemis « mous » (limaces). | Idée *(à valider)* |
| 🔍 Loupe | Maintien | Rayon de feu solaire — DPS canalisé précis. | Idée *(à valider)* |

> Les armes idées ne servent que de réservoir : à piocher/écarter au fil du dev.

### 4.3 Mécaniques-clés détaillées

- **Arrosoir (poison) :** la traînée a une durée de vie au sol ; chaque tick inflige des dégâts ; le **timer de poison d'un ennemi se rafraîchit tant qu'il touche du pesticide** (immobiliser/regrouper les ennemis dans le poison = kill garanti).
- **Râteau (attraction) :** tire les ennemis vers le joueur (ou vers un point). Combo canonique : **râteau → arrosoir** (on agglutine puis on noie le paquet dans le poison), le tout en mouvement (donc dégâts max).

---

## 5. Augments & synergies

### 5.1 Acquisition

- **Après chaque boss : 3 augments proposés → le joueur en choisit 1.** (1 choix garanti par biome, soit ~5 par run.)
- **Sources bonus :** salles **Trésor/bénédiction** et salles **secrètes** rares peuvent offrir un augment supplémentaire.
- Résultat : un build **resserré et lisible** (~5-7 augments/run), où chaque choix pèse. Colle au format « court & nerveux ».

### 5.2 Archétypes d'augments *(catégories proposées — à valider)*

| Catégorie | Exemples |
|-----------|----------|
| **Vitesse** | +vitesse de déplacement (donc +dégâts), accélération, vitesse de pointe accrue. |
| **Dash / burst** | dash offensif, 2e charge de dash, i-frames allongées, explosion en sortie de dash. |
| **Modificateurs d'arme** | poison plus virulent (arrosoir), râteau qui attire + ralentit, karcher qui ricoche… |
| **Momentum / on-the-move** | bonus qui montent tant qu'on ne s'arrête pas (régen, dégâts, cadence). |
| **Keystones / transformations** | augments rares qui changent une règle (ex. « le poison se propage entre ennemis », « le dash laisse une traînée de pesticide »). |
| **Défensif (mesuré)** | petits filets de sécurité, sans casser le glass cannon. |

### 5.3 Exemples de synergies (à cultiver dans le pool)

- **Râteau + Arrosoir** → on regroupe puis on empoisonne le paquet.
- **Dash offensif + vitesse** → hit-and-run dévastateur, on traverse, on burst, on ressort.
- **Momentum + poison qui se propage** → on ne s'arrête plus jamais, le jardin fond derrière soi.
- **Karcher + knockback** → on repousse les ennemis dans nos propres flaques de poison.

> Objectif : qu'un joueur veuille **rejouer** juste pour tester une combinaison entrevue.

---

## 6. Structure d'un run

### 6.1 Vue d'ensemble

```
RUN ( ~12-15 min )
└── 5 BIOMES enchaînés
    └── BIOME ( ~2-3 min )
        ├── 1 à 3 salles intermédiaires   (combat / spéciale)
        └── 1 salle de BOSS               (gauntlet 3 vagues, ~45 s-1 min)
            └── récompense : 3 augments → choix 1
            └── choix du biome suivant : 2 options (≠ biome courant, pas 2× le même d'affilée)
→ après les 5 biomes : TERRASSE (mode survie / score, voir §10)
→ à la mort : retour au cabanon (excuse bidon aléatoire)
```

### 6.2 Le biome

- **1 à 3 salles intermédiaires** *(à confirmer : parfois 0 ?)* peuplées de mobs variés, puis **la salle de boss**.
- Certaines salles intermédiaires peuvent être des **salles spéciales** (voir §6.4).

### 6.3 La salle de boss (gauntlet à 3 vagues)

| Vague | Contenu |
|-------|---------|
| **Vague 1** | Petits sbires **thématisés boss** (nuée, faibles, nombreux). |
| **Vague 2** | Sbires **moyens** thématisés boss. |
| **Vague 3** | **Gros mobs**, puis le **BOSS** lui-même (engagé une fois les gros mobs nettoyés). |

→ Montée en tension intégrée à chaque fin de biome ; le bestiaire de la vague « prépare » l'esthétique du boss.

### 6.4 Salles spéciales

| Salle | Rôle |
|-------|------|
| **Élite** | Mini-boss / ennemi renforcé optionnel, **grosse récompense**. Pic de difficulté juteux. |
| **Trésor / bénédiction** | Salle calme, cadeau au choix (augment, arme, soin). Respiration. |
| **Secrète** *(plus tard)* | Rare, cachée, récompense bonus (dont augments). |

### 6.5 Branches & rejouabilité

- **Choix de biome après chaque boss** : on propose **2 biomes**, jamais le biome courant. Un biome **peut revenir dans une run, mais jamais deux fois d'affilée**.
- **Règle de sélection :**
  - **Pool à 3 (actuel)** : les 2 options sont **forcément les 2 autres** biomes (déterministe).
  - **Pool élargi (plus tard)** : les 2 options sont **tirées au hasard** dans le pool, en **excluant le biome courant**.
- La structure reste **essentiellement linéaire** (facile à générer/équilibrer) tout en offrant de la variété → **builds rapides et divers** sans carte complexe à produire.

---

## 7. Biomes (le jardin)

**Pool de départ : 3 biomes** (on étendra plus tard). Chaque biome a son **sous-bestiaire** et son **boss** dédié (dont les sbires du gauntlet sont thématisés).

| Biome | Ambiance / DA | Bestiaire | Boss pressenti |
|-------|---------------|-----------|----------------|
| **Plaine** *(biome de base)* | Herbe rase, espace ouvert, lumineux. Point de départ du run. | Insectes « tout-venant », équilibrés. | *(à définir)* |
| **Savane** | Herbes hautes et sèches, chaud, terreux. | **Scarabées** et insectes du même type (carapaces, tanks). | *(à définir)* |
| **Jungle** | DA **mousseuse et humide**, végétation dense, sombre, gouttes. | *(à définir — bestioles d'humidité)* | *(à définir)* |

> **Avantage du pool à 3 :** le **choix de biome après chaque boss** (« 2 options ≠ biome courant ») tombe pile — depuis n'importe quel biome, les **2 autres** sont proposés. Mécanique limpide.
>
> **Run = 5 biomes, répétitions assumées.** Avec 3 biomes au pool, une run en répète forcément certains (jamais deux fois d'affilée) — c'est **voulu**. Règle de sélection détaillée en §6.5. *(Reste à tuner : un biome/boss revu dans la même run monte-t-il en difficulté ?)*

---

## 8. Bestiaire

**Thème : faune réelle du jardin, à l'échelle mini.** Cohérent, crédible, tailles très variées. *(Pas d'objets hantés.)*

### 8.1 Archétypes (par rôle de gameplay)

| Archétype | Exemples | Comportement |
|-----------|----------|--------------|
| **Nuée / swarm** | pucerons, fourmis | Faibles, très nombreux, submergent. Idéaux pour le poison/AoE. |
| **Chasseur rapide** | araignée, scarabée | Foncent sur le joueur, punissent l'immobilité. |
| **Distance** | guêpe, moustique | Tirent/piquent à distance, forcent le déplacement. |
| **Tank** | scarabée-rhino, escargot | Lents, encaissent, contrôlent l'espace. |
| **Mou / à statut** | limace, ver | Faibles aux dégâts de zone (sel !), lents. |
| **Spécial** | chenille (segments), ver de terre (sous le sol) | Patterns particuliers. |

### 8.2 Boss

Prédateurs/animaux du jardin (taupe, crapaud, oiseau, frelon, araignée…). Chaque boss = **3 vagues** + un **pattern propre**. *(Movesets à designer par boss.)*

### 8.3 PNJ

- **Le bousier** (PNJ du cabanon) : farceur, malicieux et un brin malsain. Vendeur de la méta-progression — il garde tes outils en otage et ne les rend que contre des **Pattes**, qu'il amasse pour rouler sa **boule de pattes** censée intimider tout le jardin.

---

## 9. *(réservé — équilibrage / courbes de difficulté)*

> À remplir : scaling des PV/dégâts ennemis par biome et par profondeur de run, densité de spawn, éventuel système d'ascension/difficulté optionnelle (façon « heat »).

---

## 10. Mode Terrasse (la finale)

- Atteint après avoir nettoyé les **5 biomes** d'un run.
- **Survie infinie / score** : des ennemis affluent **en continu** et **montent en puissance**. Ça ne se « gagne » pas — on tient **le plus longtemps possible**, le run s'achève quand on tombe.
- **Score & leaderboard** *(local d'abord)* : temps tenu, kills, multiplicateur de vitesse moyen… *(métriques à définir.)*
- C'est le **défi de maîtrise ultime** : tout le build de la run y est mis à l'épreuve.
- *(À valider :* le mode Terrasse est-il rejoué via le run normal à chaque fois, ou aussi accessible en accès direct une fois atteint ?*)*

---

## 11. Méta-progression & économie

### 11.1 Le hub — le cabanon

- Petit lieu **à l'échelle mini** (rétréci avec le héros), point de départ et de retour de chaque run.
- Abrite le **bousier** (vendeur). S'enrichit visuellement avec la progression. *(Hub léger, pas de gros contenu narratif pour l'instant.)*

### 11.2 La monnaie — les Pattes

**Une seule monnaie** (plus de boutique en run → plus de monnaie de run).

| Monnaie | Gagnée par | Dépensée pour | Persiste ? |
|---------|-----------|---------------|------------|
| **Pattes** (pattes d'insectes) | tuer des insectes, boss, exploits | déblocages permanents auprès du bousier, au cabanon | ✅ conservée entre les runs |

Thématiquement parfait : on **arrache les pattes** des insectes tués, et le bousier en raffole pour sa **boule de pattes**.

### 11.3 Récupérer ses outils (déblocage des armes)

Modèle **« accomplissements + achats »**, incarné par le bousier :

1. **Un accomplissement** rend un outil **récupérable** (ex. *arme 1 = battre le boss 1*, *arme N = battre le boss N*, ou un **exploit marrant** comme « battre le boss 1 avec une combo d'armes débiles »).
2. Le joueur **paie le bousier** (en **Pattes**) pour le **récupérer** définitivement (il entre alors dans le pool jouable).
3. **Throttle de contenu — max 2 outils récupérés par run.** Si le joueur remplit d'un coup les conditions de 3 outils, il n'en récupère que **2** ; le reste est reporté aux runs suivants → on évite de noyer le joueur sous le contenu neuf.

### 11.4 Upgrades permanents

- Également **achetés au bousier** (en **Pattes**) : petits bonus permanents *(à valider — rester modéré pour préserver le glass cannon : ne pas transformer le héros en char).*
- *(À valider :* déblocage de nouveaux **augments** dans le pool, de **nouvelles salles**, du **mode Terrasse**, via le même système.*)*

### 11.5 Exploits / accomplissements

- Liste d'objectifs **variés et souvent absurdes** qui débloquent du contenu et/ou donnent des **Pattes**. Double fonction : **récompense l'expérimentation** (tester armes & builds) et **nourrit l'humour**.

---

## 12. Contrôles *(proposé — à valider)*

**Clavier + souris** (cohérent avec la visée manuelle) :

| Action | Touche |
|--------|--------|
| Déplacement | `ZQSD` / `WASD` |
| Visée | Souris |
| Arme 1 | Clic gauche |
| Arme 2 | Clic droit |
| Dash | `Espace` / `Shift` |
| Interagir | `E` |
| Pause | `Échap` |

- **Manette** : support souhaitable plus tard (stick gauche = déplacement, stick droit = visée, gâchettes = armes). *(Non prioritaire.)*

---

## 13. Direction artistique & audio

- **Style : pixel art.** Lisible, cohérent avec le genre, raisonnable à produire en solo, gros catalogue de références. Le **sprite perso/armes séparés** s'y prête naturellement.
- **Feedback de vitesse** : la DA doit rendre la vitesse **lisible** (traînée, aura, étirement, particules qui s'intensifient avec la vélocité) — c'est central, pas cosmétique.
- **UI** : minimaliste, lisible en plein chaos (PV, jauge de dash/cooldowns, multiplicateur de vitesse, monnaie).
- **Audio** *(à définir)* : musique qui monte avec l'intensité ? SFX punchy pour le dash et les kills. Ton léger.

---

## 14. Architecture technique (Bevy) *(esquisse — à valider)*

Découpage ECS pressenti, à raffiner au fil du code :

- **States (`States`)** : `Cabanon` (hub) · `EnRun` · `Terrasse` · `Pause` · `GameOver`.
- **Plugins** : `PlayerPlugin`, `WeaponsPlugin`, `EnemiesPlugin`, `RoomFlowPlugin`, `AugmentsPlugin`, `MetaProgressionPlugin`, `UiPlugin`, `AudioPlugin`.
- **Composants clés** : `Velocity`, `Speed { current, max }`, `Health`, `DashState`, `Weapon`, `WeaponSlot`, `PoisonTrail`, `Poisoned { timer }`, `EnemyKind`, `RoomState`, `WaveState`.
- **Systèmes signatures** :
  - `speed_to_damage` : lit `Velocity`/`Speed`, calcule `mult_vitesse`, l'applique aux dégâts.
  - `dash_system` : i-frames, cooldown, blocage des attaques.
  - `pesticide_trail` : émission de la traînée (arrosoir) + DoT + reset de timer au contact.
  - `rake_attract` : force d'attraction du râteau.
  - `room_flow` : enchaînement salles → vagues de boss → récompense → choix de biome.
- **Données externalisées** (RON/assets) : stats d'armes, table d'augments, définitions d'ennemis/boss, biomes, excuses de mort, conditions d'accomplissement → **data-driven** pour itérer sans recompiler.

---

## 15. Roadmap de production *(jalons)*

**Vertical slice d'abord** : une boucle jouable minimale, puis on étoffe.

- [ ] **M0 — Socle** : fenêtre Bevy, perso top-down déplaçable, dash.
- [ ] **M1 — Le hook** : `vitesse → dégâts` + feedback visuel ; poings ; ennemi « chasseur » basique dans une salle.
- [ ] **M2 — Armes** : 2 slots, visée manuelle ; arrosoir (poison/traînée) + râteau (attraction) → première synergie jouable.
- [ ] **M3 — Boucle de salle** : salles intermédiaires + salle de boss à 3 vagues ; 1 boss complet.
- [ ] **M4 — Run complète** : enchaînement de biomes + choix de biome ; augments (3→1 après boss) ; les 3 biomes.
- [ ] **M5 — Méta** : cabanon + bousier ; monnaie unique (Pattes) ; déblocage d'armes (accomplissements + achat, cap 2/run) ; excuses de mort.
- [ ] **M6 — Terrasse** : mode survie/score.
- [ ] **M7 — Contenu & équilibrage** : étendre armes/augments/biomes/bestiaire ; tuning.

---

## 16. Questions ouvertes / à trancher

- **Nom définitif** du jeu (*GameForge2026* = nom de projet).
- **Formule `vitesse → dégâts`** : valider `MULT_MIN`/`MULT_MAX` et la courbe (linéaire ? avec palier ?).
- **Survie** : soin entre salles ou non, i-frames après un coup reçu, courbe de scaling des dégâts ennemis (sur la base des 50 PV de départ).
- **Salles intermédiaires** : 0-3 ou 1-3 par biome ? densité.
- **Scaling des répétitions** : un biome/boss revu dans la même run doit-il monter en difficulté ? (run = 5 biomes, répétitions assumées.)
- **Économie des Pattes** : prix des outils/upgrades, courbe de gain.
- **Upgrades permanents** : lesquels, jusqu'où, sans casser le glass cannon.
- **Mode Terrasse** : accès direct une fois atteint ? métriques de score.
- **Boss des 3 biomes** (Plaine / Savane / Jungle) : identités + movesets.
- **Manette** : quand l'ajouter.

---

*Document maintenu au fil du projet. Toute section « à valider » attend ta décision — on les tranchera au fur et à mesure du dev.*
