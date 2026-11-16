# Journal de bord

> Entrée la plus récente **en haut**. On ne modifie ni ne supprime une entrée passée :
> une erreur se corrige par une nouvelle entrée.

---

## 2026-11-16 · Premier lot réel : découpe par méthode LLM et par méthode déterministe

- **Branche et commit** : `main` (docs seulement ; aucun code de Massicot touché).
- **Objectif** : sur un vrai lot de PV (6 fichiers scannés, 264 pages, dans `essais-prives/`, hors
  dépôt), fusionner, couper un fichier par PV (page portant le tableau A/B/C du formulaire),
  nommer chaque fichier `PV` + les 7 chiffres du numéro d'enregistrement après l'année ; le faire
  par deux méthodes en parallèle pour juger si la méthode déterministe suffit.
- **Réalisé** :
  - Fusion par `massicot fusionner` (ordre des fichiers 1 à 6) : 264 pages.
  - Méthode LLM : trois agents lisent à l'œil le haut de chaque page (images rendues à 110 dpi),
    sans OCR ; découpe par `massicot extraire`.
  - Méthode déterministe : script PowerShell (hors dépôt, `essais-prives/_outil-deterministe/`)
    qui pilote `massicot.exe` : texte du haut de page (40 %) par la couche texte du PDF (263 pages
    sur 264 en ont une, posée par le scanner), puis OCR intégré imposé sur les pages candidates ;
    9 ancres tolérantes du tableau A/B/C ; début de PV si au moins 4 ancres dans au moins 2 cases ;
    numéro cherché près de « enregistrement » (année, séparateur éventuel, 7 chiffres).
- **Vérifications** :
  - Découpe : 56 blocs pour chaque méthode, **mêmes 56 coupures** ; 264 pages couvertes.
  - Noms : 48 sur 56 identiques. Les 8 écarts sont tous des « numéro introuvable » côté
    déterministe : 7 numéros manuscrits lus par la méthode LLM, et une case vide (vérifiée à l'œil :
    la méthode LLM a raison, « sans numéro »). Aucun numéro différent entre les deux méthodes là où
    les deux en lisent un.
  - Méthode déterministe : 37 lus, 10 sans numéro, 9 introuvables ; 40 s pour le lot ; deux
    exécutions, rapport identique.
  - Méthode LLM : 44 lus, 11 sans numéro, 1 illisible ; numéros manuscrits non revérifiés.
- **Non vérifié** : le jugement de l'utilisateur sur les deux dossiers de résultats.
- **Problèmes et dette** :
  - `decouper-regex --rapport` refuse d'écrire si le fichier de rapport existe déjà (il faut l'effacer
    avant chaque relance) : à corriger (accepter `--ecraser`).
  - Le formulaire existe en deux exemplaires dans certains PV (second exemplaire sans numéro, ou
    même numéro) : chaque tableau A/B/C ouvre un fichier, selon la règle donnée par l'utilisateur.
  - Les numéros manuscrits restent hors de portée de l'OCR intégré.
- **Prochaine étape** : l'utilisateur compare les dossiers « Résultats - méthode LLM » et
  « Résultats - méthode regex » ; selon son avis, intégrer la méthode déterministe à Massicot
  (voir `docs/idees-futur.md`).

---

## 2026-11-16 · Session PC : installation de la 0.1.9, essais R1 et R2

