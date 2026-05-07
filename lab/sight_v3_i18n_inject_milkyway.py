"""MIG-019 §2B i18n — inject settings.sight.density + settings.sight.showMilkyWay
keys into all 15 locales. ar gets Arabic; the other 13 get English placeholders
(PJ-014 backfill territory).
"""
import json
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src/lib/i18n"

AR = {
    "density.label": "الكثافة",
    "showMilkyWay.label": "موجة كثافة درب التبانة",
    "showMilkyWay.hint": "شريط ناعم من النسيج بين الملاحظات ذات المحتوى المتشابه (TF-IDF). إيقاف = نجوم فقط.",
}

EN = {
    "density.label": "Density",
    "showMilkyWay.label": "Milky Way density wash",
    "showMilkyWay.hint": "Soft band of texture between notes with similar content (TF-IDF). Off = stars-only.",
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

    sight.setdefault("density", {})
    sight["density"]["label"] = values["density.label"]

    sight.setdefault("showMilkyWay", {})
    sight["showMilkyWay"]["label"] = values["showMilkyWay.label"]
    sight["showMilkyWay"]["hint"] = values["showMilkyWay.hint"]

    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"  [{locale}] OK")

print("Done — 15 locales updated.")
