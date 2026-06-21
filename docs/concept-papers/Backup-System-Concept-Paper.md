# Backup & Recovery System — Concept Paper (WANTED FEATURE)

> **Boss-wanted, banked 2026-06-21.** Eisa: *"Create a backup system for Constellation. Users will have peace of mind that, in the worst-case scenario, they have a safety net to secure their PKM/PKF."* NOT started — this is the durable capture so the eventual design starts from a real foundation. Build comes AFTER MIG-080 (the finish-MIG-080-first directive) and through the full `/migration` (it touches files, the Universe, settings, Rust fs, a scheduler, and UI). The dev-side **Backup Routine** (`git tag` + `git archive` ZIP in CLAUDE.md) is the *precedent/inspiration*; THIS is the **user-facing** feature.

## 1. The one job
Give the user **peace of mind** that their Universe (notes, links, settings, bases) survives the worst case — accidental deletion, a botched edit, disk failure, a bad sync, corruption — with a **trustworthy, easy restore**. A PKF system holds the user's intellectual life (Five Acts → Conviction); losing it is catastrophic, so the safety net is not optional.

## 2. What it is NOT
- NOT a cloud service Constellation owns or forces. NOT a proprietary backup format. NOT a replacement for the user's own sync (Git/Syncthing/iCloud) — it complements it. NOT a feature that ever modifies the user's live files silently.

## 3. Hard architectural constraints (non-negotiable — these shape every option)
- **File-Over-App.** The backup TARGET is the on-disk `.md` + YAML + LINK files. A backup MUST be restorable **without Constellation** (it's just standard files in a folder). No proprietary container that locks the user in.
- **Local-First.** Must work **fully offline** (a local folder / external drive). Cloud is the user's **choice** (back up *into* their Git/Syncthing/iCloud-synced folder), never a Constellation-owned cloud. No telemetry.
- **Reversibility / archival-not-deletion.** Backups are versioned/append-only; restores never destroy the current state (restore-to-a-side, or trash-backed). Deletes are soft (trash), recoverable.
- **The Living Links are files.** `*_LINK_*.md` files are backed up like notes (source of truth). The **`note_links` / `notes_fts` / `sky_*` SQLite tables are the EPHEMERAL index** — NOT a backup target; they are **rebuilt from the files on restore** (the index is "droppable + rebuildable from note_links at any time"). So: back up the Universe **directory**; regenerate the index after restore.
- **The Universe shape is the unit.** Target = the Universe dir: `universe.json`, `.constellation/libraries.json`, each library's folders/notes, `bases`, bookmarks, the synced settings. `cUniverse` children are **separate Universes** (back each up independently; resolve_libraries_recursive only federates at runtime).
- **Multilingual + RTL** for all UI (×15, native equivalents) per the top-principal.

## 4. Design options (initial framing — to be RESEARCHED then chosen, not yet decided)
1. **Local point-in-time snapshots** to a chosen folder/external drive (the Backup-Routine ZIP idea, made user-facing + automatic) + a **time-machine restore** (browse snapshots → restore a note or the whole Universe).
2. **Git integration** (Constellation already endorses Git as a sync choice): a built-in "snapshot = commit" on a schedule/on-close → versioned history + restore for free. *(Obsidian Git plugin is the precedent.)*
3. **Per-note version history** — keep N prior versions of each note (always-on, lighter; the safety net for a single bad edit). Pairs with the right-click **"Open version history"** item (see `Right-Click-Reference-Obsidian.md`). *(Obsidian "File Recovery" core plugin precedent.)*
4. **Trash / soft-delete** — deleted notes/folders go to a recoverable trash, never hard-deleted (archival-not-deletion).
5. **Scheduled export** — automatic ZIP/copy to a user-chosen location on a cadence.
> Likely the answer is a **combination**: per-note version history + soft-delete trash (everyday safety) **layered with** scheduled snapshots/Git (catastrophe safety). The right split is a design decision.

## 5. Open questions (for the Architect + Boss decisions)
- **Scope:** whole Universe / per-library / per-note / cUniverses?
- **Where:** local folder · external drive · the user's cloud-synced folder · a Constellation-managed location? (Local-First → the user picks; sensible defaults.)
- **Cadence:** on-close · scheduled · on-significant-change · manual?
- **Mechanism:** ZIP archives vs Git vs file-copy snapshots vs per-note version history vs a blend?
- **The index:** exclude (rebuildable, smaller, slower restore) vs include (faster restore, larger)?
- **Encryption:** optional encrypted backups (the Settings → Security precedent)?
- **Retention:** how many snapshots / how long / pruning?
- **Restore UX & granularity:** browse · diff · restore-one-note vs restore-all · a one-click "panic restore."
- **TRUST:** how does the user *verify* a backup is good — a "test restore" / integrity check? (Peace of mind requires provable, not just promised, recoverability.)
- **Performance:** must not regress boot/typing/IPC (a 1.8 GB / 7,600-note Universe); snapshots run off the hot path, resumable.

## 6. Prior art to research first (WA#5 — before any design)
Obsidian (File Recovery core plugin; Obsidian Git; Sync version history), Logseq (Git), Git itself, snapshot tools (Time Machine, restic, Borg, Arq), Syncthing file-versioning, iCloud/Dropbox version history. The proven "files + version history + restore" pattern is most likely **Git or a snapshot tool** — cross-check before inventing.

## 7. Status
WANTED FEATURE · banked 2026-06-21 (Boss) · not started · **after MIG-080** · needs WA#5 research → full `/migration`. See [[project_backup_system_wanted]].
