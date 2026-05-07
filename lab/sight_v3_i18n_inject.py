"""MIG-018 §1C i18n — inject sightV3 + constellationSightV3 keys into all
14 non-en locales. ar gets Arabic translations; the 13 others get English
placeholders (PJ-014 backfill owns the localization later).
"""
import json
import os
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src/lib/i18n"

# Arabic translations
AR_VALUES = {
    "constellationSightV3": "Constellation Sight",
    "constellationSightV3Desc": "تصور المعرفة كخريطة نجوم — رؤية كامل العالم في لمحة",
    "sightV3.title": "Constellation Sight",
    "sightV3.placeholder": "Sight v3 — أساس الإسقاط (MIG-018 §1C)",
    "sightV3.close": "إغلاق Sight",
}

# English placeholder values for the 13 non-en/ar locales
EN_PLACEHOLDER = {
    "constellationSightV3": "Constellation Sight",
    "constellationSightV3Desc": "Star-chart knowledge visualization — see your entire universe at a glance",
    "sightV3.title": "Constellation Sight",
    "sightV3.placeholder": "Sight v3 — projection foundation (MIG-018 §1C)",
    "sightV3.close": "Close Sight",
}

LOCALES = ["ar", "de", "es", "fa", "fr", "he", "hi", "ja", "ko", "pt", "ru", "tr", "ur", "zh"]

for locale in LOCALES:
    path = LOCALES_DIR / f"{locale}.json"
    with open(path, encoding="utf-8") as f:
        data = json.load(f)

    values = AR_VALUES if locale == "ar" else EN_PLACEHOLDER

    # 1. Add to settings.plugins
    plugins = data.get("settings", {}).get("plugins", {})
    if "constellationSight" not in plugins:
        print(f"  [{locale}] WARN: settings.plugins.constellationSight not found; skipping")
        continue
    if "constellationSightV3" not in plugins:
        # Insert constellationSightV3 + Desc right after constellationSightDesc.
        # JSON dict-insertion order isn't preserved by assignment in Python <3.7;
        # since we're on 3.7+ we can rebuild the dict in order.
        new_plugins = {}
        for k, v in plugins.items():
            new_plugins[k] = v
            if k == "constellationSightDesc":
                new_plugins["constellationSightV3"] = values["constellationSightV3"]
                new_plugins["constellationSightV3Desc"] = values["constellationSightV3Desc"]
        data["settings"]["plugins"] = new_plugins

    # 2. Add top-level "sightV3" namespace before "lens"
    if "sightV3" not in data:
        new_root = {}
        for k, v in data.items():
            if k == "lens":
                new_root["sightV3"] = {
                    "title": values["sightV3.title"],
                    "placeholder": values["sightV3.placeholder"],
                    "close": values["sightV3.close"],
                }
            new_root[k] = v
        data = new_root

    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")  # trailing newline matches existing convention
    print(f"  [{locale}] OK")

print("Done — 14 locales updated.")
