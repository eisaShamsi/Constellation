"""MIG-019 §2C i18n — calendar rim labels + Settings entries.
Inject into all 15 locales. ar gets Arabic; the other 13 get English placeholders.
"""
import json
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src/lib/i18n"

AR = {
    "calendarSystems.label": "أنظمة التقويم",
    "calendarSystems.hint": "كل نظام مفعّل يضيف حلقة متحدة المركز من علامات الأشهر حول القبة. الأول في القائمة هو الأقرب إلى المركز.",
    "calendarSystems.gregorian": "ميلادي",
    "calendarSystems.hijri": "هجري (إسلامي)",
    "calendarSystems.solarHijri": "هجري شمسي",
    "calendarSystems.hebrew": "عبري",
    "calendarSystems.placeholderNote": "* الهجري الشمسي والعبري يعرضان أسماء الأشهر الميلادية مع علامة عنصر نائب حتى يتم إصدار التعريب الكامل في PJ-014.",
}

EN = {
    "calendarSystems.label": "Calendar systems",
    "calendarSystems.hint": "Each enabled system adds a concentric ring of month markers around the dome. First in the list is innermost.",
    "calendarSystems.gregorian": "Gregorian",
    "calendarSystems.hijri": "Hijri (Islamic)",
    "calendarSystems.solarHijri": "Solar Hijri",
    "calendarSystems.hebrew": "Hebrew",
    "calendarSystems.placeholderNote": "* Solar Hijri and Hebrew render Gregorian month names with a placeholder marker until PJ-014 backfill ships full localization.",
}

LOCALES = ["ar", "de", "en", "es", "fa", "fr", "he", "hi", "ja", "ko", "pt", "ru", "tr", "ur", "zh"]

for locale in LOCALES:
    path = LOCALES_DIR / f"{locale}.json"
    with open(path, encoding="utf-8") as f:
        data = json.load(f)

    values = AR if locale == "ar" else EN

    sight = data.get("settings", {}).get("sight", {})
    if not sight:
        print(f"  [{locale}] WARN: settings.sight missing; skipping")
        continue

    sight.setdefault("calendarSystems", {})
    cs = sight["calendarSystems"]
    cs["label"] = values["calendarSystems.label"]
    cs["hint"] = values["calendarSystems.hint"]
    cs["gregorian"] = values["calendarSystems.gregorian"]
    cs["hijri"] = values["calendarSystems.hijri"]
    cs["solarHijri"] = values["calendarSystems.solarHijri"]
    cs["hebrew"] = values["calendarSystems.hebrew"]
    cs["placeholderNote"] = values["calendarSystems.placeholderNote"]

    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"  [{locale}] OK")

print("Done — 15 locales updated.")
