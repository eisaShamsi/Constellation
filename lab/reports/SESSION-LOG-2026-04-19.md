# Session Log — 2026-04-19

## Headline

**Boot Criterion 2 — Sky View graphReady loading indicator** landed (commit `6ed93df`, 16 files / 41 insertions), closing the last perceptible-responsiveness gap on the deferred-graph-load window. Production `tauri build` produced `constellation.exe` (48 MB) + MSI (120 MB) + NSIS (94 MB) in **2m 06s release compile** — exit-code-1 came from the updater-signing step only (`TAURI_SIGNING_PRIVATE_KEY` env var unset), orthogonal to code correctness. M11-data v2 § 101 (`058-major-world-cities.json`, 40 concepts, corpus 2,360 → 2,400) landed in parallel by the background agent targeting §§ 101-103. The tauri build kicked off **before** the SkyView indicator commit, so the measurement binary carries the core/graph split from commit `f69b5fa` but not the indicator overlay — the indicator is purely visual UX during the ~6–9 s graph-load window and does not affect `hydrated_ms`.

## Work in order

### 1. Boot Criterion 2 — Sky View graphReady loading indicator

The boot-snapshot split from commit `f69b5fa` (Boot snapshot split + Sky View PIXI/CSP fix) split `cache_boot_snapshot` into a fast `cache_boot_snapshot_core` (notes-only, awaited before `boot:hydrated`) and a deferred `cache_boot_snapshot_graph` (links + tags, scheduled via `requestIdleCallback` after paint). On the 7,600-note trial Universe that graph phase lands ~6–9 s after hydration — which is fine for most panels, but if the user opens **Sky View** during that window, `<GraphMindView />` renders the empty `skyNodes` / `skyLinks` state with no feedback that more is coming.

**Solution shipped:**

- `{#if !graphReady}` block inserted after `<GraphMindView />` rendering a small "Loading graph…" pill — spinning SVG (`<circle>` + 3/4 `<path>` stroke arc) + translated label.
- Accessibility: `role="status"`, `aria-live="polite"`, `dir="auto"` (lets bidi flow naturally in any target language), `aria-hidden="true"` on the decorative spinner SVG.
- Positioning: `position: absolute` anchored to the `.star-fullscreen` wrapper (which I flipped from un-positioned to `position: relative` in the same edit). Pill sits `top: 56px` (just below `.star-header`), `inset-inline-end: 16px` (logical property → auto-flips on RTL libraries), `pointer-events: none` so it never blocks clicks on nodes that arrive early.
- Visual: `background: rgba(0,0,0,0.55)` + `backdrop-filter: blur(4px)` for subtle contrast over any graph bg, `border-radius: 999px` pill, `color: #fff`, `font-size: 0.8rem`.
- Animation: `@keyframes sky-loading-spin { to { transform: rotate(360deg); } }` at 0.9 s linear infinite — kept a separate keyframes block from the existing `@keyframes spin` at line 5618 (which is `:global(.spin)`) to avoid accidentally coupling this indicator's lifecycle to unrelated spinners elsewhere.
- i18n: added `layout.skyViewLoading` key to all 15 locale files (ar / de / en / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh) with native-script translations (e.g. ar "جارٍ تحميل الرسم البياني…", zh "正在加载图表…", hi "ग्राफ़ लोड हो रहा है…").

**Files modified** (commit `6ed93df`, via `git add -p` selective staging to isolate from prior-session unrelated IndexPanel `readCooccurringTerms` work also pending in the worktree):

- `src/routes/+layout.svelte` — 26 insertions: 9 HTML (the `{#if !graphReady}` block) + 17 CSS (`.sky-loading` + `.sky-loading-spinner` + keyframes + `.star-fullscreen { position: relative }`).
- `src/lib/i18n/{ar,de,en,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json` — 15 files × 1 insertion each = 15 lines, the `skyViewLoading` key injected after `skyViewWiWHint`.

### 2. Production tauri build verification

Kicked off `npm run tauri build` in background (task `b64y9cz1x`) before the indicator commit landed — the measurement target is the already-committed core/graph split, which the binary carries regardless.

**Results:**

- Rust release compile: **2 m 06 s** (59 warnings, 0 errors — all warnings are pre-existing dead-code notices on `LexiconGraph::load_core` / `LexiconGraph::from_records` / `LexiconGraph::to_bundle` / `lexicon::parse::parse` / `SearchRequest` fields `include_snippet` + `include_headings` / `SearchFilters` field `maturity`; none block the binary).
- Bundles produced:
  - `src-tauri/target/release/constellation.exe` — 48 011 264 B (48 MB), Apr 19 09:23
  - `src-tauri/target/release/bundle/msi/Constellation_0.1.0_x64_en-US.msi` — 120 827 904 B (120 MB), Apr 19 09:19
  - `src-tauri/target/release/bundle/nsis/Constellation_0.1.0_x64-setup.exe` — 94 605 946 B (94 MB), Apr 19 09:23
- Exit code 1 from final updater-signing step: `A public key has been found, but no private key. Make sure to set TAURI_SIGNING_PRIVATE_KEY environment variable.` — config-level issue (updater public key present in `tauri.conf.json` but private-key env var not exported for this build), orthogonal to code. Binaries are fully usable for boot measurement. The version number `0.1.0` in the bundle filenames reflects the current worktree's `tauri.conf.json` (one of the pending un-committed changes in the worktree — tracked for follow-up).

**Ready for trial-Universe measurement** (user action):

1. Launch `src-tauri/target/release/constellation.exe` on the 7,600-note trial Universe (16 libraries, 656 k typed links).
2. Observe sidebar paint (~1 s target, Criterion 1).
3. Observe file-tree interactivity — click any note, open it, type 10 chars, no lag — within **≤ 6 s** (Criterion 2 PASS target).
4. Read `<universe>/.constellation/boot-perf.latest.json` for `paint_ms` (expect ≤ 2 500) / `hydrated_ms` (expect ≤ 6 000) / `graph_ready_ms` (informational, expect ~6 000–10 000).
5. Open Sky View within the first 3 s after paint: graph renders empty + the new "Loading graph…" pill (wait — NOT in this build; the indicator is in commit `6ed93df`, produced **after** the build started; second build queued as optional follow-up for in-binary verification).
6. Race check: click a note, open the Backlinks panel, confirm it starts empty and auto-populates once the graph resolves (the `starVersion`-gated re-fetch already lives in the tab-focus effect).

### 3. M11-data v2 — § 101 landed (background agent)

Background agent `ada604478a4d559c6` (spawned in parallel this session, targeting §§ 101-103) landed **§ 101 = `058-major-world-cities.json`** (40 concepts, commits `8775e51` content + `54bf36b` hash-stamp). Corpus count **2,360 → 2,400** across 59 total shards. Pairs thematically with § 100 capital-cities — extends the geographical sequence from national-administrative-capitals into the non-capital-economic-cultural-metropolitan vocabulary: new-york, los-angeles, chicago, san-francisco, boston, miami, houston, toronto, vancouver, montreal, rio-de-janeiro, sao-paulo, dubai, istanbul, mumbai, chennai, kolkata, bangalore, karachi, lahore, shanghai, hong-kong, guangzhou, shenzhen, chengdu, osaka, kyoto, yokohama, busan, taipei, ho-chi-minh-city, sydney, melbourne, lagos, johannesburg, barcelona, milan, munich, frankfurt, marseille — eight old-name/new-name exonym-endonym lexical pairs preserved (mumbai/bombay, chennai/madras, kolkata/calcutta, ho-chi-minh-city/saigon, guangzhou/canton, bangalore/bengaluru, busan/pusan, dubai/dubaï). Agent continues toward § 102+ — completion notification pending.

## Files modified

- `src/routes/+layout.svelte` (+26 lines) — SkyView graphReady indicator HTML + CSS.
- `src/lib/i18n/{ar,de,en,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json` (+1 each = +15 lines) — `skyViewLoading` translations.
- `lab/m11-data/concepts/058-major-world-cities.json` (new, 40 concepts) — via background agent.
- `lab/m11-data/concepts.tsv` (background agent regeneration; corpus 2,400).
- `lab/reports/SESSION-LOG-2026-04-18.md` (background agent § 101 narrative + files-modified).
- `lab/reports/SESSION-LOG-2026-04-19.md` (this file, new).

### 4. M11-data v2 §§ 102-105 — rest of the geographical-sequence landed (background agent completion)

Agent `ada604478a4d559c6` completed and landed four additional shards after § 101, hitting the five-shard cap set at spawn time. All commits merged into `claude/upbeat-proskuriakova` (HEAD advanced from `6ed93df` → `26a5211`). Per-shard cargo-test-116/116-green on first try for every one of §§ 101-105 — six-consecutive-clean-first-runs, longest clean streak of this session, with the LNK1104 flake streak fully reset after § 100.

