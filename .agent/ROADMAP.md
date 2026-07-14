# sysprobe — Roadmap (1–2 semaines)

Découpage pensé pour livrer un truc démontrable tôt, puis empiler les features. Chaque phase doit se terminer avec quelque chose qui tourne.

## Phase 0 — Scaffolding (0.5 jour)
- `cargo new sysprobe`, structure de dossiers (voir ARCHITECTURE.md §4)
- Dépendances de base dans `Cargo.toml` : `sysinfo`, `clap`, `serde`, `serde_json`, `anyhow`, `thiserror`
- `cli.rs` avec les sous-commandes en squelette (juste `println!("todo")`)
- **Livrable** : `cargo run -- --help` affiche les bonnes sous-commandes

## Phase 1 — Snapshot CLI simple (1–2 jours)
- Implémenter les collectors CPU / RAM / disque / réseau via `sysinfo`
- `SystemSnapshot` complet, affichage table dans le terminal
- `sysprobe --json` fonctionnel
- **Livrable** : `sysprobe` donne un vrai état système lisible + `--json` propre

## Phase 2 — Processus + tri intelligent (1 jour)
- `ProcessCollector` : top N processus par CPU/RAM
- `sysprobe top --by cpu -n 10`
- **Livrable** : commande `top` utilisable au quotidien

## Phase 3 — Historique léger (1 jour)
- `HistoryBuffer` (ring buffer en mémoire)
- Intégration dans le mode `watch` (courbes simples)
- **Livrable** : les métriques ont une mémoire courte, pas juste l'instant T

## Phase 4 — Dashboard TUI (2–3 jours)
- `ratatui` + `crossterm`, layout avec gauges CPU/RAM, sparklines, table processus
- `sysprobe watch`
- **Livrable** : le morceau le plus impressionnant visuellement pour une démo/vidéo

## Phase 5 — Alertes + mode daemon (2 jours)
- `AlertEngine` avec seuils configurables + hystérésis
- `sysprobe daemon` : boucle tokio, écrit JSON périodique, log les alertes via `tracing`
- Config TOML chargée depuis fichier
- **Livrable** : le produit devient "utilisable en prod perso", pas juste un gadget de démo

## Phase 6 — Bonus bas niveau (optionnel, si temps restant)
- `network_proc.rs` : parsing manuel `/proc/net/tcp` sous Linux pour les connexions actives
- Export historique (`sysprobe history --last 1h`)
- **Livrable** : le talking point "syscalls/bas niveau" pour l'entretien

## Phase 7 — Finition portfolio (0.5–1 jour)
- README avec GIF/screenshot du dashboard TUI (asciinema ou terminalizer)
- Section "pourquoi ce projet" orientée impact, pas liste de features
- `cargo clippy` clean, `cargo fmt`, CI GitHub Actions basique (build + clippy + test)
- **Livrable** : repo prêt à être linké sur le CV / LinkedIn

---

## Ordre de priorité si le temps manque

Si tu dois couper : garde Phase 0 → 1 → 2 → 4 (le TUI est ce qui vend le projet visuellement) → 7.
Les alertes/daemon (Phase 5) et le bonus `/proc` (Phase 6) sont ce qui sépare "bon projet" de "projet mémorable", à faire seulement si Phase 0-4-7 sont solides.
