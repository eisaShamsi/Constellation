/**
 * Sight v3 — Rim-axis mode definitions (the 6 toggleable views).
 *
 * MIG-019 §2G.2. Per `docs/SIGHT-V3-VISUAL-SPEC.md` §1, the chart is a
 * multi-lens diagnostic instrument. The same Universe is read from six
 * different cognitive angles by switching the rim axis. Radius (centrality)
 * and color (community) stay invariant; only azimuth changes.
 *
 * Each mode declares: how its wedges are formed, what label to show on
 * each wedge, and whether it's currently backed by data. Modes with
 * `ready: false` render as dimmed buttons in the toggle bar with a
 * "Available later" tooltip — the implementation lights up when the
 * corresponding Concept Paper P2/P3/P4 features ship.
 */

export type SightMode = 'regions' | 'linkTypes' | 'time' | 'confidence' | 'stages' | 'acts';

/** Wedge labels are user-facing — they go through `$t()` at render time.
 *  Here we list the i18n KEYS, not the strings themselves. */
export interface ModeMeta {
    /** Internal id, also stored in `appSettings.sight.lastMode`. */
    id: SightMode;
    /** Single-letter label for the toggle button (R/L/T/C/S/A). */
    letter: string;
    /** i18n key for the full mode name (e.g., "sight.v3.mode.regions"). */
    nameKey: string;
    /** i18n key for the one-line description shown below the toggle bar. */
    captionKey: string;
    /** Backed by current data? If false, render as dimmed "Available later". */
    ready: boolean;
    /** Wedge count: fixed (number) or 'variable' (computed from data). */
    wedges: number | 'variable';
    /** For non-variable modes: the wedge labels (i18n keys). */
    wedgeLabels: ReadonlyArray<string> | null;
    /** What data field defines a note's azimuth in this mode. Used in
     *  the visual-spec doc — not consumed at runtime, but kept here so
     *  the contract is single-sourced. */
    azimuthRule: string;
}

/** All 6 modes in canonical toggle order: ready first (R · L · T),
 *  then "available later" (C · S · A). */
export const MODES: ReadonlyArray<ModeMeta> = Object.freeze([
    {
        id: 'regions',
        letter: 'R',
        nameKey: 'sight.v3.mode.regions.name',
        captionKey: 'sight.v3.mode.regions.caption',
        ready: true,
        wedges: 'variable',
        wedgeLabels: null,
        azimuthRule: 'library_path of the note',
    },
    {
        id: 'linkTypes',
        letter: 'L',
        nameKey: 'sight.v3.mode.linkTypes.name',
        captionKey: 'sight.v3.mode.linkTypes.caption',
        ready: true,
        wedges: 7,
        wedgeLabels: Object.freeze([
            'sight.v3.linkType.supports',
            'sight.v3.linkType.contradicts',
            'sight.v3.linkType.causes',
            'sight.v3.linkType.exemplifies',
            'sight.v3.linkType.generalizes',
            'sight.v3.linkType.derivesFrom',
            'sight.v3.linkType.partOf',
        ]),
        azimuthRule: 'dominant outgoing link type',
    },
    {
        id: 'time',
        letter: 'T',
        nameKey: 'sight.v3.mode.time.name',
        captionKey: 'sight.v3.mode.time.caption',
        ready: true,
        wedges: 'variable',
        wedgeLabels: null,
        azimuthRule: 'created_at year (with month sub-divisions on the most recent year)',
    },
    {
        id: 'confidence',
        letter: 'C',
        nameKey: 'sight.v3.mode.confidence.name',
        captionKey: 'sight.v3.mode.confidence.caption',
        ready: false,
        wedges: 4,
        wedgeLabels: Object.freeze([
            'sight.v3.confidence.hypothesis',
            'sight.v3.confidence.evidence',
            'sight.v3.confidence.established',
            'sight.v3.confidence.contested',
        ]),
        azimuthRule: 'confidence of strongest outgoing link',
    },
    {
        id: 'stages',
        letter: 'S',
        nameKey: 'sight.v3.mode.stages.name',
        captionKey: 'sight.v3.mode.stages.caption',
        ready: false,
        wedges: 6,
        wedgeLabels: Object.freeze([
            'sight.v3.stage.spark',
            'sight.v3.stage.birth',
            'sight.v3.stage.growth',
            'sight.v3.stage.maturity',
            'sight.v3.stage.dormancy',
            'sight.v3.stage.archival',
        ]),
        azimuthRule: 'dominant lifecycle stage of links',
    },
    {
        id: 'acts',
        letter: 'A',
        nameKey: 'sight.v3.mode.acts.name',
        captionKey: 'sight.v3.mode.acts.caption',
        ready: false,
        wedges: 5,
        wedgeLabels: Object.freeze([
            'sight.v3.act.observation',
            'sight.v3.act.connection',
            'sight.v3.act.tension',
            'sight.v3.act.synthesis',
            'sight.v3.act.conviction',
        ]),
        azimuthRule: 'which Act produced the note',
    },
]);

/** Quick lookup by id. */
export const MODE_BY_ID: ReadonlyMap<SightMode, ModeMeta> = Object.freeze(
    new Map(MODES.map((m) => [m.id, m])),
);

/** The default mode for a fresh Universe (spec §1.3). */
export const DEFAULT_MODE: SightMode = 'regions';

/** Resolve a possibly-stored-but-now-unready mode to the default.
 *  E.g., if a user had "confidence" set on an old build but the
 *  Confidence Pack hasn't shipped here, fall back to Regions. */
export function resolveMode(stored: string | undefined | null): SightMode {
    if (!stored) return DEFAULT_MODE;
    const meta = MODE_BY_ID.get(stored as SightMode);
    if (!meta || !meta.ready) return DEFAULT_MODE;
    return meta.id;
}

/** Keyboard shortcut → mode (spec §5.3). Case-insensitive. */
export const KEYS_TO_MODE: ReadonlyMap<string, SightMode> = Object.freeze(
    new Map<string, SightMode>([
        ['r', 'regions'],
        ['l', 'linkTypes'],
        ['t', 'time'],
        ['c', 'confidence'],
        ['s', 'stages'],
        ['a', 'acts'],
    ]),
);