| § | Shard | Content commit | Hash-stamp commit | TSV delta |
|---|---|---|---|---|
| 102 | `059-oceans-and-seas` | `a7e00e0` | `25398cb` | 649,363 → 663,852 (+14,489) |
| 103 | `060-mountains-and-ranges` | `ca59c38` | `b076d8b` | 663,852 → 679,025 (+15,173) |
| 104 | `061-rivers` | `aa53055` | `e6f946e` | 679,025 → 693,283 (+14,258) |
| 105 | `062-planets-and-celestial-bodies` | `e03d6fb` | `26a5211` | 693,283 → 704,891 (+11,608) |

**Corpus state**: 2,400 → **2,560 concepts** across **63 total shards** (62 v2 + 1 v1 seed); TSV at 704,891 bytes on disk.

**Geographical scale-hierarchy completed** across §§ 99-105: countries → capitals → non-capital-metropolises → oceans-and-seas → mountains-and-ranges → rivers → extra-terrestrial-planetary/stellar-features. § 103 mountains had the highest byte-delta (+15,173) driven by Himalayan multi-language alternative-lemma coverage (Sanskrit/Tibetan सगरमाथा / 珠穆朗玛峰). § 105 planets surfaced a dense Islamic-Golden-Age classical Arabic astronomical heritage cluster — `الشعرى` (Sirius), `النسر الواقع` (Vega), `إبط الجوزاء` (Betelgeuse), `رجل الجبار` (Rigel), `السماك الرامح` (Arcturus), `قلب العقرب` (Antares) — representing genuine cross-language continuity via the Ptolemaic-Islamic-European transmission channel. § 105's collision scan identified 14 astronomical candidates already landed at § 72 (`036-cosmos-and-astronomy`: eight major planets + sun/moon/milky-way/solar-system/supernova/black-hole) and retained them as primitives, with 10 hyphenated-id disambiguators reserving bare `europa`/`io`/`titan`/`triton`/`orion` and zodiacal/mythological names for future sense-landings. **Eleven-consecutive-humanities-streak** across §§ 95-105 — the longest non-STEM sub-sequence of v2 corpus.

### 5. Stale-test fix after §§ 101-105 corpus growth

