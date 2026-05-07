"""MIG-019 §2E i18n — always-on labels toggle.
Inject into all 15 locales. ar gets Arabic; the other 13 get English placeholders.
"""
import json
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src/lib/i18n"

AR = {
    "labels.label": "التسميات",
    "alwaysOnLabels.label": "عرض تسميات الكوكبات بشكل دائم",
    "alwaysOnLabels.hint": "إيقاف (افتراضي): تظهر التسميات عند التحويم أو التحديد. تشغيل: تبقى التسميات مرئية في مراكز المناطق.",
}

EN = {
    "labels.label": "Labels",
    "alwaysOnLabels.label": "Show constellation labels at rest",
    "alwaysOnLabels.hint": "Off (default): labels appear on hover or selection. On: labels stay visible at territory centroids.",
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

    sight.setdefault("labels", {})
    sight["labels"]["label"] = values["labels.label"]

    sight.setdefault("alwaysOnLabels", {})
    sight["alwaysOnLabels"]["label"] = values["alwaysOnLabels.label"]
    sight["alwaysOnLabels"]["hint"] = values["alwaysOnLabels.hint"]

    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"  [{locale}] OK")

print("Done — 15 locales updated.")