- **Branche et commit** : `main` (docs seulement ; aucun code touché).
- **Objectif** : installer la Release sur le PC, vérifier R1 et R2, préparer R3.
- **Réalisé** :
  - Zip `Massicot.0.1.9.zip` de la Release `v0.1.9` décompressé dans
    `%LOCALAPPDATA%\Programs\Massicot` (dossier fixe : une mise à jour se décompresse par-dessus).
  - R1 : `massicot.exe --version` → `massicot 0.1.9`. Pas d'avertissement SmartScreen vu
    (lancement depuis un terminal).
  - Essai rapide sur deux PDF inventés, fabriqués dans un dossier temporaire hors dépôt (Python
    absent du PC : fabriqués par `uv` avec reportlab et Pillow) : un PDF texte de 6 pages
    (3 en-têtes) et un scan d'images de 5 pages (2 en-têtes, léger bruit, 0,6° de travers).
    `infos` correct ; `decouper --toutes 2` → 3 fichiers ; `decouper-regex` à blanc sur le PDF
    texte → 3 blocs justes ; `texte` sur le scan (OCR intégré) lit l'en-tête avec « ? » à la
    place des accents, comme prévu ; `decouper-regex --zone-haute 20` à blanc sur le scan →
    2 blocs justes, alerte « ? » sur chaque en-tête ; découpe par jeton → 2 fichiers et
    `rapport-decoupe.json`.
  - Sans `--zone-haute`, un renvoi en bas de page (« procès-verbal n° 999 ») ouvre un faux bloc
    sur chaque page qui le porte (5 blocs au lieu de 2, noms suffixés -2, -3) : comportement
    attendu, la zone haute est indispensable sur les vrais lots.
  - `essais-prives/` créé, ignoré par git (`git check-ignore` : règle `.gitignore` ligne 6).
  - R2 : serveur MCP essayé à la main (initialisation, liste des 14 outils, appel `infos` sans
    erreur), puis, avec l'accord de l'utilisateur, `claude mcp add --scope user massicot` avec la
    racine `essais-prives/` ; `claude mcp list` → `massicot … ✔ Connected`.
  - Tesseract absent du PC ; non installé (à proposer seulement si l'OCR intégré déçoit sur les
    vrais scans).
