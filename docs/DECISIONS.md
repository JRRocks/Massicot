# Décisions

> Une décision ne se réécrit pas. Pour la changer : une ligne nouvelle qui la remplace, et la
> mention « remplacée par D-xx » sur l'ancienne.
> Colonne « Qui » : `utilisateur` pour ce qu'il a tranché, `développeur` pour un choix technique
> sur lequel il peut revenir.

| N° | Date | Décision | Pourquoi | Qui |
|---|---|---|---|---|
| D-01 | 2026-11-16 | Pas de fork GitHub : un projet à part entière, fortement inspiré de PdfCraft, dont il reprend les moteurs ; les fonctions en plus s'inspirent de PDFsam Enhanced. | PdfCraft (0.21.1) a les moteurs mais pas la découpe par regex, ni le mélange alterné, ni la découpe par signets au-delà du niveau 1. Un fork GitHub d'un dépôt public serait forcément public et lié. | utilisateur |
| D-02 | 2026-11-16 | Nom : Massicot. | Outil qui coupe les liasses ; court, français, sans conflit de marque. | utilisateur |
| D-03 | 2026-11-16 | Interface : ligne de commande + serveur MCP (pilotage par Claude) + fenêtre Windows. | L'utilisateur veut pouvoir s'en servir seul, comme PDFsam, et me le faire piloter. | utilisateur |
| D-04 | 2026-11-16 | Dépôt public `JRRocks/Massicot`. | Compilation gratuite ; règles « rien de privé » dès le premier commit. | utilisateur |
| D-05 | 2026-11-16 | Messages de l'outil et commits en français ; code et commentaires en anglais. | Cohérence avec les moteurs repris de PdfCraft. | utilisateur |
| D-06 | 2026-11-16 | La découpe d'un lot de PV se fait par regex sur le texte de chaque page (couche texte, sinon OCR), sans LLM. Un en-tête ouvre un bloc, qui court jusqu'au prochain en-tête ; un fichier PDF par bloc. | Demande de l'utilisateur : moteur déterministe. | utilisateur |
| D-07 | 2026-11-16 | Le code se fait d'abord en session cloud ; les essais sur de vrais PDF viendront plus tard. | Demande de l'utilisateur. | utilisateur |
| D-08 | 2026-11-16 | Cible : PC Windows 64 bits, hors ligne, sans compte, pour l'utilisateur seul. | Usage décrit (« sur mon ordinateur »). | développeur |
| D-09 | 2026-11-16 | Langage : Rust partout (moteur, CLI, MCP) ; fenêtre en egui comme PdfCraft. | Reprise directe des moteurs Rust de PdfCraft, un seul exécutable sans dépendance. | développeur |
| D-10 | 2026-11-16 | Les crates de PdfCraft utiles sont copiées dans le dépôt (révision source notée), pas tirées par dépendance git. | « Projet à part entière » : il doit compiler même si PdfCraft change ou disparaît. À réviser si la copie s'avère démesurée (nouvelle ligne). | développeur |
| D-11 | 2026-11-16 | Licence MIT OR Apache-2.0, comme PdfCraft ; ses mentions de copyright et d'origine sont conservées. | Obligation des licences reprises. | développeur |
