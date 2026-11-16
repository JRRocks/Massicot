# Journal de bord

> Entrée la plus récente **en haut**. On ne modifie ni ne supprime une entrée passée :
> une erreur se corrige par une nouvelle entrée.

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