- **Vérifications** : sorties des commandes lues en session (ci-dessus).
- **Non vérifié** : Claude qui décrit un PDF par le MCP (le serveur ne paraît que dans une
  session ouverte après l'ajout) ; R3 (vrais scans).
- **Anomalies** : la commande `claude` n'est pas dans le PATH de ce PC (Claude Code y vient de
  l'application de bureau, qui garde son exécutable dans son propre dossier de données) :
  la commande `claude mcp add` du README ne marche pas telle quelle dans PowerShell. Le README
  pourrait le dire, ou donner l'équivalent `.mcp.json` / `~/.claude.json` en premier.
- **Prochaine étape** : R3 puis P5.1 sur le premier vrai lot déposé dans `essais-prives/`.

---

## 2026-11-16 · Session cloud P1.1 à P2.5 : squelette, moteurs, commandes PDF, découpe par en-tête, MCP

- **Branche et commit** : `main` (push direct, D-04 et brief).
- **Objectif** : tâches P1.1 à P2.5 du plan, d'après `docs/BRIEF-P1-P2.md`.
- **Réalisé** :
  - P1.1 : workspace Cargo, exécutable `massicot` (aide et erreurs en français, `--version`),
    CI GitHub Actions (tests Linux, compilation Windows MSVC, essai de l'exécutable, zip
    `Massicot.<version>.zip` avec les modèles OCR, Release `v<version>`), version `0.1.<n° de run>`.
  - P1.1 vérifiée : CI verte (run 1), Release `v0.1.1` publiée ; zip téléchargé et ouvert en
    session (exe, modèles, licences).
  - P1.2 : moteurs repris de PdfCraft `412e696` (7 crates) et de craft-libs `8e03cd0` (2 crates),
    plus 9 crates tierces corrigées par PdfCraft (`vendor/`), par `scripts/importer-moteurs.py` ;
    versions des dépendances alignées sur le `Cargo.lock` de PdfCraft ; `ATTRIBUTION.md`, `NOTICE`,
    dossier `licences/` du zip, liste des crates compilées (`scripts/dependances.py`) ;
    commande `massicot infos` ; générateur de PDF d'essai inventés (`crates/essais`).
  - Évaluation de `crates/automation` (outils JSON + serveur MCP de PdfCraft) : elle exige
    `pdfcraft-engine`, donc 27 crates de PdfCraft et 18 de craft-libs, soit l'essentiel de PdfCraft
    hors interface ; ses outils sont ceux de PdfCraft, pas ceux de Massicot. Non reprise (D-14) :
    l'ensemble minimal copié (sans elle) fait environ 12 % du code de PdfCraft, donc pas d'arrêt
    au titre de D-10 ; le serveur MCP sera écrit pour Massicot (P2.5).
  - P1.2 vérifiée : CI verte (run 2, compilation Windows en 4 min, `massicot.exe infos` essayé
    sur le runner sur un PDF généré), Release `v0.1.2`.
  - P2.1 : commandes `fusionner` (un signet par fichier, ou `--sans-signets`), `extraire` (ordre
    donné), `decouper` (`--toutes N`, `--avant`, `--signets NIVEAU`, `--taille`), `tourner`
    (`--paires`, `--impaires`, `--pages`), `supprimer`, `deplacer` (`--vers debut|fin|N`),
    `inverser` ; jamais d'écrasement sans `--ecraser` ; noms de fichiers sûrs pour Windows ;
    essai de découpe ajouté sur le runner Windows.
  - P2.1 vérifiée : CI verte (run 3, découpe d'essai réussie sur le runner Windows), Release `v0.1.3`.
  - P2.2 : commande `melanger` (recto et verso scannés à part ; `--verso-inverse` ; pas réglables
    `--pas-recto`, `--pas-verso` ; le reste du plus long fichier suit).
  - P2.2 vérifiée : CI verte (run 4), Release `v0.1.4`.
  - P2.3 : commandes `texte` (couche texte, sinon OCR ; `--ocr auto|toujours|jamais`) et `ocr`
    (texte invisible ajouté aux pages sans texte, comme le fait PdfCraft : Helvetica étirée sur
    chaque mot, bloc marqué `/OCR`). Moteurs : intégré (`ocrs`, modèles à côté de l'exe) ou
    Tesseract en français s'il est installé (`--moteur auto|integre|tesseract`).
  - Mesure de l'OCR intégré sur des en-têtes scannés inventés (`examples/mesure_ocr.rs`) : il lit
    les lettres accentuées et « ° » comme « ? » (« PROC?S-VERBAL N? 12 », « D?lib?ration n? 4 »),
    mais résiste au bruit. Tesseract 5.3 `fra` (installé dans la session) lit tout juste à
    200 dpi propre (« PROCÈS-VERBAL N° 12 »), mais perd les en-têtes sur un scan très bruité et
    penché. D'où Tesseract en moteur optionnel (D-16) et la lecture tolérante de P2.4.
  - CI : Tesseract `fra` installé sur le runner Linux, son test devient obligatoire
    (`MASSICOT_EXIGER_TESSERACT`) ; sur Windows, l'essai se fait sur l'exécutable du paquet
    (modèles trouvés à côté de lui) et comprend un OCR d'essai.
  - P2.3 vérifiée : CI verte (run 5 : tests Linux avec Tesseract exigé, OCR d'essai sur
    l'exécutable du paquet Windows), Release `v0.1.5`.
  - P2.4 : commande `decouper-regex`. Texte de chaque page (couche texte, sinon OCR), normalisé
    (accents et casse ignorés par défaut, espaces réduits) ; zone haute en % d'après la position
    des mots ; plusieurs regex ; pages avant le premier en-tête à part (par défaut), jointes au
    premier bloc ou ignorées ; modèle de nom (`{bloc}`, `{page}`, `{fichier}`, `{texte}`, groupes
    de la regex), nom nettoyé, doublons suffixés -2, -3 ; mode à blanc par défaut avec rapport
    JSON par bloc (pages, texte trouvé, groupes, extrait, confiance, alertes) et un jeton ; la
    vraie découpe exige ce jeton (même fichier, mêmes réglages) et écrit `rapport-decoupe.json`
    à côté des fichiers. Lecture tolérante pour le moteur OCR intégré (« ? » accepté à la place
    d'une lettre ou d'un symbole, jamais d'un chiffre). Essai sur le runner Windows : mode à blanc
    puis découpe d'un scan généré.
  - P2.5 : `massicot mcp --racine DIR`, serveur MCP stdio (JSON-RPC, un message par ligne) ;
    un outil par commande, schéma tiré de la définition de la commande ; outil `lister` ;
    chemins confinés au dossier racine (`..`, liens et chemins absolus extérieurs refusés) ;
    configuration pour Claude Code sous Windows dans `README.md` (aussi dans le zip, `LISEZMOI.md`).
    Essai sur le runner Windows : initialisation, liste des outils et un appel.
  - P2.4 vérifiée : CI verte (run 6 : lots scannés inventés lus par l'OCR intégré et par
    Tesseract sous Linux ; mode à blanc puis découpe d'un scan sur le runner Windows),
    Release `v0.1.6`.
  - P2.5 vérifiée sur le runner Windows (run 7) : serveur lancé depuis le paquet, dossier racine
    `C:\Users\…\Temp\…` bien lu, liste des outils et un appel `infos` ; Release `v0.1.7`,
    puis `v0.1.8` (chemins d'exemple neutres dans un test). Le zip contient `LISEZMOI.md`.
  - Brief `docs/BRIEF-P1-P2.md` supprimé : P1.1 à P2.5 faits.
- **Décisions** : D-12 à D-18.
- **Vérifications** : `cargo test --workspace` vert en session à chaque tâche (49 tests à la
  fin, dont l'OCR intégré et Tesseract `fra` sur des lots scannés inventés) ; CI verte à chaque
  push ; contrôle « dépôt public » avant chaque commit (seuls signalements : contenu public des
  crates tierces copiées, adresses d'auteurs et exemples de lopdf, laissés tels quels).
- **Non vérifié sur le PC** : tout ce qui demande le vrai PC (liste R1 à R3 du plan) : lancement
  de l'exe hors du runner de la CI, configuration MCP dans Claude Code, OCR sur de vrais scans.
- **Problèmes et dette** : le moteur OCR intégré lit les accents comme « ? » (compensé par la
  lecture tolérante, signalé dans les rapports) et invente des mots sur les taches d'un scan
  bruité ; Tesseract perd les en-têtes d'un scan très bruité et penché (pas de prétraitement).
  Avertissement de compilation dans `vendor/hayro-interpret` (import inutilisé), laissé tel quel
  (code copié).
- **Prochaine étape** : essais R1 à R3 sur le PC ; P3.1 (maquettes de la fenêtre, plusieurs
  variantes, à valider avant tout code) ; P5.1 sur les vrais lots.

---

## 2026-11-16 · Lancement de Massicot, après l'analyse de PdfCraft

- **Branche et commit** : `main`, commit initial (mémoire du projet, licences, README).
- **Objectif** : savoir si PdfCraft se pilote et couvre PDFsam ; sinon lancer un projet à part.
- **Réalisé** :
  - Analyse du code de PdfCraft (`storytold/pdfcraft`, Rust, MIT/Apache) : ligne de commande
    `pdfcraft-cli` (info, text, edit, combine, extract, split, batch, run, mcp, ui), table
    d'environ 300 outils d'automatisation, serveur MCP, OCR (moteur `ocrs` intégré, anglais
    seulement ; Tesseract en option s'il est installé).
  - Essai réel de PdfCraft 0.21.1 installé sur le PC, sur un PDF de 5 pages généré : fusion,
    lecture du texte par page, découpe avant la page 3, extraction dans l'ordre donné, rotation de
    90° : tout a fonctionné.
  - Écarts avec PDFsam : pas de découpe par texte ou regex, pas de mélange alterné, découpe par
    signets au niveau 1 seulement. Tesseract (français) absent du PC.
  - Cadrage avec l'utilisateur, mémoire du projet posée, dépôt public créé.
- **Décisions** : D-01 à D-11.
- **Vérifications** : commandes `pdfcraft-cli` lancées sur le PC (résultats ci-dessus) ; contrôle
  « dépôt public » avant le premier push.
- **Non vérifié sur le PC** : rien encore de Massicot.
- **Problèmes et dette** : l'OCR intégré de PdfCraft ne connaît que l'anglais ; à évaluer sur des
  en-têtes accentués (« PROCÈS-VERBAL ») avant de choisir Tesseract `fra`.
- **Prochaine étape** : P1.1 en session cloud.
