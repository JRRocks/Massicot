# Brief de la session de développement P1.1 à P2.5

> Document de travail : à supprimer une fois P2.5 terminé (le journal garde la trace).

## Contexte

L'utilisateur veut confier à Claude des PDF à traiter avec un outil PDF de son PC Windows. Exemple :
50 pages scannées de plusieurs procès-verbaux (PV) ; l'outil produit un PDF par PV en coupant à
chaque en-tête de PV, détecté par une regex, sans LLM. Les essais sur de vrais PDF viendront plus tard.

PdfCraft 0.21.1 (Rust, `storytold/pdfcraft`) est installé sur son PC ; il a été analysé et essayé
sur un PDF de test. `pdfcraft-cli` sait : info, text, edit (rotation), combine, extract, split
(toutes les N pages, avant des pages données, signets de niveau 1, taille), run (environ 300 outils
JSON), mcp, et l'OCR `ocrs` (anglais seulement). Il lui manque, par rapport à PDFsam : la découpe par
texte ou regex, le mélange alterné, la découpe par signets au-delà du niveau 1.

Décisions de l'utilisateur, détaillées dans `docs/DECISIONS.md` (D-01 à D-11) :
- pas de fork : un projet à part entière, « Massicot », fortement inspiré de PdfCraft, dont il reprend les moteurs ;
- les fonctions en plus s'inspirent de PDFsam Enhanced (plus tard : P4.1) ;
- trois interfaces : CLI, MCP, et une fenêtre Windows (après des maquettes validées) ;
- un dépôt public ;
- messages de l'outil et commits en français, code en anglais.

## Règles de l'utilisateur

- Il ne lit pas le code. Parle-lui peu, en français, et seulement pour trois choses : une question qui te bloque, un choix à valider, le résultat final.
- N'annonce jamais que ça marche sans l'avoir vu marcher (tests, CI). Un test instable se répare : on ne le relance pas jusqu'au vert.
- Chaque livraison porte un numéro de version nouveau. Si la compilation ou les tests échouent, rien n'est livré.
- Rien de visible sans aperçu validé.
- Tiens la mémoire à jour :
  - une entrée en haut de `docs/JOURNAL.md` ;
  - les statuts dans `docs/PLAN.md` ;
  - une ligne nouvelle dans `docs/DECISIONS.md` pour chaque décision, sans réécrire les anciennes ;
  - les idées pour plus tard dans `docs/idees-futur.md`.
- **Dépôt public : rien de privé.** Aucune adresse e-mail, aucun nom d'utilisateur ni chemin de poste (écris `C:\Users\<utilisateur>`), aucun vrai PDF de l'utilisateur (ils vivent dans `essais-prives/`, ignoré). Les commits portent l'identité noreply `JRRocks <314636139+JRRocks@users.noreply.github.com>` : vérifie-la avant le premier commit.
- Push direct sur `main`, sans pull request. Ne touche à aucun autre dépôt.

## Travail demandé

Tâches P1.1 à P2.5 de `docs/PLAN.md`, dans l'ordre, une à la fois. Chaque tâche se termine par des
tests verts, un commit poussé sur `main`, une CI verte et une Release publiée.

**P1.1 Squelette et CI.** Un workspace Cargo et un binaire `massicot`. GitHub Actions à chaque push sur `main` :
1. `cargo test --workspace` sous Linux ;
2. compilation Windows x86_64 (runner `windows-latest`, MSVC) ; sur ce runner, `massicot.exe --version` et une découpe d'essai sur un PDF généré ;
3. un zip `Massicot.<version>.zip` (exe, modèles OCR, `LICENSE-*`, `NOTICE`, `ATTRIBUTION.md`), publié en Release `v<version>`.

La version monte seule à chaque compilation (par exemple `0.1.<run_number>`), jamais deux fois le même numéro ; `--version` l'affiche.

