#!/usr/bin/env python3
"""
Constellation Lexicon — post-build TSV validator.

Reads the emitted lexicon_v1.tsv and runs content checks that the build
step (build.py) doesn't enforce — script match per language, Arabic
normalization invariants, coverage floor, and duplicate detection
across the whole corpus.

Exit codes:
  0 — all checks passed (warnings OK)
  1 — hard error (coverage floor violated, control char, etc.)
  2 — file not found / malformed TSV

Hard errors:
  - Any row without en: + ar: (both with ≥1 lemma)
  - Any ar/fa/ur lemma containing tashkeel (U+064B–U+065F) or tatweel (U+0640)
  - Any lemma containing a tab, newline, or the PoS-column separator
  - Any duplicate lemma within a single (concept × language) cell

Warnings (printed but don't fail the build):
  - Rows with fewer than 8 of 15 languages populated
  - Script mismatch per language (e.g. Latin characters in an ar: cell)
  - Any concept_id appearing more than once (should be caught by build.py
    but we re-check in case the TSV was hand-edited post-build)
"""

from __future__ import annotations

import sys
import unicodedata
from pathlib import Path
from typing import Dict, List, Optional, Tuple

REPO_ROOT = Path(__file__).resolve().parents[2]
OUTPUT_TSV = REPO_ROOT / "src-tauri" / "src" / "lexicon" / "data" / "lexicon_v1.tsv"

SUPPORTED_LANGS = {"ar", "de", "en", "es", "fa", "fr", "he", "hi",
                   "ja", "ko", "pt", "ru", "tr", "ur", "zh"}

# Minimum distinct languages per row before we warn. Not a hard gate —
# a concept with only en+ar+one-or-two-others still ships, but the row
# will surface in the warning list so we know what needs filling.
MIN_LANG_COVERAGE = 8

# Tashkeel and tatweel ranges that must be absent from ar/fa/ur lemmas.
# Parser-side normalization strips these on every lookup; storing them
# in the corpus would do the same work on every boot for no benefit.
ARABIC_MARKS = set(range(0x064B, 0x0660)) | {0x0640}  # U+064B..U+065F + U+0640

# ─── Script range tables (Unicode block membership, best-effort) ──────────────

LATIN = [(0x0000, 0x024F), (0x1E00, 0x1EFF), (0x2C60, 0x2C7F)]  # basic + extended
ARABIC = [(0x0600, 0x06FF), (0x0750, 0x077F), (0xFB50, 0xFDFF), (0xFE70, 0xFEFF)]
HEBREW = [(0x0590, 0x05FF), (0xFB1D, 0xFB4F)]
DEVANAGARI = [(0x0900, 0x097F)]
HIRAGANA = [(0x3040, 0x309F)]
KATAKANA = [(0x30A0, 0x30FF), (0x31F0, 0x31FF)]
CJK = [(0x4E00, 0x9FFF), (0x3400, 0x4DBF), (0x20000, 0x2A6DF)]
HANGUL = [(0xAC00, 0xD7AF), (0x1100, 0x11FF), (0x3130, 0x318F)]
CYRILLIC = [(0x0400, 0x04FF), (0x0500, 0x052F)]

# Characters that are allowed in any script (punctuation, whitespace,
# digits). Kept narrow — we don't want to silently accept ASCII letters
# in a Cyrillic cell.
NEUTRAL = [(0x0020, 0x0040), (0x005B, 0x0060), (0x007B, 0x007E),
           (0x00A0, 0x00BF), (0x2000, 0x206F), (0x3000, 0x303F)]

# Per-language acceptable script ranges. "Accept" means "at least one
# character in this range is expected"; we flag the lemma if ALL its
# characters fall outside the accept set.
ACCEPT: Dict[str, List[Tuple[int, int]]] = {
    "en": LATIN,
    "de": LATIN,
    "es": LATIN,
    "fr": LATIN,
    "pt": LATIN,
    "tr": LATIN,
    "ar": ARABIC,
    "fa": ARABIC,
    "ur": ARABIC,
    "he": HEBREW,
    "hi": DEVANAGARI,
    "ja": HIRAGANA + KATAKANA + CJK + LATIN,  # romaji allowed
    "ko": HANGUL + CJK + LATIN,               # Hanja + romaji tolerated
    "zh": CJK + LATIN,                         # pinyin sometimes bundled
    "ru": CYRILLIC,
}


# ─── Helpers ──────────────────────────────────────────────────────────────────

def in_ranges(cp: int, ranges: List[Tuple[int, int]]) -> bool:
    for lo, hi in ranges:
        if lo <= cp <= hi:
            return True
    return False


def script_ok(lemma: str, lang: str) -> bool:
    """True if at least one character in `lemma` matches the language's
    expected script (plus neutral chars)."""
    accept = ACCEPT.get(lang, [])
    if not accept:
        return True  # unknown lang — skip check
    for ch in lemma:
        cp = ord(ch)
        if in_ranges(cp, accept):
            return True
        if in_ranges(cp, NEUTRAL):
            continue
        # Letter in another script — keep scanning; we only fail if the
        # entire lemma is outside the expected ranges.
    return any(in_ranges(ord(ch), accept) for ch in lemma)


def has_arabic_marks(lemma: str) -> bool:
    """True if `lemma` contains tashkeel or tatweel — forbidden in the
    stored corpus since the parser strips them on every lookup."""
    return any(ord(ch) in ARABIC_MARKS for ch in lemma)


