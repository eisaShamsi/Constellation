"""Add compute-now / computing i18n keys to all 15 locales."""
import json
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src" / "lib" / "i18n"

TRANSLATIONS = {
    "en": {"computeNow": "Compute now", "computing": "Computing\u2026"},
    "ar": {"computeNow": "\u0627\u062d\u0633\u0628 \u0627\u0644\u0622\u0646", "computing": "\u062c\u0627\u0631\u064d \u0627\u0644\u062d\u0633\u0627\u0628\u2026"},
    "de": {"computeNow": "Jetzt berechnen", "computing": "Berechne\u2026"},
    "es": {"computeNow": "Calcular ahora", "computing": "Calculando\u2026"},
    "fa": {"computeNow": "\u0627\u06a9\u0646\u0648\u0646 \u0645\u062d\u0627\u0633\u0628\u0647 \u06a9\u0646", "computing": "\u062f\u0631 \u062d\u0627\u0644 \u0645\u062d\u0627\u0633\u0628\u0647\u2026"},
    "fr": {"computeNow": "Calculer maintenant", "computing": "Calcul\u2026"},
    "he": {"computeNow": "\u05d7\u05e9\u05d1 \u05db\u05e2\u05ea", "computing": "\u05de\u05d7\u05e9\u05d1\u2026"},
    "hi": {"computeNow": "\u0905\u092d\u0940 \u0917\u0923\u0928\u093e \u0915\u0930\u0947\u0902", "computing": "\u0917\u0923\u0928\u093e \u0939\u094b \u0930\u0939\u0940 \u0939\u0948\u2026"},
    "ja": {"computeNow": "\u4eca\u3059\u3050\u8a08\u7b97", "computing": "\u8a08\u7b97\u4e2d\u2026"},
    "ko": {"computeNow": "\uc9c0\uae08 \uacc4\uc0b0", "computing": "\uacc4\uc0b0 \uc911\u2026"},
    "pt": {"computeNow": "Calcular agora", "computing": "Calculando\u2026"},
    "ru": {"computeNow": "\u0412\u044b\u0447\u0438\u0441\u043b\u0438\u0442\u044c", "computing": "\u0412\u044b\u0447\u0438\u0441\u043b\u0435\u043d\u0438\u0435\u2026"},
    "tr": {"computeNow": "\u015eimdi hesapla", "computing": "Hesaplan\u0131yor\u2026"},
    "ur": {"computeNow": "ابھی حساب لگائیں", "computing": "حساب ہو رہا ہے…"},
    "zh": {"computeNow": "\u7acb\u5373\u8ba1\u7b97", "computing": "\u8ba1\u7b97\u4e2d\u2026"},
}

for lang, keys in TRANSLATIONS.items():
    fp = LOCALES_DIR / f"{lang}.json"
    with open(fp, "r", encoding="utf-8") as f:
        data = json.load(f)
    ns = data.get("graphView")
    if not isinstance(ns, dict):
        print(f"WARN: no 'graphView' namespace in {lang}.json, skipping")
        continue
    for k, v in keys.items():
        ns[k] = v
    with open(fp, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"  {lang}.json: +{len(keys)} keys")

print("done")
