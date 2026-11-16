# Attribution

Ce que Massicot reprend ou distribue d'autrui : auteur, source, licence. Massicot lui-même est
sous licence MIT ou Apache-2.0, au choix (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`).

## Distribué dans le zip Windows

| Fichier | Titre | Auteur | Licence | Source | SHA-256 |
|---|---|---|---|---|---|
| `modeles/text-detection.onnx` | Ocrs text detection model | Robert Knight (ocrs-models ; entraîné sur HierText) | [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/legalcode.txt) | https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.onnx (fiche du modèle : https://huggingface.co/robertknight/ocrs) | `a917b23dbd9524b465df7e922641b3ff2981623df4ded5a0234004ef2fee7cfe` |
| `modeles/text-recognition.onnx` | Ocrs text recognition model | Robert Knight (ocrs-models ; entraîné sur HierText) | [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/legalcode.txt) | https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.onnx (fiche du modèle : https://huggingface.co/robertknight/ocrs) | `86c145c2edb96c8caed5b1ebb8f44d706408922211451c309c625157dd6061c5` |

Les deux modèles sont distribués sans modification. Ils ne sont pas dans le dépôt : la CI les
télécharge (`scripts/modeles-ocr.sh`) et vérifie leur empreinte avant de les mettre dans le zip.