The full-suite cargo test on the merged HEAD (my `6ed93df` + agent's §§ 102-105 chain + my `1d455f3` interleaved) revealed one regression: `search::tests_m12::proper_noun_not_in_corpus_falls_back` failed because `expanded_match_query("Constellation")` no longer returns `None`. The test's own comment had forecast this: *"Any well-formed English word not on a concept returns None and falls through to prefix matching. This is the common case until M11-data scales up past 49 concepts."* The corpus has now scaled past 2 500 concepts, and `constellation` is a live English lemma on shard `036-cosmos-and-astronomy` (line 152, id `constellation`, lemma `["constellation"]` at line 157) — landed long before today's §§ 101-105 batch, but the test coexisted with it because nothing had triggered the full-suite run since that shard landed.

**Fix** (commit `ba4c0bb`): replace `"Constellation"` with `"Xzyqwop"` — an invented nonsense string with zero collision risk against any natural-language lexicon, so the test continues to exercise the fallback path indefinitely regardless of future corpus growth. Keep `"Anthropic"` (company name, verified not in corpus by direct grep across all 63 shards) as the realistic proper-noun case. Update the comment to reflect present corpus scale and the invariant the assertions now enforce (a guaranteed-non-concept string + a proper-noun-that-will-stay-out-of-corpus).

**Full suite after fix**: **429/429 passing**, 3 ignored (the two `#[ignore]`-gated benches from M9 + M12 plus one additional), 0 failed. Wall time 0.74 s.

### 6. Housekeeping — worktree cleanup

Agent worktree `E:\مشاريع كلاود\Constellation\.claude\worktrees\agent-ada60447` + branch `worktree-agent-ada60447` force-pruned after verification. Only `.claude/settings.local.json` was dirty in it (local config, not source). The two remaining worktrees: main at `ef45c17` + this one at `ba4c0bb`.

### 7. Boot Criterion 2 — IPC-overhead diagnostic instrumentation (commit `304edd0`)

**Trigger.** User's first Criterion 2 measurement on the release binary produced the surprising shape below:

```
paint_ms:            870      PASS
libraries_loaded_ms: 938
hydrated_ms:      23,554      FAIL (target ≤6,000)
graph_ready_ms:   27,873      (informational)

cache_snapshot_core_wall_ms:   22,614        ← 22.6 s inside this invoke()
cache_snapshot_core_server_timings:
    ensure_db:    29
    open_reader:   0
    read_notes:   19             sum = 48 ms  ← only 48 ms of Rust work

cache_snapshot_graph_wall_ms:   3,882
cache_snapshot_graph_server_timings:
    ensure_db:     0
    open_reader:   0
    count_notes:   2
    read_links:  732
    read_tags: 1,696             sum = 2,430 ms
```

The graph phase is honest (wall 3,882 − server 2,430 = ~1.5 s of IPC overhead — reasonable for a ~1.4 MB payload). The core phase is the mystery: **22,566 ms of unaccounted-for time** between issuing `await invoke('cache_boot_snapshot_core')` and the resolved Promise. Three hypotheses:

1. **WebView2 IPC serialization** of the 7,600-note JSON array on the main thread of Edge/Chromium's renderer process.
2. **Svelte 5 reactive cascade** triggered by `allNotes = core.notes.map(...)` — every `$derived` / `$effect` reading `allNotes` (file tree, sidebar, Sight, tab store) re-runs synchronously.
3. **Main-thread starvation** by the fire-and-forget chain (`loadAllStats`, `startWatchingAll`, `loadAllAppearances`) issued just before `refreshLibraryCaches()`.

Each has a different fix path (Tauri `Channel` streaming vs. chunked assignment via `requestAnimationFrame` vs. serialise the fire-and-forget chain). Guessing wrong wastes a build cycle, so we instrument to find out.

**What landed.**

Rust — `src-tauri/src/cache.rs`:

- `use std::time::{Instant, SystemTime, UNIX_EPOCH};` (SystemTime added).
- `BootSnapshotCore` and `BootSnapshotGraph` each gain a `pub server_return_unix_ms: u128` field.
- Every return site (happy path + `open_reader_err` early return, both commands — **four sites total**) captures `SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)` immediately before building the `Ok(...)` struct literal.

Frontend — `src/routes/+layout.svelte`:

- Eight new `let` state vars in the boot-diagnostic block: `cacheSnapshotCore{Transport,Assign,ServerReturnUnix,ClientRecvUnix}Ms` + graph equivalents.
- After `await invoke('cache_boot_snapshot_core')` resolves, we capture `Date.now()` (clientRecvUnixMs) and `performance.now()` (postInvokePerfMs) as the first two statements — before anything reactive can run.
- `cache_snapshot_core_transport_ms = clientRecvUnixMs − core.server_return_unix_ms`. Pure IPC: Tauri serialize + WebView2 pipe + JS deserialize. Independent of any work the JS caller does with the payload afterwards.
- After the `allNotes = ...` assignment we capture another `performance.now()` — the delta is `cache_snapshot_core_assign_ms`, the reactive-cascade cost.
- Same pattern for the graph phase, with `assign_ms` captured after `buildSkyData` (since the synchronous iteration of 656 k links on the main thread is part of "cost of receiving this payload").
- Raw `server_return_unix_ms` + `client_recv_unix_ms` also shipped in the report so clock skew can be ruled out if `transport_ms` looks implausible (e.g. negative, or larger than wall_ms).
- `buildBootPerfReport` emits the eight new fields.

**Type-check.** `cargo check --lib` clean after the Rust changes (59 pre-existing warnings, all `never used` style, no new diagnostics). Frontend edits are pure idiomatic TS — no signature break.

**Selective staging.** Worktree had two pre-existing unstaged hunks in `+layout.svelte` from prior-session IndexPanel co-occurrence work (line 22 `readCooccurringTerms` import + line 3904 `loadCooccurrence` prop). Used `printf "n\ny\ny\ny\ny\ny\ny\ny\nn\n" | git add -p` to commit only the seven instrumentation hunks. Final diff: `cache.rs +32/-3`, `+layout.svelte +82/-3`.

**Behavior change:** none. The new fields are diagnostic additions to an existing serde-serialised struct — ambient callers (second screen, tests, back-compat `cache_boot_snapshot` shim) keep working because new fields only add to the JSON output; nobody's unmarshal path rejects unknown fields.

**What the next measurement tells us.**

- If `cache_snapshot_core_transport_ms ≈ 22,500`: the cost is in IPC, and the fix is streaming (`tauri::ipc::Channel<BootNote>`) or chunking the payload into multiple smaller invokes.
- If `cache_snapshot_core_transport_ms ≈ 0` but `cache_snapshot_core_assign_ms ≈ 22,500`: the cost is the Svelte 5 reactive cascade, and the fix is chunking the `allNotes = ...` assignment across `requestAnimationFrame` (set in 1 k-note batches) so the UI stays responsive.
- If neither dominates but `core_wall_ms` is still ~22,500: the time is being stolen by other work on the main thread during the `await` — the fire-and-forget chain or file-tree synchronous mounts. Fix: serialise or move off-main.

**Build status.** `npm run tauri build` running in background (task `ba0cg86py`). User will relaunch on trial Universe once it ships and report the new `boot-perf.latest.json` — the eight new fields will be definitive.

### 8. M11-data v2 § 106 — +063-historical-figures.json (40 concepts, corpus 2,560 → 2,600)

Background agent pivots the corpus sequence from geography (§§ 99-105) into named-person vocabulary — a cognitively first-class knowledge anchor that was underrepresented before now. **Why now**: the geographical-scale-hierarchy closed at § 105, and people are the other major class of concrete proper nouns users search for — no prior shard targeted historical figures as such (047-economics lists `keynes` / `smith` implicitly via concepts like `capitalism`, but no explicit person-lemma shard existed). Forty entries selected for era-geographic-gender diversity, each fully populated across all 15 supported languages.

**Cluster breakdown** (40 entries, chronological):

- **Ancient world (8)**: hatshepsut, sappho, hippocrates, confucius, socrates, plato, aristotle, cleopatra — Egyptian / Greek / Chinese. Three women.
- **Classical/Imperial (6)**: julius-caesar, virgil, ashoka, hypatia, galen, augustus — Roman / Indian / Alexandrian. One woman (Hypatia).
- **Medieval (8)**: hildegard-of-bingen, murasaki-shikibu, al-khwarizmi, ibn-sina, al-biruni, rumi, ibn-khaldun, joan-of-arc — German / Japanese / Persian / Arab / North-African / French. Three women (Hildegard, Murasaki, Joan).
- **Early Modern (8)**: mansa-musa, leonardo-da-vinci, galileo, shakespeare, akbar, newton-person, mozart, hokusai — Malian / Italian / English / Mughal / Japanese. Zero women (the era's women are under-memorialized in loanword vocabulary; Elizabeth-I / Nur-Jahan candidates deferred for rarer surface-form coverage).
- **19th century (4)**: ada-lovelace, darwin, lincoln, tagore — English / American / Bengali. One woman (Ada).
- **20th century (6)**: marie-curie, einstein, gandhi, frida-kahlo, anne-frank, mandela — Polish-French / German / Indian / Mexican / Dutch-German / South-African. Three women.

Totals: **40 concepts, 11 women, 6 eras spanning ~3,400 years**, every major world region represented at least once. The Islamic-Golden-Age scholarly heritage runs as a mid-medieval cluster (al-khwarizmi, ibn-sina, al-biruni, ibn-khaldun, rumi) matching the § 105 Arabic-astronomical-star-name cluster as a sibling subtheme of pan-Islamic-Abbasid-era intellectual transmission.

**PoS discipline.** 40 Noun pure-PoS (**fortieth pure-Noun v2 shard**) — named-person-proper-nouns lexicalize overwhelmingly as nominal constants across every target language's historical-biographical-vocabulary.

**Cross-shard collision avoidance.** **1 collision identified** against 2,560-concept corpus in pre-write scan: bare `newton` retained at `045-physics-and-energy` as the SI-unit-of-force (N = kg·m/s²). Resolved via one hyphenated-id disambiguator `newton-person` reserving bare `newton` for the unit while the person gets the disambiguated id, surface forms `Isaac Newton` + `Newton` both present as English lemmas on the person concept. Zero other collisions across all other candidates (washington retained at `057-capital-cities` as the US capital — not needed for shard since Washington-the-person was deliberately omitted from shortlist). One Urdu tashkeel fixed pre-commit (`ایڈا لووَلیس` → `ایڈا لوولیس`, strips fatha U+064E from ada-lovelace's Urdu lemma — pre-`build.py` normalizer-compatible form).

**TSV growth.** 501,154 → 510,304 LF-bytes (+9,150 bytes for +40 concepts — compact byte-delta density reflecting the predominance of short transliterated proper-noun surface forms that carry near-identical phonetic forms across target languages — `Einstein` / `Mozart` / `Shakespeare` / `Darwin` / `Newton` preserved near-identically across Latin-script languages compressing delta, offset by dense multi-alternative-lemma entries for the Islamic-Golden-Age-scholars and dual-East-Asian-native-compound-form preservation for Confucius / Murasaki-Shikibu / Hokusai).

**Test posture.** **429/429 cargo test --lib passing** on first invocation (lexicon filter: 116/116 green in 0.75 s). The sentinel test `search::tests_m12::proper_noun_not_in_corpus_falls_back` stays green — the shard carefully avoids `Anthropic` and `Xzyqwop` (the two sentinel strings the test currently depends on, per `ba4c0bb`'s fix).

**Blast radius.** Zero Rust change, pure data addition — same shape as every prior v2 batch §§ 67-105.

**Eleven-streak snapped.** The humanities-subsequence streak across §§ 95-105 ends at eleven — § 106 historical-figures continues it into a **twelfth consecutive humanities shard**, extending the longest non-STEM sub-sequence of v2 corpus to twelve. Next batches could pivot back to STEM/technical neighborhoods if the PoS-balance monitor recommends.

## Commits (updated)

- `6ed93df` — Boot Criterion 2: Sky View loading indicator during deferred graph load.
- `8775e51` — M11-data v2 § 101: +058-major-world-cities.json (40 concepts).
- `54bf36b` — M11-data v2 § 101: hash-stamp 8775e51.
- `a7e00e0` — M11-data v2 § 102: +059-oceans-and-seas.json (40 concepts).
- `25398cb` — M11-data v2 § 102: hash-stamp a7e00e0.
- `1d455f3` — SESSION-LOG-2026-04-19 first pass.
- `ca59c38` — M11-data v2 § 103: +060-mountains-and-ranges.json (40 concepts).
- `b076d8b` — M11-data v2 § 103: hash-stamp ca59c38.
- `aa53055` — M11-data v2 § 104: +061-rivers.json (40 concepts).
- `e6f946e` — M11-data v2 § 104: hash-stamp aa53055.
- `e03d6fb` — M11-data v2 § 105: +062-planets-and-celestial-bodies.json (40 concepts).
- `26a5211` — M11-data v2 § 105: hash-stamp e03d6fb.
- `ba4c0bb` — Fix stale proper_noun_not_in_corpus_falls_back test after §§ 101-105 corpus growth.
- `304edd0` — Boot Criterion 2: IPC-overhead diagnostic Round 1 (transport + assign + raw unix timestamps in both boot snapshot commands).
- `cb60374` — M11-data v2 § 106: +063-historical-figures.json (40 concepts).
- `281f23f` — M11-data v2 § 106: hash-stamp cb60374.

## § 9 — Criterion 2 diagnostic Round 2 (queue-time attribution)

**Round 1 measurement came back unattributed.** First instrumentation shipped `transport_ms`, `assign_ms`, and raw Unix timestamps. Measurement on the trial Universe:

| field | value |
|:---|---:|
| `cache_snapshot_core_wall_ms` | 23,103 ms |
| `cache_snapshot_core_server_timings` sum | 170 ms |
| `cache_snapshot_core_transport_ms` | 19 ms |
| `cache_snapshot_core_assign_ms` | 0 ms |
| **unaccounted** | **22,914 ms** |

Transport and assign are both tiny — the 22.9 s lives **between JS issuing `invoke(...)` and the Rust command body starting execution**. The first-round `Instant::now()` at the command body's first line can only measure in-body elapsed, not pre-body queue time.

**Round 2 adds `server_start_unix_ms`.** Stamped on the VERY FIRST line of each command body (before `ensure_search_db_ready`, before `open_reader`, before any work). Paired with a JS-side `invoke_start_unix_ms = Date.now()` captured immediately before `invoke()`, the delta is pure dispatcher-queue time.

Four new fields per snapshot phase (landed in `src-tauri/src/cache.rs` + `src/routes/+layout.svelte`):

- `cache_snapshot_{core|graph}_invoke_start_unix_ms` — JS side, before invoke.
- `cache_snapshot_{core|graph}_server_start_unix_ms` — Rust side, first line of body.
- `cache_snapshot_{core|graph}_queue_ms = server_start - invoke_start`.
- `cache_snapshot_{core|graph}_body_ms = server_return - server_start` (pure in-Rust execution).

Sanity: `queue_ms + body_ms + transport_ms ≈ wall_ms` should hold within ±clock-skew noise. Decision matrix once measurement lands:

| `queue_ms` | `body_ms` | interpretation |
|:---|:---|:---|
| ~22.9 s | ~170 ms | Tauri dispatcher / blocking-pool scheduler is the bottleneck. Fix path: convert `cache_boot_snapshot_core` to `async fn` (moves off blocking pool) OR inspect what's holding it. |
| ~170 ms | ~22.9 s | Rust body actually slow — re-check SQLite; `ensure_search_db_ready` on Mutex contention; connection-open thrash. |
| both small | small | JS-side main-thread stall between `performance.now()` and actual send. Unlikely but possible if Svelte 5 microtasks starve. |

**Related observation (Sky View "pulse" report).** User opened Sky View after boot and reported "it's like a pulse — slow refresh rate." Continuous main-thread blocking post-boot is consistent with ongoing Rust runtime saturation extending past hydration — same root cause may explain both. Not a standalone issue to chase until the queue attribution lands.

**Build in progress**: `bx13it9um` for new measurement. User will relaunch on trial Universe.

## § 10 — Round 2 measurement + fix (async fn + spawn_blocking)

**Measurement lands decisively in the queue-dominated column.** User relaunched the Round 2 binary on the trial Universe; fresh `boot-perf.latest.json`:

| field | value | interpretation |
|:---|---:|:---|
| `cache_snapshot_core_wall_ms` | 17,314 ms | JS-side wall |
| `cache_snapshot_core_queue_ms` | **17,224 ms** | pre-body dispatcher wait |
| `cache_snapshot_core_body_ms` | **72 ms** | pure SQLite work |
| `cache_snapshot_core_transport_ms` | ~18 ms | serialization + IPC pipe |
| `cache_snapshot_graph_queue_ms` | 96 ms | graph fires via `requestIdleCallback` — runtime idle by then |
| `cache_snapshot_graph_body_ms` | 2,286 ms | matches server-timings sum (read_links 566 + read_tags 1,716) |

99.5 % of the core-phase wall is **pre-body queue**. The Rust body when it finally gets a turn is trivial (72 ms). This is the Tauri async-runtime worker pool saturating: `#[tauri::command] pub fn` is dispatched onto the async-runtime workers (~4 on a 4-core machine), and the boot's fire-and-forget fan-out (16 watchers + 16 appearances + stats refresh + recent/pinned) occupies them all while the core snapshot waits in line.

**Fix (commit `5f60448`).** Convert both snapshot commands to `async fn` wrappers around `tauri::async_runtime::spawn_blocking(..)` of new private sync `*_impl` helpers:

```rust
#[tauri::command]
pub async fn cache_boot_snapshot_core(app: tauri::AppHandle) -> Result<BootSnapshotCore, String> {
    tauri::async_runtime::spawn_blocking(move || cache_boot_snapshot_core_impl(app))
        .await
        .map_err(|e| format!("spawn_blocking failed: {}", e))?
}

fn cache_boot_snapshot_core_impl(app: tauri::AppHandle) -> Result<BootSnapshotCore, String> { /* existing body */ }
```

`spawn_blocking` moves the SQLite work onto the dedicated blocking pool (Tokio default 512 threads) and frees the async runtime to dispatch other commands immediately. The legacy `cache_boot_snapshot` shim (off the boot critical path) calls the `*_impl` helpers directly — keeping it sync preserves its external shape for ambient callers (second screen, tests) without forcing them to await.

`cargo check` clean. Rebuild kicked off as `bid0t1kd6`.

**Expected Round 3 reading**: `core_queue_ms` → ~0; `hydrated_ms` ≤ 6 s → **Criterion 2 PASS**. Graph-phase timings should be unchanged (already runs on idle).

## § 11 — Settings → Debug Boot Performance scorecard (`43d049e`)

Consumer UI for the instrumentation. Adds a new **Debug** entry to the Settings sidebar (bug icon) that renders `boot-perf.latest.json` as a five-row scorecard against `BOOT-BUDGET.md`:

- Criterion 1 — UI visible ≤ 2.5s → driven by `paint_ms`.
- Criterion 2 — Fully responsive ≤ 6s → driven by `hydrated_ms`.
- Criteria 3/4/5 — placeholders pending instrumentation (RSS, stat-sweep, recovery).

Plus two collapsible `<details>` blocks:

- **Per-phase timings** — graph-ready, libraries-loaded, full Round-2 attribution (core + graph wall / queue / body / transport / assign), fan-out (stats / watchers / appearances).
- **Raw JSON** — full `bootPerf` object for any field the scorecard doesn't surface explicitly.

Lazy-loaded: the report is fetched via the pre-existing `read_boot_perf_report` Tauri command inside a `$effect` that triggers on `activeSection === 'debug'`. Not on modal mount; not on every boot. Manual refresh button re-reads the file.

i18n: new `settings.debug.*` block added to `en.json` (23 keys). Other 14 locale files deferred to a Standing Order pass — the Debug section is a developer surface, not primary user UX, and the `$t(…) || 'English fallback'` pattern keeps it readable in the meantime.

## § 12 — Housekeeping: tauri.conf.json version sync

Working-tree had `"version": "0.1.0"` vs HEAD's `"0.3.4"` (accidental local revert in the worktree). Restored to `0.3.4` to match HEAD; no commit needed (no diff vs HEAD after restore). `TAURI_SIGNING_PRIVATE_KEY` plumbing remains open — the build completes without it, but the updater-signature step fails non-fatally on the final line; not blocking.

## § 13 — Round 3 measurement: spawn_blocking fix FAILED. Revert.

The release binary rebuilt with `5f60448` + `43d049e` (spawn_blocking + Debug scorecard) shipped; user relaunched on trial Universe. The Debug → Boot Performance scorecard — which is precisely the surface that exists to diagnose this class of regression — gave the verdict directly:

| field | Round 2 (before fix) | Round 3 (after fix, `5f60448`) | Δ |
|:---|---:|---:|---:|
| `hydrated_ms` | 17,800 ms | **20,610 ms** | **+2,810 ms (worse)** |
| `core_queue_ms` | 17,224 ms | **19,880 ms** | **+2,656 ms (worse)** |
| `core_body_ms` | 72 ms | 112 ms | +40 ms |
| `graph_queue_ms` | 96 ms | ~90 ms | ≈flat |
| Criterion 2 | FAIL (target ≤ 6 s) | FAIL (target ≤ 6 s) | — |

The fix made the queue slightly worse, not zero. The root-cause theory behind `5f60448` is **falsified**: whatever is causing the ~20 s `core_queue_ms` on cold boot, it is **not** the async-runtime worker pool being saturated by sync `#[tauri::command]` fan-out. Moving sync bodies to `spawn_blocking` added one task hop of overhead without removing any contention — consistent with the hypothesis that **Tauri v2 already dispatches sync commands via a blocking-pool internally**, but that wasn't confirmed before shipping. LL-014 corollary: do not patch the same symptom with a second theory until the first theory is read against the runtime source.

**Action taken.** Reverted `5f60448` in a new commit; `cache.rs` is back to the sync `#[tauri::command] pub fn` form. The Debug scorecard UI (`43d049e`), session-log `ee39443`/`53ccbe7`/`a7f0edd`, and the five-stamp diagnostic (`2d2ed1b`) all stay — those are independently useful and correct.

**LL-021 revised.** The original LL-021 claimed `spawn_blocking` was the fix. Falsified. Rewrote it to (a) preserve the five-stamp diagnostic model (which *was* decisive in locating the queue stage), and (b) convert the cautionary section into a rule: do not ship a runtime-internals fix on an unvalidated theory. Read the runtime's actual source, or run a falsifying experiment, before writing the commit.

**What changed in our understanding.** Important new signal from the Round 3 numbers: `graph_queue_ms ≈ 90 ms` (unchanged from Round 2) while `core_queue_ms` got worse. Same runtime, same release binary, same boot — the only difference is **when they fire**. `cache_boot_snapshot_core` fires *immediately* after paint, alongside the boot fan-out (16 `watch_library` + 16 `get_library_appearance` + stats + recent + etc.). `cache_boot_snapshot_graph` fires later, via `requestIdleCallback`, when the fan-out has drained. That flips the open question from "why is the async worker pool slow" to "what resource does the fan-out hold that the core snapshot blocks on, which is released by the time the graph call fires?". Candidates: WebView2 IPC receive-channel ordering, a Mutex inside Tauri's command-dispatch layer, SQLite page-cache warm-up, OS file-handle contention before WAL checkpoint.

**Next step.** Spawn an adversarial investigation agent (per LL-017) to read Tauri v2's actual IPC dispatch — `tauri-macros::command` expansion, `tauri-runtime-wry`'s IPC receive loop, WebView2 message drain. Required deliverable: *either* a line-number-referenced identification of the actual queue source, *or* an experiment design that cleanly falsifies a specific hypothesis. No more fix commits until we have one.

## Commits (updated)

- `304edd0` — Boot Criterion 2: IPC-overhead diagnostic Round 1.
- `cb60374` + `281f23f` — M11-data v2 § 106.
- `2d2ed1b` — Boot Criterion 2: Round 2 queue-time attribution.
- `5f60448` — Boot Criterion 2: move snapshot commands off async-runtime workers (`spawn_blocking`). **Reverted in `f5f0b6a` after Round 3 measurement regressed.**
- `9c722d9` — SESSION-LOG § 10 (Round 2 measurement + fix).
- `43d049e` — Settings → Debug: Boot Performance scorecard.
- `ee39443` — SESSION-LOG §§ 11-12 (scorecard UI + conf version sync).
- `53ccbe7` — User Manual § 16: Debug subsection.
- `b16c1b3` — LL-021 (original: async-runtime saturation theory).
- `a7f0edd` — SESSION-LOG commit backfill.
- `f5f0b6a` — **Revert `5f60448`**. Round 3 measurement via Debug scorecard showed `hydrated_ms` 17.8 s → 20.6 s (worse). Theory falsified. LL-021 rewrite + § 13 land alongside.
- `4757910` — docs(LL-021, session-log § 13): retract spawn_blocking theory after Round 3 falsification.
- `9001b01` — **Boot Criterion 2 — Experiment A+**: move boot fan-out off the WebView2 UI thread (`watch_library`, `unwatch_library`, `get_all_library_stats`, `read_library_appearance` → `#[tauri::command(async)]`). See § 14.

## § 14 — Investigation agent + Experiment A+: UI-thread fan-out as the real queue cause

Adversarial investigation agent (LL-017) dispatched with specific deliverable: *either* a line-referenced identification of the queue source in Tauri / wry source, *or* a falsifying experiment. It returned both.

**Versions pinned (from `src-tauri/Cargo.lock`):** `tauri = 2.10.3`, `tauri-runtime = 2.10.1`, `tauri-runtime-wry = 2.10.1`, `tauri-macros = 2.5.5`, `wry = 0.54.2`. Source read from `~/.cargo/registry/src/index.crates.io-…/`.

**Key findings (all file:line verified):**

1. **`#[tauri::command] pub fn foo()` does NOT get auto-wrapped.** `tauri-macros/src/command/wrapper.rs:225-244` branches on `ExecutionContext`. Sync `pub fn` → `ExecutionContext::Blocking` → `body_blocking` (line 384-390) emits a **direct synchronous call** on the calling thread. No `spawn_blocking`, no task hop. **This means the reverted `5f60448` was NOT a no-op** — it *was* moving work off the calling thread. It just moved it to the wrong place (a blocking-pool thread) while leaving the UI thread to still do all 32 fan-out dispatches serially. The regression was the overhead of the extra hop with no contention removed.

2. **IPC receive loop is SERIAL on the WebView2 UI thread.** `wry-0.54.2/src/webview2/mod.rs:950` registers a single `WebResourceRequested` handler, which COM STA delivers serially. Line 1016 calls `custom_protocol_handler(..)` inline on the UI thread. That forwards to `tauri-runtime-wry-2.10.1/src/lib.rs:4983` → `tauri-2.10.3/src/ipc/protocol.rs:38-183` → `Webview::on_message` at `tauri-2.10.3/src/webview/mod.rs:1724-1893` → `manager.run_invoke_handler(invoke)` at line 1888, which calls the macro-generated wrapper, which (for sync commands) calls the user function directly. **The entire chain — ACL lookup, handler dispatch, and the full sync command body — runs inline on the WebView2 UI thread.** That's the serialization point.

3. **`#[tauri::command(async)] pub fn foo()` sidesteps it.** `wrapper.rs:241` marks sync fns under `ExecutionContext::Async` as kind `"sync_threadpool"`; `body_async` (316-352) generates `resolver.respond_async_serialized(async move { $path(args) })`, and `respond_async_serialized` at `tauri-2.10.3/src/ipc/mod.rs:375` calls `crate::async_runtime::spawn(async move { task.await })`. The UI thread pays only the spawn cost (microseconds) and is freed to drain the next IPC message; the sync body runs on a Tokio async-runtime worker.

4. **The `spawn_blocking` theory was wrong.** Sync commands don't touch Tokio at all. That's why `5f60448`'s `spawn_blocking` wrapper regressed — it added a hop with no benefit. What actually needed to move was the **dispatch**, not the body, and the correct lever is `#[tauri::command(async)]` on the fan-out commands.

5. **Prime suspect: `watch_library`** (notify crate's `ReadDirectoryChangesW` install on 16 libraries), **but also** `get_all_library_stats` (its body `.join()`s 16 `std::thread::spawn` workers synchronously — the UI thread stalls for the slowest library's scan), and **`read_library_appearance`** (16 × disk read + JSON parse).

**Ruled out by source read:** Tokio worker-pool saturation (sync commands don't touch Tokio); `StateManager` mutex (locked only during a HashMap lookup + downcast, and `cache_boot_snapshot_core` doesn't take `State<T>` anyway); ACL authority mutex (cheap HashMap lookup); async task starvation (Core is a sync command, never enters the runtime).

**Experiment A+ landed (`9001b01`):** converted the four highest-impact fan-out commands to `#[tauri::command(async)]`:

- `watch_library`, `unwatch_library` (`src-tauri/src/watcher.rs`)
- `get_all_library_stats`, `read_library_appearance` (`src-tauri/src/libraries.rs`)

Bodies unchanged. Docstrings added to each, citing the exact Tauri / wry file:lines so future maintainers don't need to re-do the investigation. Compile-check passed clean (0 new warnings, 59 pre-existing).

**Kept sync** (intentionally): `cache_boot_snapshot_core` and `cache_boot_snapshot_graph`. Graph fires via `requestIdleCallback` after fan-out drains, so its queue (measured at 90 ms in Round 3) is fine. Core's 19,880 ms queue in Round 3 was **caused by** the UI-thread stall above, not by core itself — once the fan-out is off the UI thread, core's own queue should drop to single-digit ms. Moving core to async would just have it compete with the 16+ fan-out bodies now running on the async-worker pool, with no net benefit.

**Predicted outcome on next measurement:**

- `core_queue_ms`: 19,880 → single-digit ms (dominant fix).
- `hydrated_ms`: 20,610 → under 6,000 (Criterion 2 PASS). Worst-case guess: if UI-thread contention is fully removed, the remaining wall is just `core_body_ms` (~112 ms) plus JS hydration and initial paint work.
- `graph_queue_ms`: unchanged (~90 ms).
- If `core_queue_ms` only partially drops, we know there's a **specific other fan-out command** still hogging the UI thread, and the Debug scorecard tells us exactly how much and how long. That makes the next iteration a surgical follow-up, not another blind guess.
- If `core_queue_ms` doesn't drop at all, the UI-thread hypothesis is falsified and we escalate to the ACL / state manager candidates the agent flagged as weak but non-zero.

## Commits (updated)

- `9001b01` — Experiment A+: fan-out commands to `#[tauri::command(async)]`.

## Open items

- **User action (priority)**: once `bahh636oa` ships, relaunch `src-tauri/target/release/constellation.exe` on trial Universe; read Settings → Debug → Boot Performance. Expected: Criterion 2 PASS with `core_queue_ms` ≈ 0.
- **Housekeeping**: `TAURI_SIGNING_PRIVATE_KEY` env-var plumbing for release builds (updater signature generation, non-fatal).
- **M11-data v2**: next targets §§ 107+ — continue toward 20 K-concept long-term goal. Deferred until Criterion 2 verification confirms the fix.

## Standing Order follow-ups

- Update `docs/help.uConstellation.World/` + `docs/User Manual.md` + 14 translations if the SkyView indicator becomes documented user-facing behaviour (deferred — transient 6–9 s transitional state, not a persistent feature).
- `/simplify` pass (code review) after both boot-criterion-2 halves (core/graph split + indicator) are end-to-end verified on the trial Universe.

---

## § 15 — Round 5: Experiment A+ validated but incomplete; the real fan-out was JS-side

### Round 4 measurement (commit `9001b01`, Experiment A+ in place)

User re-tested on trial Universe. Debug → Boot Performance scorecard:

- **Criterion 2: FAIL** — `hydrated_ms` = 20,160 ms (target 6,000).
- `paint_ms` = 525 ms (Criterion 1 PASS).
- `core_queue_ms` = **19,502 ms** ← still almost entirely consumed before body runs.
- `core_body_ms` = 73 ms (fast, as expected).
- `core_wall_ms` = 19,575 ms.
- `load_all_stats_wall_ms` = **0 ms**, `start_watching_all_wall_ms` = **13 ms**, `load_all_appearances_wall_ms` = **13 ms**.

### What this proved

The four commands converted in Experiment A+ are now exactly where we predicted they'd be: essentially zero wall time. The dispatch mechanism works. **But they were never the blocker** — all three measured fan-outs fire inside `+layout.svelte` **after** `await refreshLibraryCaches()` resolves, meaning they always queued *behind* `cache_boot_snapshot_core`, never in front of it.

So the 19,502 ms `core_queue_ms` gap comes from something **earlier** in boot — something between paint (525 ms) and core's body starting (~20,000 ms) — that was never in the fan-out we converted.

### Investigation (source read, no speculation)

Traced the JS boot sequence end-to-end in `src/routes/+layout.svelte`:

- Line 1482 — `libraries.set(bundle.libraries)` is called **inside** `initializeApp` before `refreshLibraryCaches` is reached.
- Line 4519 — `{#if $libraries.length > 0 && $appSettings.showDashboard}` gates `<DashboardView />`. The instant the store update on line 1482 fires, Svelte mounts `DashboardView`.
- `src/lib/components/DashboardView.svelte:85–134` — `onMount` runs `loadDashboardData()`, which sequentially `await`s:
  1. `loadAllStats()` (already fast after A+)
  2. `getChildUniverses()` — **sync** `#[tauri::command]` in `src-tauri/src/universe.rs:1102`
  3. for each child: `invoke('read_child_universe_libraries')` — **sync** `#[tauri::command]` in `src-tauri/src/universe.rs:1167`
  4. `scanAllLibraryTags()` (`src/lib/libraries/tagUtils.ts:14–27`) — **16 sequential** `invoke('scan_library_tags')` calls, one per library in a `for` loop.

`scan_library_tags` (`src-tauri/src/libraries.rs:1670`) is a sync `#[tauri::command]` that calls `scan_tags_recursive` — which walks every directory and calls `fs::read_to_string` on every `.md` file, then runs a regex. On the trial Universe that's ~7,600 file reads, sequentially, **on the WebView2 UI thread**. Multiply by 16 libraries, with every intermediate IPC message also queued behind it on the same STA thread, and the 19.5 s queue time is fully explained.

### Why this is the real blocker (not core itself)

The five-stamp model correctly located "core's 19.5 s is spent queued." The mechanism (`#[tauri::command]` sync binding runs inline on the UI thread) is also correct. What we got wrong in Round 4 was **which** sync command was holding the thread: we converted the `+layout.svelte` fan-out (which fires *after* core) when the culprit was `DashboardView.onMount` (which fires *before* core, the instant `libraries.set` runs).

The refined UI-thread hypothesis now matches the shape of the bug exactly: any sync fan-out that fires **between** `libraries.set()` and `await invoke('cache_boot_snapshot_core')` will queue in front of core. DashboardView is the chief offender on this boot path.

### Round 5 fix (`f018ad7`)

Three commands converted from `#[tauri::command]` to `#[tauri::command(async)]`:

- `scan_library_tags` — `src-tauri/src/libraries.rs:1690`
- `get_child_universes` — `src-tauri/src/universe.rs:1114`
- `read_child_universe_libraries` — `src-tauri/src/universe.rs:1185`

All three now route through `respond_async_serialized` → `tauri::async_runtime::spawn`. Bodies unchanged. Each has a docstring explaining the DashboardView boot-path interaction and linking back to `watcher.rs` for the full Tauri dispatch chain (so a future reader sees the whole pattern in one place).

### Predicted outcome

- `core_queue_ms`: 19,502 → single-digit ms. `scan_library_tags × 16` now runs on Tokio workers, so they can't hold the UI thread.
- `hydrated_ms`: ~20,160 → under 6,000 (Criterion 2 PASS).
- DashboardView's initial paint may show "loading" slightly longer (tag list arrives async-parallel rather than queued-sync), which is the acceptable trade-off. Long-term, Rule 8 (Write-Time Derivation) says the correct fix is a persisted tag index maintained by a trigger on note-save — tracked as a separate open item, not needed to close Criterion 2.

### What we would look at if `core_queue_ms` does NOT drop

If Round 5 doesn't close the gap, the five-stamp scorecard will still pinpoint the remaining blocker. Candidates to audit next, in order:
1. Any other `onMount` hook mounted by the `libraries.set`/`appSettings.set` cascade (sidebar tree, status bar, quick-capture panel, recent files panel).
2. Sidebar virtualization's first-render measurement pass if it issues IPC.
3. Anything inside `initializeApp` between store-set and `refreshLibraryCaches` that invokes a sync command.

### LL-021 addendum (methodology)

The five-stamp model worked: it pointed to "core queued 19.5 s." The UI-thread mechanism was correct. What failed in Round 4 was **scope selection** — we converted the most-visible fan-out (`+layout.svelte`) without first reading every caller that runs between `libraries.set()` and `invoke('cache_boot_snapshot_core')`. Lesson: when the queue is on a command, enumerate **everything that races it on the same thread**, not just the obvious fan-out that runs *after* it.

## Commits (updated)

- `9001b01` — Experiment A+: fan-out commands to `#[tauri::command(async)]` (Round 4; kept in place — these are still correct).
- `f018ad7` — Round 5: DashboardView fan-out commands to `#[tauri::command(async)]`.

## Open items (updated)

- **User action (priority)**: once the Round 5 build ships, relaunch `src-tauri/target/release/constellation.exe` on trial Universe; read Settings → Debug → Boot Performance. Expected: Criterion 2 PASS with `core_queue_ms` ≈ 0.
- **Housekeeping**: `TAURI_SIGNING_PRIVATE_KEY` env-var plumbing for release builds.
- **M11-data v2**: §§ 107+ deferred until Criterion 2 verification passes.
- **Rule 8 follow-up**: persisted tag index (maintained by trigger on note save) to replace `scan_library_tags`'s filesystem walk entirely. Tracked separately; not needed for Criterion 2.

---

## § 16 — Round 5 FAIL, two falsifying diagnostics, Round 6: Rust-side IPC arrival tracer

### Round 5 measurement (commit `f018ad7`)

User re-tested. `core_queue_ms = 19,712` — **statistically indistinguishable** from Round 4's 19,502. The three DashboardView converts did exactly nothing at the Criterion 2 level.

This is the third consecutive patch round (A, A+, DashboardView fan-out) targeting "sync commands racing on the UI thread" without moving the needle. **LL-014 triggered**: stop patching, investigate.

### Adversarial investigation (hypothesis generation)

Dispatched an agent with the brief "adversarial, try to falsify the UI-thread-contention theory." It produced three live hypotheses:

- (a) **DashboardView mount itself is the blocker** — not its IPC fan-out, but Svelte component setup / `$effect` chain / DOM measurement running synchronously on the same thread.
- (b) **The fan-out includes something we missed** — sidebar virtualization first-render, status bar mount, a recent-files panel, any other `onMount` that fires on `libraries.set()`.
- (c) **JS itself is blocked** — the `invoke()` call is sitting in a JS microtask/promise queue that can't drain because the main JS thread is busy (not awaiting Rust).

Cheapest falsifier first: hypothesis (a) via single-line gate.

### Diagnostic 1: DashboardView gate (`{#if false}`)

Changed `+layout.svelte:4512` from `{#if $appSettings.showDashboard}` to `{#if false && $appSettings.showDashboard}`. Rebuilt.

Measurement: `core_queue_ms = 19,418`. Unchanged. **Hypothesis (a) falsified.** DashboardView's entire subtree — component setup, `onMount`, all IPC it fans out — is off the critical path for the 19-second queue.

Reverted the gate; no code kept.

### Diagnostic 2: JS event-loop heartbeat

Added a 100ms `setInterval` from `boot:paint` onward, tracking max gap between firings. If JS is blocked for N seconds, the gap will be N seconds. If JS is alive, gap ≤ 200 ms. Stored max-gap under `boot_heartbeat_max_gap_ms` in the boot-perf JSON, cleared at `boot:hydrated`.

Measurement: `boot_heartbeat_max_gap_ms = 112` over an 18,614 ms queue window. **Hypothesis (c) falsified.** JS is fully alive for the entire queue. It's not blocked on a microtask, not blocked on reactivity, not blocked on store derivations. The `invoke('cache_boot_snapshot_core')` promise is just *sitting there* waiting for Rust.

### What this proves, negatively

- Not the `+layout.svelte` fan-out (Round 4 converted it — no move).
- Not the DashboardView subtree (Round 5 converted it — no move; gate confirmed it's off the critical path).
- Not the JS thread at all (heartbeat is 112 ms, not 18,000 ms).
- Not the core body (`core_body_ms = 162` on the heartbeat run).

By elimination: the 18.6 seconds lives **between** JS's `chrome.webview.postMessage` and Rust's `invoke_handler` dispatching `cache_boot_snapshot_core`. That's the WebView2 host pump, wry's `web_message_received`, Tauri's IPC router, or the command-dispatch closure itself.

### Round 6: Rust-side IPC arrival tracer (in progress)

New module `src-tauri/src/perf_trace.rs` — append-only `Mutex<Vec<(String, u64)>>`. The `invoke_handler` in `lib.rs` is wrapped in a closure that calls `perf_trace::record(invoke.message.command())` **before** dispatching to the handler produced by `generate_handler!`. This captures a Unix-ms timestamp on every command that reaches the Rust dispatcher, independent of any per-command edits.

Two new commands:
- `get_perf_trace_log` — returns the full log as `Vec<(String, u64)>`.
- `clear_perf_trace_log` — resets between runs (not wired yet; kept for future diagnostic cycles).

Frontend (`+layout.svelte`): `recordBootPerf` now awaits `get_perf_trace_log` at the `boot:hydrated` boundary and includes the returned array as `ipc_arrival_log` in the boot-perf JSON.

### Decision tree for the next measurement

- **If the log shows many command arrivals during the 18.6 s window, with timestamps drifting forward**: the dispatcher IS serialized by something. Look at which commands are there — that names the culprit and decides the next conversion.
- **If the log shows only `cache_boot_snapshot_core` arriving at the ~18.6 s mark**: the delay is upstream of Rust entirely. Next diagnostic moves into WebView2 / wry (the Tauri IPC router is not the bottleneck).
- **If the log is empty or nearly empty for the entire window**: nothing reached Rust until core did. Same conclusion as above — upstream of Rust.

### LL-021 addendum (live)

Two more diagnostics' worth of methodology:
1. **Test cheap falsifiers before expensive rewrites.** The DashboardView gate is 14 characters. It falsified a whole hypothesis in one rebuild.
2. **Heartbeats are almost free.** A `setInterval(…, 100)` with max-gap tracking costs nothing and conclusively separates "JS blocked" from "JS waiting on Rust."
3. **When the JS layer and the Rust body are both fast but the promise is slow, instrument the transport.** That's what Round 6 is doing.

## Commits (updated)

- `9001b01` — Experiment A+: fan-out commands to `#[tauri::command(async)]` (Round 4; kept in place — these are still correct).
- `f018ad7` — Round 5: DashboardView fan-out commands to `#[tauri::command(async)]` (kept in place — correctness-improving; performance-neutral as measured).
- `4195c09` — Round 6: `perf_trace` module + `invoke_handler` wrapper + boot-perf `ipc_arrival_log` field.

## Open items (updated)

- **User action (priority)**: once the Round 6 build ships, relaunch `src-tauri/target/release/constellation.exe` on trial Universe; read Settings → Debug → Boot Performance. The `ipc_arrival_log` field tells us whether the queue lives in Tauri's Rust dispatcher or upstream of it.
- **Housekeeping**: `TAURI_SIGNING_PRIVATE_KEY` env-var plumbing for release builds.
- **M11-data v2**: §§ 107+ deferred until Criterion 2 verification passes.
- **Rule 8 follow-up**: persisted tag index to replace `scan_library_tags`'s filesystem walk. Still tracked separately; not needed for Criterion 2.

---

## § 17 — Round 6 measurement: named the blocker. Round 7: `constellation_map_universe → (async)`.

### Round 6 measurement (commit `4195c09`, trial Universe, 7,595 notes)

Scorecard:
- Criterion 1 PASS — `paint_ms = 829`.
- Criterion 2 **FAIL** — `hydrated_ms = 21,667`.
- `cache_snapshot_core_queue_ms = 20,693`, `core_body_ms = 72`, `boot_heartbeat_max_gap_ms = 111`.
- `ipc_arrival_log` (timestamps relative to `invoke_start_unix_ms = 1776604385000`):

| t (ms) | command |
|---|---|
| +428 | `constellation_link_decay` |
| +428 | `list_universes` |
| +431 | `check_migration_needed` |
| +433 | `set_active_universe` |
| +551 | `constellation_boot_bundle` |
| +566 | `constellation_map_universe` ← |
| +17,792 | `constellation_map_universe` (2nd) |
| +21,294 | `cache_boot_snapshot_core` ← picked up 20.7 s after JS posted it |
| +21,388 | `get_perf_trace_log` |

The 17.2-second gap between the two `constellation_map_universe` arrivals is the dispatcher being blocked by the first call running inline on the UI thread. The second call took a further 3.5 s (OS page cache warm after the first pass). `cache_boot_snapshot_core`'s `invoke_start_unix_ms` was +601 — posted only 35 ms after the first map call — and sat queued for the entire 20.7 s. That fully accounts for `core_queue_ms`.

### Why two calls

`+layout.svelte:4134` wraps `<ConstellationMap>` in `<div class="map-overlay" class:map-visible={showConstellationMap}>` — **always mounted**, CSS-hidden. Its `onMount` → `loadData()` → `invoke('constellation_map_universe')`. First call.

`+layout.svelte:4173` wraps `<OrgChart fullscreen={true}>` in `<div class="orgchart-overlay" class:orgchart-visible={showOrgChart}>` — **always mounted**, CSS-hidden. Its `$effect` at `OrgChart.svelte:735` (`if (fullscreen && !mapRoot && !loading) loadFullscreenData()`) → `invoke('constellation_map_universe')`. Second call.

Both mount patterns are deliberate ("preserve drill-down state across navigation"). Both trigger the heavy walk unconditionally on boot.

### Round 7 fix (pending build)

`src-tauri/src/map.rs:193` — convert `constellation_map_universe` from `#[tauri::command]` to `#[tauri::command(async)]`. Body unchanged (still walks every library's filesystem). Both boot-time dispatches now route through `tauri::async_runtime::spawn`; the UI thread is free for `cache_boot_snapshot_core` to dispatch immediately.

Expected outcome: `core_queue_ms` drops from ~20,693 to single-digit ms; `hydrated_ms` drops from ~21,667 to ~1,500–2,000 ms (paint + core body + assign + the other small fan-out). Criterion 2 PASS.

### What this doesn't fix (deferred)

The trial Universe now does a ~17-second filesystem walk for map data the user may never ask for. Post-Criterion-2 follow-up (tracked separately):
1. Gate both overlays with `{#if showConstellationMap}` / `{#if showOrgChart}` so the walk runs only on first open.
2. Persist the derived map tree (Rule 8 — write-time derivation), maintained by note-save triggers, so even the first open is instant.

### Commits (updated, pending Round 7)

- `9001b01` — Experiment A+ (Round 4).
- `f018ad7` — Round 5: DashboardView fan-out → `(async)`.
- `4195c09` — Round 6: IPC arrival tracer.
- `b0bd3eb` — session-log hash backfill.
- `8a74949` — Round 7: `constellation_map_universe` → `#[tauri::command(async)]`.
- `81675b0` — session-log hash backfill for Round 7.

---

## § 18 — Criterion 2 CLOSED (Round 7 verified)

### Measurement (trial Universe, 7,595 notes, commit `8a74949`)

Scorecard:
- Criterion 1 **PASS** — `paint_ms = 658 ms` (≤ 2,500).
- Criterion 2 **PASS** — `hydrated_ms = 811 ms` (≤ 6,000; first PASS).
- `core_queue_ms = 4` (down from 20,693 — 5,173× reduction).
- `core_body_ms = 49`, `core_wall_ms = 100`.
- `graph_ready_ms = 6,898` (informational; not gated).
- `boot_heartbeat_max_gap_ms = 111` (unchanged — JS was never the issue).

Arrival log (relative to first arrival at 1776605616574):

| t (ms) | command |
|---:|---|
| +0 | `constellation_link_decay` |
| +1 | `list_universes` |
| +5 | `check_migration_needed` |
| +8 | `set_active_universe` |
| +21 | `constellation_boot_bundle` |
| +40 | `constellation_map_universe` (1st, async-spawned) |
| +42 | `constellation_map_universe` (2nd, async-spawned) |
| +75 | `cache_boot_snapshot_core` |
| +175 | `get_perf_trace_log` |

The two map calls fire 2 ms apart (both immediately handed to Tokio workers, no longer serializing the UI thread). `cache_boot_snapshot_core` arrives 33 ms after the second map call with zero queue. Arrival log surgically confirms the Round 6 hypothesis and the Round 7 fix.

### What the ship-gate looks like now

All ship-gates that were previously blocked are now met:
- Criterion 1 (paint ≤ 2.5 s) — PASS.
- Criterion 2 (fully responsive ≤ 6 s) — PASS at 811 ms, 7.4× under budget.
- Criterion 3 (RSS ≤ 350 MB) — still "not measured"; measurement only, no code change needed.
- Criterion 4 (post-boot stat sweep) — implementation pending; tracked separately.
- Criterion 5 (kill-mid-index recovery) — implementation pending; tracked separately.

### What's still non-optimal (acceptable, deferred)

`constellation_map_universe` still does a full filesystem walk on every boot (twice — once per mount-time caller), just in parallel on Tokio workers. On trial Universe that's ~17 seconds of background IO+CPU for data the user may never ask for. Rule 8 follow-up pending — lazy-mount the overlays + persist the derived map tree. Not blocking Criterion 2, not affecting typing latency, tracked in open items.

### Methodology payoff (LL-021 live)

Round 4 → 7 cycle in retrospect: three patch rounds that kept the same shape (sync→async conversions on commands the developer *thought* were the blocker) all failed because the blocker was never read — only guessed. The moment we built the IPC arrival tracer (one mutex + one Vec, ~30 lines), the answer fell out in the first measurement. LL-014 ("three-strike rule") triggered correctly but didn't prescribe a direction; the direction came from LL-017 + heartbeat + arrival log. LESSONS-LEARNED update pending (§ LL-021 addendum with the full five-diagnostic methodology).

### Commits (final)

- `9001b01` — Experiment A+ (Round 4; correct but not the blocker).
- `f018ad7` — Round 5 (DashboardView fan-out; same — correct but not the blocker).
- `4195c09` — Round 6: IPC arrival tracer (diagnostic; kept in place for future cycles).
- `b0bd3eb` — Round 6 hash backfill.
- `8a74949` — Round 7: `constellation_map_universe` → `#[tauri::command(async)]` (the actual fix).
- `81675b0` — Round 7 hash backfill.

### Open items after Criterion 2 closure

- **LESSONS-LEARNED update**: append LL-021 addendum covering the five-diagnostic methodology (queue-time stamps → heartbeat → gate diagnostic → IPC arrival tracer → named-culprit conversion).
- **Milestone tag**: `milestone/criterion-2-closed` per CLAUDE.md backup routine.
- **/simplify pass**: Standing Order code review.
- **Rule 8 follow-up (upgrade, not fix)**: gate `<ConstellationMap>` and `<OrgChart fullscreen>` with `{#if}` so the filesystem walk runs only on first open; persist the derived map tree via note-save triggers.
- **Housekeeping**: `TAURI_SIGNING_PRIVATE_KEY` env-var plumbing.
- **M11-data v2**: §§ 107+ toward 20K-concept goal — now unblocked.

## § 19 — Rule 8 follow-up: lazy-mount the Map + OrgChart overlays

### Problem restated (post-Criterion-2)

Criterion 2 passed at `hydrated_ms = 811` — but the boot `ipc_arrival_log` still showed
two `constellation_map_universe` dispatches 17 seconds apart. The Round 7 `async`
conversion routed that work off the UI thread (which is why Criterion 2 closed), but the
work itself was pure waste: a full Universe filesystem walk for a view the user may never
open in that session.

Root cause: `<ConstellationMap>` and the fullscreen `<OrgChart>` overlays in
`+layout.svelte` were both **always mounted**, hidden with CSS (`class:map-visible=…`,
`class:orgchart-visible=…`) rather than `{#if}`. The original comment said "always
rendered, hidden with CSS to preserve drill-down state" — the motivation was real (state
in `mapFocusNode` / `mapColorMode` must survive close/reopen), but the cost was the two
mount-time IPC calls on every boot.

### Fix landed

Two-part change, one file (`src/routes/+layout.svelte`), +78/-55 net:

1. **State** (lines ~495-505): added `mapEverOpened` / `orgChartEverOpened` sticky flags
   + two single-line `$effect` hooks that flip them to `true` when the respective overlay
   is first shown. Colocated next to the existing `mapColorMode` / `mapFocusNode` state
   so future readers see the lazy-mount machinery right alongside the drill-down state it
   preserves.

2. **Template** (lines ~4144, 4194): wrapped both overlay `<div>`s with
   `{#if mapEverOpened}` / `{#if orgChartEverOpened}`. Kept the existing `class:*-visible`
   toggles on the inner div so open/close animation and focus handling work unchanged.
   Replaced the old one-line comment with a multi-line explainer pointing at the Round 7
   async conversion and the Rule 8 write-time-derivation follow-up that will supersede
   this lazy-mount later.

Drill-down state preservation: the `*EverOpened` flag is one-way — it never flips back —
so after the first open, the overlay stays mounted for the rest of the session. This is
functionally identical to the old CSS-hide pattern from second open onwards. First open
is the one that changed: it now costs one async `constellation_map_universe` dispatch,
but it's triggered by a user gesture, routed through `tauri::async_runtime::spawn`, and
the loading state is already handled by the component itself (`loading = true` while the
IPC is in flight). No new "loading" UI needed.

### Why this is a Rule 8 **follow-up**, not a Rule 8 **fix**

Rule 8 (Write-Time Derivation): every computed view should be maintained at write time,
not read time. The canonical fix for Map is to persist the map tree in SQLite (via
triggers on note save / rename / delete), so even first-open is a cheap DB read instead
of a filesystem walk. That's a larger refactor — schema, triggers, back-fill migration —
and it's tracked separately.

Lazy-mount is the cheap win: the walk still happens eventually on first open, but never
unbidden. Combined with Round 7's async routing, the UX is: zero boot-path impact, then
a brief interactive loading state on first click. Acceptable until the persistent-map
refactor ships.

### Verification

- **Build**: `npm run tauri build` — **2m 13s release compile**, exit 0 code / exit 1 only
  from `TAURI_SIGNING_PRIVATE_KEY` housekeeping (unrelated, pre-existing open item). Both
  bundles produced: `Constellation_0.3.4_x64_en-US.msi` + `Constellation_0.3.4_x64-setup.exe`.
- **svelte-check**: zero new errors; the 53 errors listed are all pre-existing in files I
  didn't touch (NoteEditor, ConstellationMap type-refs, OrgChart null-check, etc.).
- **Grep audit**: confirmed `showConstellationMap` initializes `false` (line 334),
  `showOrgChart` initializes `false` (line 387), all setters come from user-gesture
  handlers (dock buttons, command palette, return-pill). No boot-time `show* = true`
  path, so `*EverOpened` stays `false` through boot as intended.
- **Second `<OrgChart>` instance in sidebar** (line 3726): uses `embedded={true}` not
  `fullscreen={true}`, and is already conditionally mounted inside
  `{:else if sidebarMode === 'skyview'}`. The IPC walk in `OrgChart.svelte:735` is gated
  on `fullscreen && !mapRoot && !loading` — the embedded instance never triggers it. Safe.
- **SecondScreenPage** also imports `ConstellationMap`, but that's a separate window
  that only mounts when the user opens the second screen — not on main-window boot. Safe.

### Documentation landed

- `docs/LESSONS-LEARNED.md` — LL-022 added: "Always-Mounted UI = Always-Running IPC",
  codifying the CSS-hide-vs-{#if} distinction, the `*EverOpened` sticky-flag pattern,
  and the rule: any always-mounted component whose mount performs IPC larger than O(1)
  must be audited. Points at Rule 8 as the deeper follow-up.

### Expected behavioural measurement (next boot on trial Universe)

- `hydrated_ms`: still ~811 ms (no change expected — Criterion 2 was already passing).
- `ipc_arrival_log`: `constellation_map_universe` entries **should vanish entirely** from
  the boot log (they were the last unbidden commands). If not, there's another caller
  and further investigation is warranted.
- First click on "Map" button: one `constellation_map_universe` dispatch, ~17 s for the
  walk on 7,600 notes, overlay shows loading state until it resolves. Subsequent Map
  opens in the same session: instant (mounted instance re-shown).
- Same pattern for OrgChart fullscreen.

### Commits

- `<pending commit>` — `Boot Criterion 2 Rule 8 follow-up: lazy-mount Map + OrgChart overlays`
  (`src/routes/+layout.svelte` +78/-55, `docs/LESSONS-LEARNED.md` +LL-022,
  `lab/reports/SESSION-LOG-2026-04-19.md` +§ 19).
