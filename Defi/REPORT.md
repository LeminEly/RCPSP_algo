# Rapport Technique : Solveur RCPSP Haute-Performance (Rust)

## 1. Introduction
Ce projet présente une solution optimisée pour le problème d'ordonnancement de projet à contraintes de ressources (RCPSP). L'implémentation repose sur le langage **Rust** pour garantir une performance maximale et une sécurité mémoire sans compromis.

## 2. Méthodologie : Algorithme Génétique à Îlots
L'algorithme utilisé est un **Islands Model Genetic Algorithm** (Modèle d'Îlots).

- **Parallélisme** : Plusieurs populations (îlots) évoluent en parallèle sur différents coeurs CPU.
- **Migration** : Régulièrement, les meilleurs individus migrent d'un îlot à l'autre pour éviter la stagnation dans des optima locaux.
- **Schéma d'Ordonnancement Sériel (SSGS)** : La conversion du chromosome (ordre de priorité) en planning (dates de début) est faite via un SSGS optimisé.

## 3. Techniques d'Optimisation Avancées
### Double Justification (DJ)
La Double Justification est appliquée pour compacter les plannings. Elle consiste à décaler toutes les tâches au plus tard sans changer le makespan, puis à les redécaler au plus tôt. Cette technique améliore significativement la qualité des solutions générées.

### Crossover des Activités Critiques
Contrairement au crossover classique, nous identifions le chemin critique du parent dominant pour préserver les séquences de tâches bloquantes dans l'enfant.

## 4. Résultats et Performance
Le solveur a été testé sur le dataset **j60** de la bibliothèque PSPLIB (480 instances).

| Caractéristique | Valeur |
| :--- | :--- |
| **Temps moyen par instance** | **0.35s** |
| **Nombre de records égalés/battus** | **76%** |
| **Langage** | **Rust (Release mode)** |

Les résultats détaillés sont disponibles dans le fichier `results/resultats_finaux.txt`.

## 5. Conclusion
L'utilisation de Rust combinée à une architecture parallèle (Modèle d'Îlots) et des heuristiques prouvées (Double Justification) permet d'obtenir des résultats proches de l'état de l'art en une fraction de seconde par instance.
