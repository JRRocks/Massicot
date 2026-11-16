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
- **Décisions** : D-12, D-13.
- **Vérifications** : `cargo test --workspace` vert en session (3 tests) ; contrôle « dépôt public »
  sans signalement.
- **Non vérifié sur le PC** : tout (aucun essai sur Windows réel).
- **Problèmes et dette** : l'essai de découpe sur le runner Windows arrive avec la commande de
  découpe (P2.1).
- **Prochaine étape** : P1.2.

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
