#!/usr/bin/env python3
"""Add linkDashboard.mostTraveled + noTraveled keys to all 15 locales."""
import json
from pathlib import Path

TRANSLATIONS = {
    "en": ("Most Traveled", "No traveled paths yet"),
    "ar": ("الأكثر ترددًا", "لا توجد مسارات مطروقة بعد"),
    "de": ("Meist begangen", "Noch keine begangenen Pfade"),
    "es": ("Más transitados", "Aún no hay rutas transitadas"),
    "fa": ("پرتردّدترین", "هنوز مسیر پرتردّدی نیست"),
    "fr": ("Les plus empruntés", "Aucun chemin emprunté"),
    "he": ("הנפוצים ביותר", "אין נתיבים נפוצים עדיין"),
    "hi": ("सबसे अधिक यात्रा", "अभी तक कोई यात्रा पथ नहीं"),
    "ja": ("最多通過", "よく通るパスはまだありません"),
    "ko": ("가장 많이 이동", "자주 다닌 경로가 없습니다"),
    "pt": ("Mais percorridos", "Nenhum caminho percorrido"),
    "ru": ("Наиболее пройденные", "Пока нет пройденных путей"),
    "tr": ("En Çok Geçilen", "Henüz geçilen yol yok"),
    "ur": ("سب سے زیادہ چلا ہوا", "ابھی کوئی چلا ہوا راستہ نہیں"),
    "zh": ("最常通行", "暂无常走路径"),
}

root = Path(__file__).resolve().parents[2] / "src" / "lib" / "i18n"
for locale, (traveled, none_yet) in TRANSLATIONS.items():
    p = root / f"{locale}.json"
    data = json.loads(p.read_text(encoding="utf-8"))
    ld = data.setdefault("linkDashboard", {})
    ld["mostTraveled"] = traveled
    ld["noTraveled"] = none_yet
    p.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{locale}: mostTraveled={traveled!r}, noTraveled={none_yet!r}")
