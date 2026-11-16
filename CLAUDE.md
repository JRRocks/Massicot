# CLAUDE.md : Massicot

Boîte à outils PDF pour le PC Windows de l'utilisateur (fusion, découpe, extraction, rotation,
mélange, OCR, et découpe d'un lot scanné à chaque en-tête trouvé par regex), pilotée par Claude
en ligne de commande et par MCP, et par l'utilisateur dans une fenêtre.

Les règles communes à tous mes projets viennent de la Library (`JRRocks/Library`, `SOCLE.md`).
Ce fichier ne contient que ce qui est propre à ce projet. Il reste court : le détail va dans `docs/`.

## Mémoire du projet

| Fichier | Contenu |
|---|---|
| `docs/PLAN.md` | Tâches numérotées, statut, critères, liste « à vérifier sur le PC » |
| `docs/JOURNAL.md` | Une entrée par session, la plus récente en haut |
| `docs/DECISIONS.md` | Décisions numérotées et datées |
| `docs/idees-futur.md` | Idées pour plus tard : à ne lire que sur ma demande |

## Commandes

À compléter dès que le squelette compile (tâche P1.1) :

```bash
cargo test --workspace            # vérifier
cargo build --release -p massicot  # compiler l'exécutable
```

## Livraison

- Canal : GitHub Releases du dépôt, fichier `Massicot.<version>.zip` (Windows x64 : exécutable,
  modèles OCR, licences), produit par la CI à chaque push sur `main`, tests passés d'abord.
- Version : monte toute seule à chaque compilation de la CI ; jamais deux livraisons au même numéro.
  `massicot --version` l'affiche.
- Signature : aucune pour l'instant (Windows SmartScreen peut avertir au premier lancement).
- Dépôt : **public** ; push direct sur `main`, sans pull request.

## Sauvegarde

- Miroir Drive : dossier `Massicot/` du dossier Drive « Claude Code » (chemin exact dans le
  `CLAUDE.md` de ce dossier Drive). Du PC vers le Drive seulement, jamais depuis une session cloud :
  `robocopy "<dossier local>" "<lecteur>:\…\Claude Code\Massicot" /MIR /XD .git target essais-prives`

## Conventions

- Messages de l'outil (CLI, fenêtre, erreurs) : français avec accents ; code et commentaires :
  anglais (comme les moteurs repris de PdfCraft) ; messages de commit : français.
- Identité des commits : `JRRocks <314636139+JRRocks@users.noreply.github.com>`.

## Ce qui compte vraiment dans ce code

- **Dépôt public : aucun vrai PDF de l'utilisateur** (PV, scans, extraits de texte, rapports de
  découpe) n'entre dans le dépôt, ni dans un commit, ni dans le journal. Ils vivent dans
  `essais-prives/` (ignoré). Les jeux d'essai sont générés ou inventés.
- **La découpe par en-tête ne dépend d'aucun LLM** : texte de chaque page (couche texte, sinon
  OCR), puis regex. Claude peut proposer la regex, mais le moteur tourne seul.
- **Toujours un rapport à blanc avant de couper** : quelles pages ouvrent un bloc, avec le texte
  reconnu, pour vérifier avant d'écrire les fichiers.
- **Moteurs repris de PdfCraft** : licences MIT/Apache et mention d'origine conservées
  (`NOTICE`, `ATTRIBUTION.md`, révision source notée) ; ni le nom, ni le logo ArtCraft/PdfCraft.
- **Rien de visible sans aperçu validé** : la fenêtre se montre d'abord en maquettes (variantes),
  et ne se code qu'après le « oui » de l'utilisateur.

## Ce que seul le PC peut vérifier

Lancement de l'exécutable sur Windows réel, la fenêtre, l'OCR sur les vrais scans de
l'utilisateur, le pilotage par Claude depuis le PC (MCP).
