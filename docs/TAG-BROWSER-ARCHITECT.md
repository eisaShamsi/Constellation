# Universe-wide Tag Browser — Architect (lightweight)

**Date:** 2026-05-29
**Trigger:** Boss: *"I want a real universe-wide tag browser."* (Task #12)
**Type:** New feature. Small — reuses existing pieces; the only real decision is placement.

---

## Goal

A **discoverable**, **universe-wide** tag browser: see every tag across the active Universe + all federated cUniverses, with counts, hierarchy, and click-to-find. Today Eisa's only tag surface is the right-sidebar **per-note** Tags tab (shows the open note's tags). The universe-wide tag list that exists (Notes Navigator mode) is undiscoverable and unused.

## The happy accident — almost everything already exists

| Piece | Status |
|---|---|
| **Federated tag data** | `allLibraryTags: Record<string, number>` — already aggregates tag→count across parent + cUniverses (MIG-061 §M + MIG-062 §A). Ready. |
| **A hierarchical tag-browser component** | `src/lib/components/TagsPanel.svelte` — builds a nested `parent/child` tag tree with counts, expand/collapse, and a filter box; takes `tags` + `onTagClick`. **Imported in `+layout.svelte` (line 88) but currently UNUSED** — ready to repurpose. |
| **Click-to-find behavior** | `handleTagClick(tag)` (`+layout.svelte:4055`) sets `searchHubInitialQuery = '#'+tag` and opens Search Hub — which is **federated** (MIG-058/059). So a tag click already does the right thing: federated search for that tag. |

So the feature ≈ render `TagsPanel` fed with `allLibraryTags`, `onTagClick={handleTagClick}`, in a discoverable place. ~30–50 lines. No backend. No new data path.

## Current tag surfaces (for reference)

- **Right-sidebar `'tags'` tab** (`+layout.svelte:6470`): renders `activeNoteTags` as `#chip`s for the OPEN note. Per-note by design. (Does *not* use TagsPanel — inline chips.)
- **Notes Navigator tag mode** (`sidebarMode === 'list'`): universe-wide tag list, but in a sidebar mode Eisa doesn't use / can't find.

## The one decision — placement (Q1)

Where does the universe-wide tag browser live?

- **Option A — Extend the right-sidebar Tags tab** with a toggle: **"This note" ⇄ "All tags"**. "This note" = today's chips; "All tags" = `TagsPanel` fed with `allLibraryTags`. *Pros:* most discoverable (Eisa already found this tab), reuses the unused TagsPanel, one surface for both tag views, tiny. *Cons:* the tab does double duty.
- **Option B — New left-sidebar "Tags" section** (a collapsible section like Five Acts / Bases) showing the all-tags tree always in the file-explorer sidebar. *Pros:* always visible alongside files. *Cons:* competes for left-sidebar vertical space; another always-on section.
- **Option C — New full-page dock Tag Browser** (its own dock button + panel). *Pros:* room to breathe; a "tag dashboard." *Cons:* heaviest; a new dock button for what's essentially a list; overkill for v1.

## Tag-click behavior (Q2 — likely no decision needed)

Reuse `handleTagClick` → opens **federated Search Hub** with `#tag`. Consistent with the per-note chips today; federated by virtue of Search Hub. Unless you want clicking a tag to do something else (filter the tree in place, etc.), this is settled.

## Invariants

- Reuses `allLibraryTags` — no new data path; stays federated automatically.
- Per-note Tags chips behavior unchanged (Option A keeps them as the "This note" view).
- RTL: tag names can be Arabic — `TagsPanel` rows need `dir` handling (verify/patch).
- i18n: the toggle labels ("This note" / "All tags") → 15 locales.

## Recommendation

**Q1 = Option A** (extend the right-sidebar Tags tab with a This-note ⇄ All-tags toggle). Most discoverable since you already use that tab, reuses the ready-made TagsPanel, smallest footprint, and the search-on-click behavior is already consistent there. **Q2 = reuse `handleTagClick`.**

## Plan sketch (if Option A approved)

1. Right-sidebar Tags tab: add a 2-way toggle (`tagView: 'note' | 'all'`, `$state`, default 'note').
2. When 'all': render `<TagsPanel tags={allLibraryTags} onTagClick={handleTagClick} />`. RTL `dir` on rows.
3. i18n: 2 toggle labels × 15 locales.
4. Boss-test: open the Tags tab → flip to "All tags" → see the federated tag tree (cUniverse tags included) → click a tag → federated Search Hub opens with `#tag`.
5. PCS (fold into the next batch; help-doc note in the Federation topic).

## Approval

Pick Q1 (A / B / C). On approval I cascade the plan (it's a single small build + Boss-test).
