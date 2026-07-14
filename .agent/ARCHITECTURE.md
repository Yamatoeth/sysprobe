# sysprobe — Architecture

Outil CLI Rust de visibilité système en temps réel : CPU, RAM, disque, réseau, processus, historique léger, export JSON, alertes, mode daemon.

Nom de code utilisé dans ce doc : `sysprobe`. Renomme si tu as déjà une idée (ex: `probe`, `vitals`, `pulse`).

---

## 1. Objectifs / non-objectifs

**Objectifs**
- Snapshot instantané du système en une commande (`sysprobe`)
- Dashboard live en TUI (`sysprobe watch`)
- Mode daemon en arrière-plan avec alertes (`sysprobe daemon`)
- Export JSON scriptable (`sysprobe --json`)
- Historique léger en mémoire (ring buffer, pas de DB)
- Cross-platform si possible (Linux prioritaire, macOS bonus), sinon assumer Linux only et le dire clairement dans le README

**Non-objectifs (à ne pas dériver dessus)**
- Pas de stockage persistant complexe (pas de SQLite/Postgres)
- Pas de serveur web / API HTTP (sauf si tu veux un bonus scope-creep après le MVP)
- Pas de clustering / multi-hôte
- Pas de configuration YAML énorme — un seul fichier TOML simple

---

## 2. Stack technique

| Besoin | Crate | Pourquoi |
|---|---|---|
| Infos système cross-platform | `sysinfo` | Évite de réécrire les syscalls à la main pour CPU/RAM/disque/réseau/process, tout en restant "bas niveau" dans son usage |
| CLI parsing | `clap` (derive) | Standard, rapide à mettre en place, sous-commandes propres |
| TUI live | `ratatui` + `crossterm` | Dashboard terminal pour le mode `watch` |
| Async / boucle daemon | `tokio` | Boucle de polling non bloquante, timers, futur bonus réseau |
| Sérialisation | `serde`, `serde_json` | Export JSON |
| Config | `toml` + `serde` | Fichier de config simple (seuils, intervalle) |
| Erreurs | `thiserror` (lib) + `anyhow` (bin) | Erreurs propres sans boilerplate |
| Logs | `tracing` + `tracing-subscriber` | Utile pour le mode daemon |
| Temps | `chrono` | Timestamps humains dans l'historique/export |

Pour montrer "syscalls / OS interfaces" en vrai (et pas juste via `sysinfo`), prévoir **un module optionnel** qui lit directement `/proc` sous Linux pour un ou deux indicateurs (ex: connexions réseau actives via `/proc/net/tcp`). Ça te donne un vrai talking point d'entretien ("j'ai utilisé une lib cross-platform pour l'essentiel, mais j'ai parsé `/proc/net/tcp` moi-même pour les connexions actives parce que X").

---

## 3. Architecture logique

```
                         ┌───────────────┐
                         │     CLI       │  clap: parse args → Command
                         └───────┬───────┘
                                 │
                 ┌───────────────┼────────────────┐
                 ▼               ▼                ▼
           Command::Snapshot Command::Watch  Command::Daemon
                 │               │                │
                 └───────┬───────┴────────┬───────┘
                         ▼                ▼
                  ┌─────────────┐  ┌─────────────┐
                  │ Collectors  │  │  Scheduler  │  (tokio interval)
                  └──────┬──────┘  └──────┬──────┘
                         ▼                ▼
                   SystemSnapshot ──▶ HistoryBuffer (ring buffer)
                         │                │
              ┌──────────┼────────────────┼───────────┐
              ▼          ▼                ▼            ▼
          Renderer   JSON Export    AlertEngine    (futur: metrics sink)
          (table/TUI)                   │
                                         ▼
                                   Notifier (stdout / log / webhook bonus)
```

### Composants clés

- **Collectors** : un trait `Collector` avec une impl par ressource (`CpuCollector`, `MemoryCollector`, `DiskCollector`, `NetworkCollector`, `ProcessCollector`). Chacun produit un morceau de `SystemSnapshot`.
- **SystemSnapshot** : struct sérialisable (serde) qui représente l'état du système à un instant T. C'est le contrat central du projet — tout le reste consomme ou produit ce type.
- **HistoryBuffer** : ring buffer (`VecDeque<SystemSnapshot>` borné, ex: 120 points = 2h à 1 point/min) pour les mini-courbes en TUI et l'export historique.
- **AlertEngine** : évalue chaque nouveau `SystemSnapshot` contre des seuils configurables (`cpu_percent_max`, `mem_percent_max`, `disk_percent_max`), déclenche une alerte si dépassement + hystérésis simple (ne pas re-alerter à chaque tick).
- **Renderer** : deux implémentations — table statique (mode snapshot unique) et TUI live (`ratatui`) pour `watch`.
- **Daemon** : boucle tokio qui poll à intervalle régulier, alimente `HistoryBuffer`, fait tourner `AlertEngine`, et optionnellement écrit un fichier JSON à chaque tick pour que d'autres outils puissent le lire.

