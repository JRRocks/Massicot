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
| P1.3 | Installateur Windows `Massicot.<version>.exe` (Inno Setup), publié avec le zip, essayé sur le runner : installation, mise à jour par-dessus, désinstallation | Massicot s'installe comme une autre application : menu Démarrer, Applications installées, `massicot` dans tout terminal ; une version plus récente s'installe par-dessus | 🖥️ |
| P2.1 | Fonctions PDFsam Basic en ligne de commande : fusion, extraction, découpe (toutes les N pages, avant des pages, par signets d'un niveau choisi, par taille), rotation (dont paires/impaires), suppression, déplacement, inversion | Chaque commande produit le PDF attendu sur les PDF d'essai générés | ✅ |
| P2.2 | Mélange alterné (recto et verso scannés à part, second fichier inversé en option) | Deux PDF de 3 pages donnent un PDF 1-A,2-B,3-C… | ✅ |
| P2.3 | OCR des pages scannées (texte invisible ajouté) et texte par page | Un PDF image d'essai devient cherchable | ✅ |
| P2.4 | Découpe par regex : rapport à blanc puis découpe, nom de fichier tiré de la regex | Sur un lot d'essai de PV inventés, un fichier par PV, et le rapport dit quelle page ouvre quel PV | ✅ |
| P2.5 | Serveur MCP (stdio) qui expose toutes les commandes | Claude pilote Massicot depuis le PC | ✅ |
| P3.1 | Maquettes de la fenêtre (plusieurs variantes) | L'utilisateur choisit et dit « oui » | ✅ |
| P3.2 | Fenêtre Windows selon la maquette validée : outils de PDFsam Basic (dont la découpe par en-tête) avec aperçu des pages (D-26) | L'utilisateur ouvre un PDF, en voit les pages et lance depuis la fenêtre fusion, découpe, extraction, rotation, mélange, OCR et découpe par en-tête | 🖥️ |
| P3.3 | Zoom des pages dans la fenêtre (D-31, aperçu `docs/previews/zoom-des-pages.html`) : Ctrl + molette, une page de plus ou de moins par ligne, dans la grille et le rapport à blanc, zoom gardé d'une ouverture à l'autre | Sur le PC, Ctrl + molette vers le haut retire une page par ligne, vers le bas en ajoute une, sans taille intermédiaire ; les pages restent nettes en grand ; le zoom vaut aussi dans le rapport et se retrouve à la réouverture | ⬜ |
| P4.1 | Liste des fonctions de PDFsam Enhanced utiles ici, soumise à l'utilisateur | L'utilisateur valide la liste et l'ordre | ✅ |
| P4.2 | Lot 1 « ranger les scans » (D-25), après P3.2 : compresser ; recadrer, marges, format de page ; créer un PDF depuis des images ; insérer les pages d'un autre PDF ; numéros de page, en-tête, pied de page, filigrane. En ligne de commande, MCP et fenêtre | Chaque fonction produit le PDF attendu sur les PDF d'essai et se lance depuis la fenêtre | ⬜ |
| P4.3 | Lot 4 « dans la fenêtre » (D-25), après P4.2 : modifier le texte et les polices ; commentaires (notes, tampons, surlignage) ; formulaires (remplir, créer, exporter les données) | L'utilisateur corrige un mot, annote une page et remplit un formulaire depuis la fenêtre | ⬜ |
| P5.1 | Essais sur les vrais lots de PV de l'utilisateur (hors dépôt, `essais-prives/`) | Les PV sortent un par fichier, sans erreur de coupe | 🔄 premier lot fait (journal du 2026-11-16), en attente du jugement de l'utilisateur |

## À vérifier sur le PC

| N° | Depuis la version | Ce qu'il fait | Ce qu'il doit voir | État |
|---|---|---|---|---|
| R1 | 0.1.7 | Télécharge le zip de la dernière Release, le décompresse, lance `.\massicot.exe --version` dans PowerShell | `massicot 0.1.<n>` (SmartScreen peut avertir une fois) | ✅ 2026-11-16, 0.1.9 installée dans `%LOCALAPPDATA%\Programs\Massicot` |
| R2 | 0.1.7 | `claude mcp add --scope user massicot -- "<chemin>\massicot.exe" mcp --racine "<dossier des PDF>"` (README), puis `claude mcp list` | `massicot … ✔ Connected` ; Claude décrit un PDF du dossier | ✅ 2026-11-16, 0.1.9 : ajouté (racine `essais-prives/`), `✔ Connected` vu ; 2026-11-22 : dans une session ouverte après l'ajout, Claude a décrit un vrai lot par l'outil MCP `infos` (pages, formats, signets) |
| R3 | 0.1.7 | Demande à Claude un mode à blanc de `decouper-regex` sur un vrai lot de PV scannés (dans `essais-prives/`) | Un bloc par PV, en-têtes lus (accents en « ? » avec le moteur intégré) ; à faire avant P5.1 | ✅ 2026-11-16 (lot de 264 pages : 56 blocs ; voir journal) |
| R4 | 0.1.21 (P1.3) | Télécharge `Massicot.<version>.exe` de la dernière Release et le lance ; cherche « Massicot » dans le menu Démarrer et l'ouvre ; dans un nouveau terminal, `massicot --version` ; regarde Paramètres ▸ Applications ▸ Applications installées ; redémarre Claude Code (`massicot … ✔ Connected` ; sa configuration MCP doit viser `%LOCALAPPDATA%\Programs\Massicot\massicot.exe`) ; à la Release suivante, installe la nouvelle par-dessus ; une fois (facultatif) : désinstalle depuis Paramètres, puis réinstalle | SmartScreen avertit (exe non signé : « Informations complémentaires », « Exécuter quand même ») ; assistant en français, « Prêt à installer » (la 0.1.9 posée à la main est reprise) ; console « Massicot » avec l'accueil ; entrée Massicot avec son icône, éditeur « Massicot » ; mise à jour : « Prêt à mettre à jour », sans désinstaller ; désinstallation : dossier, raccourci et PATH retirés | 🔄 2026-11-22, 0.1.29 installée en mode silencieux par-dessus la 0.1.9 posée à la main : `massicot --version`, entrée de désinstallation, raccourci du menu Démarrer et PATH vus ; l'exécutable tenu ouvert par des serveurs MCP a été mis de côté sans bloquer l'installation. Reste : SmartScreen et écrans de l'assistant, lancement depuis le menu Démarrer, mise à jour par la Release suivante, désinstallation |
| R5 | (P3.2) | Ouvre Massicot depuis le menu Démarrer ; ouvre un PDF (bouton ou glisser-déposer) ; choisit « Découpe par en-tête », suit les quatre étapes sur un vrai lot (dans `essais-prives/`) ; essaie un autre outil (extraire des pages sélectionnées) | La fenêtre s'ouvre sans console derrière ; vignettes des pages ; rapport à blanc juste ; fichiers écrits ; « Ouvrir le dossier » ouvre l'Explorateur | 🔄 2026-11-22, 0.1.29 : lancée depuis un raccourci vers `massicot-fenetre.exe`, aucune console visible ; vrai lot de 264 pages ouvert, vignettes affichées ; découpe par en-tête suivie en quatre étapes : rapport à blanc de 56 blocs (mêmes coupes que la méthode LLM), 56 fichiers écrits. Reste : menu Démarrer, glisser-déposer, « Ouvrir le dossier », un autre outil |
