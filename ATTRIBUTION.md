# Attribution

Ce que Massicot reprend ou distribue d'autrui : origine, révision, auteurs, licence. Massicot
lui-même est sous licence MIT ou Apache-2.0, au choix (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`).
Les textes de licence des éléments tiers sont dans le dossier `licences/` du zip Windows.

## 1. Moteurs PDF repris de PdfCraft

- **Dépôt** : https://github.com/storytold/pdfcraft
- **Révision** : `412e6963b56c62d6cc6f29cb4cb10d521dc825a4` (branche `main` du 2026-11-16, version 0.21.1)
- **Licence** : MIT OR Apache-2.0, Copyright (c) 2026 ArtCraft Team and the PdfCraft contributors
- **Copie** : `scripts/importer-moteurs.py`, qui note aussi chaque écart.

| Dossier | Paquet | Rôle dans Massicot |
|---|---|---|
| `crates/cos` | `pdfcraft-cos` | Lecture et écriture des objets PDF (façade de `craft-pdf`) |
| `crates/content` | `pdfcraft-content` | Flux de contenu des pages |
| `crates/organize` | `pdfcraft-organize` | Pages et structure : fusion, extraction, découpe, rotation, suppression, déplacement, inversion, signets |
| `crates/fonts` | `pdfcraft-fonts` | Polices et encodage du texte (couche de texte invisible de l'OCR) |
| `crates/geom` | `pdfcraft-geom` | Géométrie (dépendance du rendu) |
| `crates/render` | `pdfcraft-render` | Rendu des pages en image et texte des pages avec sa position |
| `crates/ocr` | `pdfcraft-ocr` | OCR (moteur `ocrs` intégré, Tesseract en option) et couche de texte invisible |

Les sources sont copiées sans changement, sauf ce qui suit. Les tests, bancs d'essai et exemples
des crates ne sont pas copiés ; leurs manifestes `Cargo.toml` sont réécrits pour se suffire à
eux-mêmes (valeurs reprises de l'espace de travail d'origine, sans dépendances de test).

| Fichier | Changement | Pourquoi |
|---|---|---|
| `crates/ocr/src/tesseract.rs` (`encode_png`, marqué `Massicot patch:`) | L'image remise à Tesseract est encodée en PNG par la crate `png` au lieu de `craft-image-codecs` | `craft-image-codecs` (35 000 lignes, une douzaine de formats d'image) ne servait qu'à cela |

Dépôt PdfCraft non repris : ni le reste des crates (moteur d'édition, formulaires, signature,
interface…), ni `docs/brand`, ni les noms et logos ArtCraft et PdfCraft.

## 2. Dépendances de PdfCraft venant de craft-libs

PdfCraft tire ces crates par dépendance git ; Massicot en garde une copie (`vendor/craft-libs/`).

- **Dépôt** : https://github.com/storytold/craft-libs
- **Révision** : `8e03cd016d42032b867fef6bad86bc69bf2c1920` (celle qu'épingle PdfCraft pour `craft-pdf`)
- **Licence** : MIT OR Apache-2.0, Copyright (c) 2026 ArtCraft Team and the craft-libs contributors

| Dossier | Paquet | Écart avec l'original |
|---|---|---|
| `vendor/craft-libs/pdf` | `craft-pdf` | Fonction `write-svg` retirée du manifeste (elle dépend de crates de craft-libs non copiées ; rien ne l'active) |
| `vendor/craft-libs/stream-codecs` | `craft-stream-codecs` | Utilise `vendor/hayro-jbig2` (copie de PdfCraft, plus récente) au lieu de la copie de craft-libs, qui en est une version antérieure |

## 3. Crates tierces corrigées par PdfCraft (`vendor/`)

Copiées entières depuis `vendor/` de PdfCraft, à la même révision, avec les correctifs de PdfCraft
(marqués `PdfCraft patch:` dans les sources ; détail dans `vendor/README.md` de PdfCraft).
Elles remplacent les versions de crates.io (`[patch.crates-io]` du `Cargo.toml` racine).

| Crate | Version | Licence | Auteurs |
|---|---|---|---|
| `hayro`, `hayro-syntax`, `hayro-interpret`, `hayro-cmap`, `hayro-postscript`, `hayro-jbig2` | 0.7.1, 0.7.2, 0.7.0, 0.1.0, 0.1.0, 0.3.0 | Apache-2.0 OR MIT | The Hayro Authors |
| `lopdf` | 0.45.0 | MIT | Junfeng Liu |
| `vello_common`, `vello_cpu` | 0.0.8 | Apache-2.0 OR MIT | the Vello Authors |

## 4. Matériel tiers inclus dans l'exécutable

Repris des fichiers `NOTICE`, `ATTRIBUTION.toml` et `LICENSES.toml` de PdfCraft, pour ce que les
crates copiées embarquent.

| Élément | Auteur | Licence | Source |
|---|---|---|---|
| Police Inter (Regular, Medium, SemiBold), `assets/fonts/Inter-*.ttf` | The Inter Project Authors (Rasmus Andersson) | SIL OFL 1.1 (`licences/OFL-Inter.txt`) | https://github.com/rsms/inter |
| Police JetBrains Mono Regular, `assets/fonts/JetBrainsMono-Regular.ttf` | The JetBrains Mono Project Authors | SIL OFL 1.1 (`licences/OFL-JetBrainsMono.txt`) | https://github.com/JetBrains/JetBrainsMono |
| Police Dancing Script, `assets/fonts/DancingScript.ttf` | The Dancing Script Project Authors (Pablo Impallari) | SIL OFL 1.1, nom réservé « Dancing Script » (`licences/OFL-DancingScript.txt`) | https://github.com/google/fonts (ofl/dancingscript) |
| Polices Foxit de substitution des 14 polices standard, `vendor/hayro-interpret/assets/*.pfb` | Copyright 2014 PDFium Authors ; code d'origine copyright 2014 Foxit Software Inc. | BSD-3-Clause (`licences/LICENSE_FOXIT`) | https://pdfium.googlesource.com/pdfium |
| Ressources CMap (embarquées par `hayro-cmap`) | Copyright 1990-2023 Adobe | BSD-3-Clause (`licences/CMap-LICENSE.txt`) | https://github.com/adobe-type-tools/cmap-resources |
| Profil de couleur `CGATS001Compat-v2-micro.icc` (`hayro-interpret`) | Clinton Ingram | CC0-1.0 (`licences/CGATS_LICENSE.txt`) | https://github.com/saucecontrol/Compact-ICC-Profiles |
| Données Unicode ICU4X (`icu_normalizer_data`, `icu_properties_data`, via crates.io) | Unicode, Inc. | Unicode-3.0 | https://github.com/unicode-org/icu4x |

Ni Adobe, ni Foxit, ni les auteurs de ces éléments ne cautionnent Massicot.

## 5. Distribué à côté de l'exécutable, dans le zip Windows

| Fichier | Titre | Auteur | Licence | Source | SHA-256 |
|---|---|---|---|---|---|
| `modeles/text-detection.onnx` | Ocrs text detection model | Robert Knight (ocrs-models ; entraîné sur HierText) | [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/legalcode.txt) | https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.onnx (fiche du modèle : https://huggingface.co/robertknight/ocrs) | `a917b23dbd9524b465df7e922641b3ff2981623df4ded5a0234004ef2fee7cfe` |
| `modeles/text-recognition.onnx` | Ocrs text recognition model | Robert Knight (ocrs-models ; entraîné sur HierText) | [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/legalcode.txt) | https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.onnx (fiche du modèle : https://huggingface.co/robertknight/ocrs) | `86c145c2edb96c8caed5b1ebb8f44d706408922211451c309c625157dd6061c5` |

Les deux modèles sont distribués sans modification. Ils ne sont pas dans le dépôt : la CI les
télécharge (`scripts/modeles-ocr.sh`) et vérifie leur empreinte avant de les mettre dans le zip.

## 6. Bibliothèques de crates.io

La liste complète des crates compilées dans l'exécutable Windows, avec version, licence et dépôt,
est produite à chaque livraison (`scripts/dependances.py`) et rangée dans le zip :
`licences/dependances.md`.
