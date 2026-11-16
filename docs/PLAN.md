# Plan

> Statuts : ⬜ à faire · 🔄 en cours · ✅ terminé (critère vérifié, vérification écrite au journal) ·
> 🖥️ fait, à vérifier sur le PC · ⛔ bloqué (avec la raison).
> Aucune idée « pour plus tard » ici : elles vont dans `docs/idees-futur.md`.

## But

Une boîte à outils PDF pour Windows qui couvre PDFsam Basic, puis les fonctions utiles de PDFsam
Enhanced, avec en plus la découpe d'un lot scanné à chaque en-tête reconnu par regex (un
procès-verbal par fichier), pilotable par Claude (CLI, MCP) et par l'utilisateur (fenêtre).

## Tâches

| N° | Tâche | Critère que l'utilisateur peut observer | Statut |
|---|---|---|---|
| P1.1 | Squelette Rust + CI : tests, puis compilation Windows, puis publication | Chaque push sur `main` publie `Massicot.<version>.zip` dans les Releases ; `massicot --version` affiche ce numéro | ✅ |
| P1.2 | Reprise des moteurs de PdfCraft (copie, licences, `ATTRIBUTION.md`, révision source) | La CI reste verte ; `massicot infos <fichier.pdf>` décrit un PDF | ✅ |
| P2.1 | Fonctions PDFsam Basic en ligne de commande : fusion, extraction, découpe (toutes les N pages, avant des pages, par signets d'un niveau choisi, par taille), rotation (dont paires/impaires), suppression, déplacement, inversion | Chaque commande produit le PDF attendu sur les PDF d'essai générés | ✅ |
| P2.2 | Mélange alterné (recto et verso scannés à part, second fichier inversé en option) | Deux PDF de 3 pages donnent un PDF 1-A,2-B,3-C… | ✅ |
| P2.3 | OCR des pages scannées (texte invisible ajouté) et texte par page | Un PDF image d'essai devient cherchable | ✅ |
| P2.4 | Découpe par regex : rapport à blanc puis découpe, nom de fichier tiré de la regex | Sur un lot d'essai de PV inventés, un fichier par PV, et le rapport dit quelle page ouvre quel PV | ✅ |
| P2.5 | Serveur MCP (stdio) qui expose toutes les commandes | Claude pilote Massicot depuis le PC | 🖥️ |
| P3.1 | Maquettes de la fenêtre (plusieurs variantes) | L'utilisateur choisit et dit « oui » | ⬜ |
| P3.2 | Fenêtre Windows selon la maquette validée | L'utilisateur fait une découpe par en-tête depuis la fenêtre | ⬜ |
| P4.1 | Liste des fonctions de PDFsam Enhanced utiles ici, soumise à l'utilisateur | L'utilisateur valide la liste et l'ordre | ⬜ |
| P5.1 | Essais sur les vrais lots de PV de l'utilisateur (hors dépôt, `essais-prives/`) | Les PV sortent un par fichier, sans erreur de coupe | ⬜ |

## À vérifier sur le PC

| N° | Depuis la version | Ce qu'il fait | Ce qu'il doit voir | État |
|---|---|---|---|---|
| R1 | 0.1.7 | Télécharge le zip de la dernière Release, le décompresse, lance `.\massicot.exe --version` dans PowerShell | `massicot 0.1.<n>` (SmartScreen peut avertir une fois) | à essayer |
| R2 | 0.1.7 | `claude mcp add --scope user massicot -- "<chemin>\massicot.exe" mcp --racine "<dossier des PDF>"` (README), puis `claude mcp list` | `massicot … ✔ Connected` ; Claude décrit un PDF du dossier | à essayer |
| R3 | 0.1.7 | Demande à Claude un mode à blanc de `decouper-regex` sur un vrai lot de PV scannés (dans `essais-prives/`) | Un bloc par PV, en-têtes lus (accents en « ? » avec le moteur intégré) ; à faire avant P5.1 | à essayer |
