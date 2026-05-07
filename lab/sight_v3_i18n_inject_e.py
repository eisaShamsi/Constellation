"""MIG-018 §1E i18n — inject sightV3.tooltip.* and sightV3.sidePanel.* keys
into all 15 locales. ar gets Arabic translations; the other 13 get English
placeholders (PJ-014 backfill).
"""
import json
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src/lib/i18n"

AR = {
    "tooltip.community": "المجتمع",
    "tooltip.centralityRank": "رتبة المركزية",
    "tooltip.lifecycle": "المرحلة",
    "sidePanel.community": "المجتمع",
    "sidePanel.centralityRank": "رتبة المركزية",
    "sidePanel.connections": "الروابط",
    "sidePanel.incomingLinks": "روابط واردة",
    "sidePanel.outgoingLinks": "روابط صادرة",
    "sidePanel.openNote": "فتح في المحرر",
    "sidePanel.close": "إغلاق اللوحة",
    "sidePanel.structuralGaps": "اقتراحات روابط مفقودة",
    "placeholder": "Sight v3 — أساس الإسقاط (MIG-018 §1E)",
}

EN = {
    "tooltip.community": "Community",
    "tooltip.centralityRank": "Centrality rank",
    "tooltip.lifecycle": "Lifecycle",
    "sidePanel.community": "Community",
    "sidePanel.centralityRank": "Centrality rank",
    "sidePanel.connections": "Connections",
    "sidePanel.incomingLinks": "Incoming links",
    "sidePanel.outgoingLinks": "Outgoing links",
    "sidePanel.openNote": "Open in editor",
    "sidePanel.close": "Close panel",
    "sidePanel.structuralGaps": "Structural gap suggestions",
    "placeholder": "Sight v3 — projection foundation (MIG-018 §1E)",
}

LOCALES = ["ar", "de", "en", "es", "fa", "fr", "he", "hi", "ja", "ko", "pt", "ru", "tr", "ur", "zh"]

for locale in LOCALES:
    path = LOCALES_DIR / f"{locale}.json"
    with open(path, encoding="utf-8") as f:
        data = json.load(f)

    values = AR if locale == "ar" else EN

    if "sightV3" not in data:
        print(f"  [{locale}] WARN: sightV3 namespace missing; skipping")
        continue

    sv3 = data["sightV3"]

    # tooltip sub-namespace
    sv3.setdefault("tooltip", {})
    sv3["tooltip"]["community"] = values["tooltip.community"]
    sv3["tooltip"]["centralityRank"] = values["tooltip.centralityRank"]
    sv3["tooltip"]["lifecycle"] = values["tooltip.lifecycle"]

    # sidePanel sub-namespace
    sv3.setdefault("sidePanel", {})
    sv3["sidePanel"]["community"] = values["sidePanel.community"]
    sv3["sidePanel"]["centralityRank"] = values["sidePanel.centralityRank"]
    sv3["sidePanel"]["connections"] = values["sidePanel.connections"]
    sv3["sidePanel"]["incomingLinks"] = values["sidePanel.incomingLinks"]
    sv3["sidePanel"]["outgoingLinks"] = values["sidePanel.outgoingLinks"]
    sv3["sidePanel"]["openNote"] = values["sidePanel.openNote"]
    sv3["sidePanel"]["close"] = values["sidePanel.close"]
    sv3["sidePanel"]["structuralGaps"] = values["sidePanel.structuralGaps"]

    # placeholder bump §1D → §1E
    sv3["placeholder"] = values["placeholder"]

    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"  [{locale}] OK")

print("Done — 15 locales updated.")