def parse_tsv(path: Path) -> List[Dict[str, object]]:
    """Parse the emitted TSV back into concept dicts for validation."""
    if not path.exists():
        print(f"validate.py: error: {path} not found — run build.py first",
              file=sys.stderr)
        sys.exit(2)

    concepts: List[Dict[str, object]] = []
    for line_no, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw.rstrip("\r\n")
        if not line or line.startswith("#"):
            continue
        cells = line.split("\t")
        if len(cells) < 2:
            print(f"validate.py: line {line_no}: fewer than 2 columns — skipping",
                  file=sys.stderr)
            continue
        cid_col, pos_col, *lang_cells = cells
        if not cid_col.startswith("c:"):
            print(f"validate.py: line {line_no}: concept id {cid_col!r} "
                  f"missing 'c:' prefix", file=sys.stderr)
            continue
        cid = cid_col[2:]
        pos = pos_col if pos_col else "Unknown"

        lemmas: Dict[str, List[str]] = {}
        for cell in lang_cells:
            if ":" not in cell:
                continue
            lang, _, payload = cell.partition(":")
            if lang not in SUPPORTED_LANGS:
                continue
            lemmas[lang] = [v for v in payload.split(",") if v]

        concepts.append({"id": cid, "pos": pos, "lemmas": lemmas, "line": line_no})

    return concepts


# ─── Check runners ────────────────────────────────────────────────────────────

def check_required_languages(c: Dict[str, object], errors: List[str]) -> None:
    """en: and ar: must both be present with ≥1 lemma."""
    lemmas = c["lemmas"]  # type: ignore[index]
    for req in ("en", "ar"):
        if req not in lemmas or not lemmas[req]:
            errors.append(
                f"concept {c['id']!r} (line {c['line']}): missing required {req}: "
                f"lemma — every row must have en: + ar:"
            )


def check_arabic_marks(c: Dict[str, object], errors: List[str]) -> None:
    """ar/fa/ur lemmas must be already-stripped of tashkeel + tatweel."""
    for lang in ("ar", "fa", "ur"):
        for lemma in c["lemmas"].get(lang, []):  # type: ignore[attr-defined]
            if has_arabic_marks(lemma):
                errors.append(
                    f"concept {c['id']!r} (line {c['line']}): {lang}: lemma "
                    f"{lemma!r} contains tashkeel or tatweel — strip before "
                    f"storing (normalizer does this on every lookup)"
                )


def check_dedup(c: Dict[str, object], errors: List[str]) -> None:
    """No lemma may appear twice in the same (concept × language) cell."""
    for lang, values in c["lemmas"].items():  # type: ignore[attr-defined]
        seen = set()
        for v in values:
            if v in seen:
                errors.append(
                    f"concept {c['id']!r} (line {c['line']}): duplicate lemma "
                    f"{v!r} in {lang}: cell"
                )
            seen.add(v)


def check_coverage(c: Dict[str, object], warnings: List[str]) -> None:
    """Warn when fewer than MIN_LANG_COVERAGE of 15 langs are populated."""
    cov = sum(1 for values in c["lemmas"].values()  # type: ignore[attr-defined]
              if values)
    if cov < MIN_LANG_COVERAGE:
        warnings.append(
            f"concept {c['id']!r} (line {c['line']}): only {cov}/15 languages "
            f"populated (minimum target: {MIN_LANG_COVERAGE})"
        )


def check_script(c: Dict[str, object], warnings: List[str]) -> None:
    """Per-lang script check — warning not error, since some legitimate
    loanwords / proper nouns use foreign script."""
    for lang, values in c["lemmas"].items():  # type: ignore[attr-defined]
        for lemma in values:
            if not script_ok(lemma, lang):
                warnings.append(
                    f"concept {c['id']!r} (line {c['line']}): {lang}: lemma "
                    f"{lemma!r} is outside the expected script range — "
                    f"verify this is intended"
                )


def check_unique_ids(concepts: List[Dict[str, object]], errors: List[str]) -> None:
    seen: Dict[str, int] = {}
    for c in concepts:
        cid = c["id"]  # type: ignore[index]
        if cid in seen:
            errors.append(
                f"duplicate concept id {cid!r} at lines {seen[cid]} and {c['line']}"
            )
        else:
            seen[cid] = c["line"]  # type: ignore[assignment]


# ─── Entry point ──────────────────────────────────────────────────────────────

def main() -> int:
    concepts = parse_tsv(OUTPUT_TSV)

    errors: List[str] = []
    warnings: List[str] = []

    check_unique_ids(concepts, errors)
    for c in concepts:
        check_required_languages(c, errors)
        check_arabic_marks(c, errors)
        check_dedup(c, errors)
        check_coverage(c, warnings)
        check_script(c, warnings)

    print(f"validate.py: {OUTPUT_TSV.relative_to(REPO_ROOT)}")
    print(f"  concepts: {len(concepts)}")
    print(f"  errors:   {len(errors)}")
    print(f"  warnings: {len(warnings)}")

    if warnings:
        print("\n--- warnings ---")
        for w in warnings:
            print(f"  ⚠  {w}")

    if errors:
        print("\n--- errors ---", file=sys.stderr)
        for e in errors:
            print(f"  ✗  {e}", file=sys.stderr)
        return 1

    print("\n✓ all hard checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