---

## 4. Structure de fichiers

```
sysprobe/
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md
├── config/
│   └── sysprobe.example.toml
├── src/
│   ├── main.rs              # entry point, wiring
│   ├── cli.rs                # clap: Cli, Command enum
│   ├── config.rs             # struct Config, load from TOML + defaults
│   ├── error.rs               # AppError (thiserror)
│   ├── snapshot.rs            # struct SystemSnapshot + sous-structs
│   ├── collectors/
│   │   ├── mod.rs             # trait Collector, fn collect_all()
│   │   ├── cpu.rs
│   │   ├── memory.rs
│   │   ├── disk.rs
│   │   ├── network.rs         # via sysinfo
│   │   ├── network_proc.rs    # bonus: parsing /proc/net/tcp direct (Linux)
│   │   └── processes.rs
│   ├── history.rs             # HistoryBuffer (ring buffer + sparkline data)
│   ├── alert.rs               # AlertEngine, AlertRule, AlertEvent
│   ├── export.rs              # to_json(), write_json_file()
│   ├── daemon.rs              # run_daemon() boucle tokio
│   └── ui/
│       ├── mod.rs
│       ├── table.rs            # rendu snapshot unique (mode CLI simple)
│       ├── dashboard.rs        # ratatui: layout, widgets
│       └── sparkline.rs        # mini-courbes ASCII/ratatui
└── tests/
    ├── snapshot_tests.rs
    ├── alert_tests.rs
    └── history_tests.rs
```

---

## 5. Types centraux (à faire scaffolder par Codex tel quel)

```rust
// snapshot.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network: NetworkInfo,
    pub top_processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CpuInfo {
    pub global_usage_percent: f32,
    pub per_core_usage_percent: Vec<f32>,
    pub load_avg_1: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: f32,
    pub swap_used_bytes: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfo {
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub active_connections: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}
```

```rust
// collectors/mod.rs
pub trait Collector {
    type Output;
    fn collect(&mut self, sys: &mut sysinfo::System) -> anyhow::Result<Self::Output>;
}
```

```rust
// alert.rs
pub struct AlertRule {
    pub name: String,
    pub metric: MetricKind,       // Cpu, Memory, Disk(mount)
    pub threshold_percent: f32,
    pub sustained_ticks: u32,     // évite le bruit sur un pic isolé
}

pub enum AlertEvent {
    Triggered { rule_name: String, value: f32, at: chrono::DateTime<chrono::Utc> },
    Resolved { rule_name: String, at: chrono::DateTime<chrono::Utc> },
}
```

---

## 6. CLI (clap)

```
sysprobe                     # snapshot unique, table lisible dans le terminal
sysprobe --json              # snapshot unique, sortie JSON sur stdout
sysprobe watch               # dashboard TUI live (ratatui)
sysprobe watch --interval 2  # refresh toutes les 2s
sysprobe daemon              # tourne en fond, écrit history + alerts
sysprobe daemon --config path/to/sysprobe.toml
sysprobe history --last 1h   # dump de l'historique en mémoire/fichier
sysprobe top --by cpu -n 10  # top processus, tri custom
```

---

## 7. Config (`sysprobe.toml`)

```toml
[general]
refresh_interval_secs = 2
history_capacity = 120

[alerts]
cpu_percent_max = 90.0
mem_percent_max = 90.0
disk_percent_max = 95.0
sustained_ticks = 3

[export]
json_output_path = "./sysprobe_snapshot.json"
```

---

## 8. Stratégie de test

- **Unit** : `AlertEngine` (seuils, hystérésis), `HistoryBuffer` (rotation, capacité), sérialisation `SystemSnapshot` (round-trip JSON).
- **Collectors** : difficile à tester en dur (dépend de la machine) → tester la *forme* des données (bornes 0-100% etc.), pas les valeurs.
- **Integration légère** : lancer `sysprobe --json` en subprocess dans un test et valider que le JSON parse correctement.

---

## 9. Ce qui fait la différence en entretien

- Le module `network_proc.rs` (parsing manuel de `/proc/net/tcp`) → preuve de compréhension bas niveau au-delà d'une lib.
- L'`AlertEngine` avec hystérésis (`sustained_ticks`) → preuve de réflexion produit, pas juste "if x > y".
- Le mode daemon avec écriture JSON périodique → preuve que tu penses "intégration avec d'autres outils", pas juste un gadget CLI.
- Séparer `Collector` (trait) de `Renderer` (trait) → découplage propre, testable, extensible sans réécrire.
