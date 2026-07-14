# Prompts Codex — sysprobe

À utiliser dans l'ordre des phases de ROADMAP.md. Colle `ARCHITECTURE.md` en contexte à chaque fois (ou référence-le si Codex a accès au repo).

---

## Prompt Phase 0 — Scaffolding

```
Tu travailles sur le projet Rust `sysprobe`, décrit dans ARCHITECTURE.md à la racine du repo.

Tâche : créer le scaffolding initial.
1. Initialise la structure de dossiers exactement comme décrit dans ARCHITECTURE.md section 4 (src/collectors/, src/ui/, tests/, config/).
2. Remplis Cargo.toml avec les dépendances déjà fournies (ne les modifie pas sauf si une version ne compile pas, auquel cas ajuste au minimum et documente pourquoi).
3. Crée cli.rs avec un enum Command (clap derive) couvrant : snapshot (défaut), Watch, Daemon, History, Top. Chaque variante doit avoir les flags décrits dans ARCHITECTURE.md section 6.
4. main.rs doit parser la CLI et dispatcher vers des fonctions stub (todo!()) par commande, avec un println! placeholder clair.
5. Vérifie que `cargo build` et `cargo run -- --help` fonctionnent sans erreur.

Ne code pas encore les collectors. Contrainte : code compilable à chaque étape, pas de warnings clippy.
```

---

## Prompt Phase 1 — Snapshot CLI simple

```
Contexte : scaffolding déjà en place (voir structure du repo). Réfère-toi à ARCHITECTURE.md section 5 pour les types SystemSnapshot, CpuInfo, MemoryInfo, DiskInfo, NetworkInfo.

Tâche :
1. Implémente snapshot.rs avec les structs exactement comme spécifiées (derive Debug, Clone, Serialize, Deserialize).
2. Implémente le trait Collector (section 5) et une struct par collector dans src/collectors/ : CpuCollector, MemoryCollector, DiskCollector, NetworkCollector — chacun utilise la crate `sysinfo` pour peupler sa partie du snapshot.
3. Ajoute collectors/mod.rs avec une fonction collect_all(sys: &mut sysinfo::System) -> anyhow::Result<SystemSnapshot> qui orchestre tous les collectors.
4. Implémente le rendu table dans ui/table.rs : affichage lisible en colonnes alignées dans le terminal (pas de dépendance TUI ici, juste du println! formaté proprement).
5. Implémente export.rs avec to_json(&SystemSnapshot) -> anyhow::Result<String>.
6. Branche la commande par défaut (snapshot) et le flag --json dans main.rs.
7. Ajoute un test d'intégration dans tests/snapshot_tests.rs qui lance le binaire avec --json et vérifie que la sortie parse comme JSON valide avec les champs attendus.

Contrainte : `cargo test` doit passer, `cargo clippy` sans warning.
```

---

## Prompt Phase 2 — Top processus

```
Tâche : implémente collectors/processes.rs (ProcessCollector) qui retourne Vec<ProcessInfo> trié par CPU décroissant, limité à N (paramètre).

1. Ajoute le champ top_processes: Vec<ProcessInfo> à SystemSnapshot s'il n'existe pas déjà.
2. Implémente la commande `sysprobe top --by cpu|memory -n 10` : affiche un tableau des N processus triés par le critère choisi.
3. Test unitaire sur la logique de tri (avec des ProcessInfo mockés, pas besoin de vrais processus).

Contrainte : le tri doit être une fonction pure testable indépendamment de la collecte (séparer "récupérer les processus" de "trier et limiter").
```

---

## Prompt Phase 3 — Historique léger

```
Tâche : implémente history.rs avec HistoryBuffer, un ring buffer borné de SystemSnapshot (capacité configurable, voir config.rs section 7 de ARCHITECTURE.md).

1. HistoryBuffer::new(capacity: usize), push(&mut self, snapshot: SystemSnapshot), et une méthode pour récupérer les N derniers points par métrique (ex: cpu_history() -> Vec<f32>) pour alimenter des sparklines plus tard.
2. Quand la capacité est dépassée, le plus ancien point est évincé (comportement ring buffer classique).
3. Ajoute config.rs : struct Config chargée depuis un fichier TOML (voir ARCHITECTURE.md section 7), avec valeurs par défaut si le fichier n'existe pas.
4. Tests unitaires : rotation correcte à capacité dépassée, valeurs par défaut de Config, parsing TOML valide et invalide.

Contrainte : HistoryBuffer ne doit dépendre d'aucune I/O, doit être testable en pur mémoire.
```

---

## Prompt Phase 4 — Dashboard TUI

```
Tâche : implémente le mode `sysprobe watch` avec ratatui + crossterm.

1. ui/dashboard.rs : layout avec au minimum — gauge CPU global, gauge RAM, sparkline historique CPU (via HistoryBuffer), table des top processus.
2. ui/sparkline.rs : wrapper autour du widget Sparkline de ratatui alimenté par HistoryBuffer::cpu_history().
3. Boucle de rafraîchissement basée sur --interval (défaut donné dans config.rs), sortie propre du terminal sur Ctrl+C (restaurer le terminal, ne pas le laisser cassé).
4. Gestion d'erreur : si le terminal ne supporte pas certaines features, fallback propre plutôt que panic.

Contrainte : le TUI ne doit jamais laisser le terminal de l'utilisateur dans un état cassé, même en cas de panic (utiliser un hook de panic qui restaure le terminal avant de re-panic).
```

---

## Prompt Phase 5 — Alertes + daemon

```
Tâche : implémente alert.rs et daemon.rs selon ARCHITECTURE.md section 5 et 6.

1. AlertRule, AlertEvent, AlertEngine avec logique d'hystérésis (sustained_ticks avant de déclencher, et détection de résolution quand la métrique repasse sous le seuil).
2. daemon.rs : boucle tokio avec interval configurable, à chaque tick : collect_all() -> push dans HistoryBuffer -> AlertEngine::evaluate() -> log les AlertEvent via tracing -> écrit le JSON du dernier snapshot dans le chemin configuré.
3. Gestion propre de l'arrêt (Ctrl+C / signal) avec tokio::signal.
4. Tests unitaires sur AlertEngine : déclenchement après sustained_ticks, pas de déclenchement sur un pic isolé, résolution correcte.

Contrainte : le daemon ne doit jamais paniquer sur une erreur de collecte ponctuelle — logger l'erreur et continuer la boucle.
```

---

## Prompt Phase 6 (bonus) — Parsing bas niveau /proc

```
Tâche : implémente collectors/network_proc.rs qui lit directement /proc/net/tcp (Linux) pour compter les connexions actives, sans passer par `sysinfo`.

1. Parse manuellement le format de /proc/net/tcp (hex local_address:port, hex rem_address:port, hex state).
2. Filtre sur l'état ESTABLISHED (0x01) pour compter les connexions actives.
3. Documente en commentaire le format du fichier et les états TCP possibles.
4. Fallback propre si le fichier n'existe pas (macOS/Windows) : retourner une erreur typée claire, pas de panic.
5. Test avec un exemple de contenu /proc/net/tcp fourni en fixture (string statique), pas besoin du vrai fichier système.

Contrainte : ce module ne doit être compilé/actif que sur Linux (cfg(target_os = "linux")), avec un fallback clair ailleurs.
```
