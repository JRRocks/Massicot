# Massicot

Boîte à outils PDF pour Windows, hors ligne et sans compte : fusionner, découper, extraire,
tourner, mélanger, reconnaître le texte des scans (OCR)… et **découper un lot de documents
scannés à chaque en-tête reconnu par une expression régulière** (par exemple un procès-verbal
par fichier), sans intelligence artificielle.

Massicot se pilote en ligne de commande et par un serveur MCP (pour un agent comme Claude) ;
une fenêtre viendra ensuite.

## Installer (Windows 64 bits)

1. Téléchargez le dernier `Massicot.<version>.zip` :
   https://github.com/JRRocks/Massicot/releases/latest
2. Décompressez-le, par exemple dans `C:\Users\<utilisateur>\Outils\Massicot`.
   `massicot.exe` et le dossier `modeles\` (OCR) doivent rester côte à côte.
3. Ouvrez un terminal dans ce dossier et tapez `.\massicot.exe --aide`.
   L'exécutable n'est pas signé : au premier lancement, Windows SmartScreen peut avertir
   (« Informations complémentaires », puis « Exécuter quand même »).

Pour mettre à jour, remplacez le dossier par celui de la nouvelle version : Massicot ne garde
aucune donnée. `massicot --version` affiche le numéro installé.

## Commandes

Chaque commande a son aide : `massicot <commande> --aide`. Les pages se comptent à partir de 1
(« 1-3,5,8- » : pages 1 à 3, 5, et de 8 à la fin). Rien n'est remplacé sans `--ecraser`.
`--json` donne le résultat en JSON.

| Commande | Rôle | Exemple |
|---|---|---|
| `infos` | Pages, formats, métadonnées, signets | `massicot infos lot.pdf` |
| `texte` | Texte de chaque page (couche texte, sinon OCR) | `massicot texte lot.pdf --pages 1-2` |
| `fusionner` | Plusieurs PDF en un | `massicot fusionner a.pdf b.pdf --sortie ab.pdf` |
| `extraire` | Des pages, dans l'ordre donné | `massicot extraire lot.pdf --pages 5,1-3 --sortie x.pdf` |
| `decouper` | Toutes les N pages, avant des pages, aux signets d'un niveau, par taille | `massicot decouper lot.pdf --toutes 2 --dossier parties` |
| `tourner` | Rotation (toutes, sélection, paires, impaires) | `massicot tourner lot.pdf --angle 90 --paires --sortie t.pdf` |
| `supprimer` | Retire des pages | `massicot supprimer lot.pdf --pages 2 --sortie s.pdf` |
| `deplacer` | Déplace des pages | `massicot deplacer lot.pdf --pages 7-8 --vers debut --sortie d.pdf` |
| `inverser` | Inverse l'ordre des pages | `massicot inverser lot.pdf --sortie i.pdf` |
| `melanger` | Recto et verso scannés à part, page à page | `massicot melanger recto.pdf verso.pdf --verso-inverse --sortie rv.pdf` |
| `ocr` | Ajoute le texte reconnu, invisible : le scan devient cherchable | `massicot ocr scan.pdf --sortie cherchable.pdf` |
| `decouper-regex` | Un fichier par en-tête trouvé (voir ci-dessous) | |
| `mcp` | Serveur MCP pour Claude (voir plus bas) | |

## Découper un lot de procès-verbaux par en-tête

Chaque page qui porte l'en-tête ouvre un bloc, qui court jusqu'au prochain en-tête ; un PDF par
bloc. Le texte de chaque page vient de sa couche texte, sinon de l'OCR (`--ocr auto`, par
défaut ; ou `toujours`, `jamais`).

1. **Mode à blanc** (par défaut : rien n'est écrit) :

   ```
   massicot decouper-regex lot.pdf --regex "PROCÈS-VERBAL N° (?<num>\d+)" --zone-haute 30 --modele "PV-{num}.pdf" --dossier PV
   ```

   Le rapport dit, pour chaque bloc, ses pages, l'en-tête trouvé, un extrait de la première page,
   la confiance de l'OCR et les alertes (page vide, nom en double, confiance faible…).
   `--rapport rapport.json` l'enregistre ; `--json` l'affiche en entier.
2. **Découpe**, après lecture du rapport : la même commande, plus `--couper <jeton>` (le jeton
   que donne le mode à blanc ; il change si le fichier ou une option change). Le rapport est
   écrit à côté des fichiers (`rapport-decoupe.json`).

Bon à savoir :

- majuscules et accents sont ignorés par défaut (`--respecter-casse`, `--respecter-accents`),
  et les espaces (retours à la ligne compris) réduits à un seul ;
- `--zone-haute 30` ne cherche l'en-tête que dans les 30 % du haut de la page : une simple
  mention « PV n° 11 » dans le corps ne coupe pas ;
- plusieurs `--regex` possibles : une page qui en vérifie une ouvre un bloc ;
- pages avant le premier en-tête : un fichier à part par défaut
  (`--avant-premier a-part|premier|ignorer`) ;
- nom des fichiers : `{bloc}` (numéro du bloc), `{page}`, `{fichier}`, `{texte}` et les groupes
  de la regex (`{num}`, `{1}`…) ; un nom déjà pris devient `PV-12-2.pdf`, `PV-12-3.pdf`…

## OCR : moteur intégré ou Tesseract

- **Intégré** (par défaut si Tesseract est absent) : rien à installer, modèles dans `modeles\`.
  Il ne connaît pas les accents : « PROCÈS-VERBAL N° 12 » se lit « PROC?S-VERBAL N? 12 ». La
  découpe par en-tête en tient compte (chaque lettre ou symbole de la regex accepte aussi « ? »
  sur ces pages) et le signale dans le rapport.
- **Tesseract** (facultatif) : lit les accents sur un scan propre. Installez-le avec la langue
  française (par exemple la version Windows de l'université de Mannheim :
  https://github.com/UB-Mannheim/tesseract/wiki, en cochant « French »). Massicot le trouve dans
  le `PATH` ou dans `C:\Program Files\Tesseract-OCR`. `--moteur tesseract` l'impose,
  `--moteur integre` l'écarte.

## Piloter Massicot par Claude (serveur MCP)

`massicot mcp --racine <DOSSIER>` lance un serveur MCP (stdio). Chaque commande y est un outil,
avec les mêmes options et les mêmes messages ; l'outil `lister` donne les PDF du dossier. Les
chemins sont relatifs au dossier racine et n'en sortent pas.

**Ajouter le serveur à Claude Code** (une fois, pour tous les projets), dans PowerShell ou
l'invite de commandes :

```
claude mcp add --scope user massicot -- "C:\Users\<utilisateur>\Outils\Massicot\massicot.exe" mcp --racine "C:\Users\<utilisateur>\Documents\PDF"
```

Remplacez les deux chemins : celui de `massicot.exe`, et le dossier où sont vos PDF (le serveur
n'accède qu'à lui). Vérifiez ensuite avec `claude mcp list` (ligne `massicot … ✔ Connected`),
ou `/mcp` dans une session.

Pour un seul projet, l'équivalent dans le fichier `.mcp.json` du projet (barres obliques
inverses doublées ; Claude Code demande d'approuver ce serveur au premier lancement) :

```json
{
  "mcpServers": {
    "massicot": {
      "command": "C:\\Users\\<utilisateur>\\Outils\\Massicot\\massicot.exe",
      "args": ["mcp", "--racine", "C:\\Users\\<utilisateur>\\Documents\\PDF"]
    }
  }
}
```

Ensuite, demandez simplement à Claude : « découpe `lot.pdf` en un fichier par procès-verbal ».
Il lance le mode à blanc, vous montre le rapport, puis coupe avec le jeton.

Durées : l'OCR prend une à trois secondes par page. Claude Code abandonne un appel d'outil resté
30 minutes sans réponse (`CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`, en millisecondes) ; pour de très
gros lots, découpez-les d'abord en parties (`decouper --toutes 100`). Un résultat très long
(`texte` sur des centaines de pages) peut dépasser la limite de Claude Code
(`MAX_MCP_OUTPUT_TOKENS`) : demandez alors le texte par plages de pages.

## Licence et origine

Licence : MIT ou Apache-2.0, au choix (`LICENSE-MIT`, `LICENSE-APACHE`).

Les moteurs PDF viennent de [PdfCraft](https://github.com/storytold/pdfcraft) (MIT ou
Apache-2.0) ; leur révision, les crates reprises et le matériel tiers inclus (polices, modèles
OCR…) sont détaillés dans `ATTRIBUTION.md` et `NOTICE`.
