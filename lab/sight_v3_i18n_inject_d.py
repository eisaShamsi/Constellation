"""MIG-018 §1D i18n — inject settings.sections.sight + settings.sight.* keys
into all 15 locales (ar gets Arabic translations; the 13 others get English
placeholders, PJ-014 backfill territory).

Also bumps sightV3.placeholder string from §1C to §1D (the only en/ar string
that mentions the phase number).
"""
import json
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src/lib/i18n"

# Arabic translations
AR_VALUES = {
    "settings.sections.sight": "Sight",
    "settings.sight.intro": "Constellation Sight v3 — تصور خريطة النجوم لعالم المعرفة لديك.",
    "settings.sight.projection.label": "الإسقاط",
    "settings.sight.projection.lambert": "Lambert (متساوي المساحة)",
    "settings.sight.projection.lambertHint": "أحجام المجتمعات متناسبة مع عدد العقد.",
    "settings.sight.projection.stereographic": "Stereographic (متساوي الزوايا)",
    "settings.sight.projection.stereographicHint": "أشكال الكوكبات محفوظة؛ الأحجام مضللة بالقرب من الحواف.",
    "sightV3.placeholder": "Sight v3 — أساس الإسقاط (MIG-018 §1D)",
}

# English placeholders
EN_PLACEHOLDER = {
    "settings.sections.sight": "Sight",
    "settings.sight.intro": "Constellation Sight v3 — star-chart visualization of your knowledge universe.",
    "settings.sight.projection.label": "Projection",
    "settings.sight.projection.lambert": "Lambert (equal-area)",
    "settings.sight.projection.lambertHint": "Community sizes visually proportional to node count.",
    "settings.sight.projection.stereographic": "Stereographic (equal-angle)",
    "settings.sight.projection.stereographicHint": "Constellation shapes preserved; sizes mislead near edges.",
    "sightV3.placeholder": "Sight v3 — projection foundation (MIG-018 §1D)",
}

LOCALES = ["ar", "de", "en", "es", "fa", "fr", "he", "hi", "ja", "ko", "pt", "ru", "tr", "ur", "zh"]

for locale in LOCALES:
    path = LOCALES_DIR / f"{locale}.json"
    with open(path, encoding="utf-8") as f:
        data = json.load(f)

    values = AR_VALUES if locale == "ar" else EN_PLACEHOLDER

    # 1. settings.sections.sight (after 'skyview')
    sections = data.get("settings", {}).get("sections", {})
    if "skyview" in sections and "sight" not in sections:
        new_sections = {}
        for k, v in sections.items():
            new_sections[k] = v
            if k == "skyview":
                new_sections["sight"] = values["settings.sections.sight"]
        data["settings"]["sections"] = new_sections

    # 2. settings.sight (new sub-namespace)
    if "sight" not in data["settings"]:
        data["settings"]["sight"] = {
            "intro": values["settings.sight.intro"],
            "projection": {
                "label": values["settings.sight.projection.label"],
                "lambert": values["settings.sight.projection.lambert"],
                "lambertHint": values["settings.sight.projection.lambertHint"],
                "stereographic": values["settings.sight.projection.stereographic"],
                "stereographicHint": values["settings.sight.projection.stereographicHint"],
            },
        }

    # 3. Update sightV3.placeholder to §1D
    if "sightV3" in data and "placeholder" in data["sightV3"]:
        data["sightV3"]["placeholder"] = values["sightV3.placeholder"]

    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"  [{locale}] OK")

print("Done — 15 locales updated.")
