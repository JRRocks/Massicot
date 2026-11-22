# Brief pour une session cloud : P3.3, zoom des pages (Ctrl + molette)

> Document de travail : la session qui livre P3.3 le supprime dans son dernier commit.

## Contexte

L'utilisateur ne lit pas le code : il juge sur ce qu'il voit. Il a demandé Ctrl + molette pour
zoomer la grille des pages, « juste retirer ou ajouter 1 feuille en plus dans le visuel, pas de
zoom à mi-chemin ».

Un aperçu HTML a été validé le 2026-11-22 (décision D-31, tâche P3.3), avec deux choix :
- le zoom agit aussi dans le rapport à blanc ;
- il est gardé d'une ouverture à l'autre.

La session locale, sur son PC et sans chaîne Rust, installera la Release publiée par la CI et
essaiera le zoom en vrai. Toi, tu codes, tu testes et tu publies.

Pars de `main` (commit `7c16201` ou plus récent). Lis d'abord `CLAUDE.md`, puis dans `docs/` :
`PLAN.md` (P3.3), `DECISIONS.md` (D-31, et D-26 à D-29 pour la fenêtre), et l'aperçu validé
`previews/zoom-des-pages.html`. Cet aperçu est le contrat : suis-le fidèlement.

## But

Dans la fenêtre (`crates/massicot/src/fenetre/`), Ctrl + molette zoome les pages : une page de
plus ou de moins par ligne à chaque cran.

## Existant

- `fenetre/mod.rs`, `fn pages()` : grille `horizontal_wrapped` de vignettes à largeur fixe
  `LARGEUR_VIGNETTE = 96.0`, écarts 16 × 14, marge 16. La vue en blocs du rapport (étapes 3 et 4)
  a des pages de 62 pt, et ses blocs ne sont jamais coupés (`largeur_bloc`).
- `fn vignette()` dessine une page : bande jaune de l'étape 2 en %, numéro en 10 pt sous 80 pt de
  large, 11,5 pt au-delà.
- `fenetre/vignettes.rs` rend les pages à 192 px de large (`LARGEUR`).

## À faire (D-31)

1. **Grille.** N pages par ligne (un entier). Largeur d'une page =
   (largeur disponible − (N − 1) × 16) / N : les pages remplissent toute la ligne. Mets en page par
   rangées explicites ; `horizontal_wrapped` passerait à la ligne sur un arrondi.
2. **Nombre par ligne au départ.** Avant tout zoom, N suit la largeur comme aujourd'hui :
   `max(1, floor((L + 16) / (96 + 16)))`. Dès le premier cran, N est fixé et ne change plus avec
   la taille de la fenêtre. Si la fenêtre devient trop étroite pour ce N (pages de moins de
   48 pt), réduis N à l'affichage sans changer la valeur gardée.
3. **Le geste.** Ctrl + molette, pointeur au-dessus de la zone des pages : vers le haut N − 1
   (minimum 1), vers le bas N + 1 (maximum : pages d'au moins 48 pt,
   `floor((L + 16) / (48 + 16))`).
   - Un cran = un pas, appliqué d'un coup, sans animation ni taille intermédiaire. Cumule les
     petits deltas (molette fine, pavé tactile) jusqu'à l'équivalent d'un cran.
   - Ctrl + molette ne doit ni faire défiler la zone, ni changer le zoom global d'egui. Vérifie
     comment egui 0.37 traite Ctrl + molette (`zoom_delta`, défilement).
   - La molette seule défile comme avant ; Ctrl + clic garde la multi-sélection.
4. **Ancrage.** La page sous la souris reste sous la souris après le pas : ajuste le défilement de
   la ScrollArea.
5. **Rapport à blanc** (vue en blocs). Le même geste agit. Largeur d'une page de bloc = largeur
   d'une page de la grille × 62 / 96, soit 62 pt comme aujourd'hui avant tout zoom. Un bloc plus
   large que la zone passe à la ligne dans son cadre au lieu de déborder.
6. **Pages nettes.** Quand une page s'affiche plus grande que sa texture (en pixels physiques),
   redessine-la plus grande en arrière-plan.
   - Procède par paliers, avec un plafond vers 2048 px de large, les pages visibles d'abord.
   - Garde l'ancienne texture jusqu'à ce que la nouvelle soit prête.
   - Plafonne la mémoire : les pages hors écran redescendent.
7. **Mémoire.** Le N fixé est gardé d'une ouverture à l'autre (stockage d'eframe, ou réglages de
   l'application s'il y en a).
8. **Rien d'autre de visible** : ni bouton, ni texte, ni indicateur. Si une contrainte technique
   impose un écart avec l'aperçu, ne le code pas : arrête-toi et décris l'écart dans le journal et
   dans ton dernier message.

## Vérifier

- Tests unitaires du calcul : N par défaut, largeur, bornes, cumul de la molette, ancrage.
- Tests d'écran egui_kittest :
  - grille à l'ouverture (les images de référence existantes changent puisque la ligne se
    remplit : regarde-les avant de les accepter) ;
  - grille à 8 et à 3 par ligne ;
  - rapport zoomé.
- `cargo test --workspace` vert.

## Livrer

- **Commits** en français, identité `JRRocks <314636139+JRRocks@users.noreply.github.com>`.
- **Dépôt public** : contrôle « rien de privé » avant chaque push, ni vrai PDF ni capture réelle.
- **Publication** : push sur `main`, sans pull request. Attends que la CI publie la Release
  (installateur `Massicot.<version>.exe`) et note son numéro.
- **Mémoire du projet** :
  - `docs/PLAN.md` : P3.3 passe en 🖥️, et une ligne R6 « à vérifier sur le PC » dit ce que
    l'utilisateur fait et ce qu'il doit voir ;
  - `docs/JOURNAL.md` : une entrée pour cette session ;
  - `docs/DECISIONS.md` : tes choix techniques.
- **Hors périmètre** : ne touche pas aux moteurs copiés (`crates/` hors `massicot` et `essais`,
  `vendor/`).
- **Fin** : supprime ce brief. Ton dernier message donne la version publiée et ce qui reste à
  vérifier sur le PC.
