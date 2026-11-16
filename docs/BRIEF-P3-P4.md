# Brief de la session de développement P4.1, P3.1 et P3.2

> Document de travail : à supprimer une fois la liste P4.1 et les maquettes P3.1 validées, et P3.2
> livrée ou en attente du « oui » (le journal garde la trace).

## Contexte

Lis d'abord `CLAUDE.md`, `docs/PLAN.md`, `docs/DECISIONS.md` (D-01 à D-18) et l'entrée la plus haute de
`docs/JOURNAL.md`. P1.1 à P2.5 sont faits ; la dernière Release est `v0.1.9`. L'utilisateur ne lit pas
le code : il juge sur ce qu'il voit et valide tout ce qui est visible à partir d'aperçus.

En parallèle, une **session locale** sur le PC de l'utilisateur installe la Release et fait les essais
réels (R1 à R3 du plan, puis P5.1 sur de vrais PV). Elle ne touche pas au code. Elle peut pousser des
mises à jour de `docs/JOURNAL.md` et de `docs/PLAN.md` (lignes R, anomalies trouvées). Fais toujours
`git pull --rebase` avant de pousser. Lis ses anomalies dans le journal, et corrige-les en priorité si
l'utilisateur te le demande.

## Travail demandé

**1. P4.1 : les fonctions de PDFsam Enhanced.**
- Recense les fonctions de PDFsam Enhanced (site officiel pdfsam.org).
- Pour chacune, dis si Massicot l'a déjà, et si les moteurs copiés ou d'autres crates de PdfCraft (`storytold/pdfcraft`) la rendent faisable.
- Soumets la liste à l'utilisateur : questions numérotées, choix à toucher, ta recommandation en premier. Pour chaque fonction, dis ce qu'elle lui apporte et l'effort qu'elle demande.
- Inscris la liste et l'ordre qu'il valide dans `docs/PLAN.md` (tâches P4.x), avec une ligne nouvelle dans `docs/DECISIONS.md`.
- Ne code aucune de ces fonctions sans son accord.

**2. P3.1 : maquettes de la fenêtre.**
- Au moins deux variantes, rendues en images, envoyées à l'utilisateur.
- Elles couvrent l'écran principal et tout le parcours de la découpe par en-tête :
  1. choisir le PDF ;
  2. saisir ou choisir la regex ;
  3. voir le rapport à blanc : un bloc par PV, ses pages, le texte trouvé, les alertes ;
  4. couper.
- Attends un « oui » explicite et la variante choisie.

**3. P3.2 : la fenêtre en egui, seulement après ce « oui ».**
- Elle suit la maquette validée.
- Elle vit dans le même exécutable (par exemple `massicot` sans argument, ou `massicot fenetre`).
- Elle réutilise les commandes existantes, sans dupliquer leur logique.
- Ses messages sont en français accentué.
- Un test de rendu d'écran l'accompagne (egui_kittest, comme PdfCraft), et l'image du résultat est montrée à l'utilisateur.
- La CI reste verte et une Release est publiée.

## Règles

- Parle à l'utilisateur en français et peu : une question qui bloque, un choix à valider, le résultat final.
- N'annonce jamais que ça marche sans l'avoir vu marcher : tests, CI, image.
- Chaque livraison a un numéro nouveau ; si la CI échoue, rien n'est livré.
- Push direct sur `main`, sans pull request.
- Dépôt public, rien de privé :
  - ni e-mail, ni nom d'utilisateur, ni chemin de poste ;
  - aucun vrai PDF de l'utilisateur ;
  - identité de commit noreply : `JRRocks <314636139+JRRocks@users.noreply.github.com>`.
- Avant chaque push, mets la mémoire à jour : journal, plan, et une ligne nouvelle dans les décisions.

## Terminé quand

- La liste P4.1 est validée et inscrite au plan.
- Les maquettes sont validées.
- Si l'utilisateur a dit oui à la fenêtre : P3.2 livrée, CI verte, Release téléchargeable.
- Ce brief est supprimé.
- Le message final est court et en français. Il dit :
  - ce qui change pour l'utilisateur ;
  - le lien ;
  - ce qui n'a pas pu être vérifié sur un vrai Windows ;
  - ce que tu attends de lui.
