#!/usr/bin/env python3
"""
MIG-022 §E.1 — Backfill cece.confidence i18n keys across 13 non-en/non-ar locales.

Inserts `confidence` sub-object into each locale's `cece` block immediately
after the existing `regime` sub-object. Preserves key order via Python 3.7+
dict ordering. Uses ensure_ascii=False to keep non-Latin scripts readable.

Run from anywhere (paths absolute):
    python lab/scripts/mig022_e1_confidence_i18n.py
"""
import json
from pathlib import Path

I18N_DIR = Path(r"E:\مشاريع كلاود\Constellation\src\lib\i18n")

# Per-locale translations for the four Confidence enum values.
# en + ar already populated by hand in the §E.1 commit body — script
# handles the remaining 13 locales.
TRANSLATIONS = {
    "de": {"high": "Hoch", "medium": "Mittel", "low": "Niedrig", "abstain": "Enthaltung"},
    "es": {"high": "Alta", "medium": "Media", "low": "Baja", "abstain": "Abstención"},
    "fa": {"high": "بالا", "medium": "متوسط", "low": "پایین", "abstain": "امتناع"},
    "fr": {"high": "Élevée", "medium": "Moyenne", "low": "Faible", "abstain": "Abstention"},
    "he": {"high": "גבוה", "medium": "בינוני", "low": "נמוך", "abstain": "הימנעות"},
    "hi": {"high": "उच्च", "medium": "मध्यम", "low": "निम्न", "abstain": "अनुपस्थिति"},
    "ja": {"high": "高", "medium": "中", "low": "低", "abstain": "棄権"},
    "ko": {"high": "높음", "medium": "중간", "low": "낮음", "abstain": "기권"},
    "pt": {"high": "Alta", "medium": "Média", "low": "Baixa", "abstain": "Abstenção"},
    "ru": {"high": "Высокая", "medium": "Средняя", "low": "Низкая", "abstain": "Воздержание"},
    "tr": {"high": "Yüksek", "medium": "Orta", "low": "Düşük", "abstain": "Çekimser"},
    "ur": {"high": "بلند", "medium": "درمیانہ", "low": "کم", "abstain": "اجتناب"},
    "zh": {"high": "高", "medium": "中", "low": "低", "abstain": "弃权"},
}


def insert_confidence_after_regime(cece_block: dict, translations: dict) -> dict:
    """Rebuild the cece block dict with `confidence` inserted right after `regime`."""
    out = {}
    inserted = False
    for key, value in cece_block.items():
        out[key] = value
        if key == "regime" and not inserted:
            out["confidence"] = translations
            inserted = True
    if not inserted:
        # `regime` not found — append at end with a comment for visibility
        print(f"  WARN: `regime` sub-block not found; appending `confidence` at end of cece block")
        out["confidence"] = translations
    return out


def main():
    for locale, trans in TRANSLATIONS.items():
        path = I18N_DIR / f"{locale}.json"
        if not path.exists():
            print(f"SKIP {locale}: file not found at {path}")
            continue
        with path.open("r", encoding="utf-8") as f:
            data = json.load(f)
        if "cece" not in data:
            print(f"SKIP {locale}: no `cece` top-level key (was V3-§10.D backfill skipped here?)")
            continue
        if "confidence" in data["cece"]:
            print(f"SKIP {locale}: cece.confidence already present — already backfilled?")
            continue
        data["cece"] = insert_confidence_after_regime(data["cece"], trans)
        with path.open("w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
            f.write("\n")  # trailing newline (matches existing files)
        print(f"OK   {locale}: cece.confidence inserted after cece.regime")


if __name__ == "__main__":
    main()
