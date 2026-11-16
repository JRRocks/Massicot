# Idées pour plus tard

> À ne lire que sur demande de l'utilisateur. Rien d'ici n'entre dans le plan sans son accord.

- OCR : prétraitement (redressement, débruitage) avant Tesseract, et vote entre les deux moteurs (`merge::recognize_with_strategy` de PdfCraft) ; notifications de progression MCP pendant un long OCR.
- Fusion avec une sélection de pages par fichier ; propriétés de fichier Windows (VERSIONINFO) et signature de l'exécutable.
- Découpe « par formulaire » intégrée à Massicot (essai du 2026-11-16, méthode déterministe) : début de document par score d'ancres tolérantes dans le haut de page (seuil, ancres réparties dans plusieurs cases), double lecture couche texte + OCR, extraction d'un numéro près d'une étiquette avec nom de repli (« sans numéro », « numéro introuvable ») et rapport CSV.