**P1.2 Reprise des moteurs de PdfCraft** (`https://github.com/storytold/pdfcraft`, MIT OR Apache-2.0).
- Épingle le commit de `main` du jour et note-le.
- Copie dans `crates/` les seules crates nécessaires, d'après le vrai graphe (`cargo tree`).
- Évalue d'abord la reprise de `crates/automation` : elle porte la table d'outils JSON et le serveur MCP, et donnerait peut-être P2.5 presque gratuitement.
- Conserve les en-têtes de licence.
- Écris `ATTRIBUTION.md` : dépôt et commit source, liste des crates, matériel tiers repris de leurs `NOTICE`, `ATTRIBUTION.toml` et `LICENSES.toml`.
- Ne reprends ni `docs/brand`, ni les noms ArtCraft et PdfCraft dans ce que voit l'utilisateur.

Si l'ensemble minimal représente l'essentiel de PdfCraft (voir D-10), arrête-toi, note-le au journal et demande à l'utilisateur s'il préfère une dépendance git épinglée.

**P2.1 à P2.3 : les fonctions de PDFsam Basic en ligne de commande**, avec messages et aide en français accentué :
- infos, texte par page, fusion ;
- extraction, dans l'ordre donné ;
- découpe toutes les N pages, avant des pages données, par signets d'un niveau choisi, par taille ;
- rotation, y compris des seules pages paires ou impaires ;
- suppression, déplacement et inversion de pages ;
- mélange alterné de deux PDF (recto et verso scannés à part), le second fichier inversé en option ;
- OCR des pages scannées, avec ajout d'un texte invisible.

`ocrs` ne lit que l'anglais. Mesure ce qu'il fait d'en-têtes accentués scannés synthétiques (« PROCÈS-VERBAL N° 12 »). S'il les lit mal, ajoute Tesseract `fra` comme moteur optionnel, comme le fait PdfCraft. Le PC de l'utilisateur n'a pas Tesseract.

**P2.4 Découpe par regex**, le cœur du projet, sans aucun LLM. Chaque page qui porte un en-tête ouvre un bloc, qui court jusqu'au prochain en-tête ; un PDF par bloc.
- Texte de chaque page : la couche texte, sinon l'OCR (option toujours, auto ou jamais).
- Normalisation avant la regex : casse et accents ignorés par défaut, espaces réduits.
- Option pour ne chercher que dans le haut de la page (par exemple 30 %), d'après la position des mots : une simple mention « PV » dans le corps de la page ne doit pas couper.
- Plusieurs regex possibles.
- Pages avant le premier en-tête : un fichier à part par défaut, signalé, et une option pour un autre choix.
- Nom des fichiers : un modèle avec groupes de capture (par exemple `PV-{num}.pdf`), nettoyé ; les doublons sont suffixés -2, -3.
- Mode à blanc obligatoire avant de couper. Il produit un rapport JSON par bloc : pages, texte trouvé, extrait de la page, confiance de l'OCR, et alertes (confiance faible, page vide, nom en double). La vraie découpe écrit le même rapport à côté des fichiers.
- Tests sur des PDF générés : des PDF texte et des PDF image seule (pour passer par l'OCR), avec des PV inventés, du bruit et des accents.

**P2.5 Serveur MCP stdio**, par exemple `massicot mcp --racine DIR`. Il expose toutes les commandes, dont la découpe regex et son mode à blanc. Documente dans `README.md` sa configuration pour Claude Code sous Windows.

**Arrête-toi après P2.5.** Ne code pas la fenêtre : P3.1 demande d'abord des maquettes (plusieurs variantes) et le « oui » de l'utilisateur.

## Terminé quand

- P1.1 à P2.5 sont au statut ✅ ou 🖥️ dans le plan.
- La CI est verte et la dernière Release est téléchargeable.
- Le journal, le plan et les décisions sont à jour et poussés.
- Ce brief est supprimé.
- Un message final à l'utilisateur, en français et court, dit :
  - ce qui change pour lui ;
  - le lien de la Release ;
  - ce qui n'a pas pu être vérifié (vrai Windows, vrais scans) ;
  - ce que tu attends de lui.
