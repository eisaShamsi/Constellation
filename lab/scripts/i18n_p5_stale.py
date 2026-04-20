#!/usr/bin/env python3
"""P5: add linkDashboard.stale + noStale + relative-age strings to all 15 locales."""
import json
from pathlib import Path

# Each locale: (stale tab label, empty state, {n} days, {n} months, {n} years)
TRANSLATIONS = {
    "en": ("Stale", "No stale links — every traversed path is fresh", "{n}d ago", "{n}mo ago", "{n}y ago"),
    "ar": ("مهجورة", "لا توجد روابط مهجورة — كل مسار مطروق لا يزال طازجًا", "منذ {n} يوم", "منذ {n} شهر", "منذ {n} سنة"),
    "de": ("Veraltet", "Keine veralteten Links — jeder begangene Pfad ist frisch", "vor {n} T", "vor {n} Mo", "vor {n} J"),
    "es": ("Obsoletos", "Sin enlaces obsoletos — cada ruta transitada está fresca", "hace {n} d", "hace {n} me", "hace {n} a"),
    "fa": ("منسوخ", "هیچ پیوند منسوخی نیست — همهٔ مسیرهای پیموده‌شده تازه‌اند", "{n} روز پیش", "{n} ماه پیش", "{n} سال پیش"),
    "fr": ("Obsolètes", "Aucun lien obsolète — chaque chemin emprunté est récent", "il y a {n} j", "il y a {n} mo", "il y a {n} a"),
    "he": ("מיושנים", "אין קישורים מיושנים — כל נתיב שנפרץ עדיין טרי", "לפני {n} ימים", "לפני {n} חודשים", "לפני {n} שנים"),
    "hi": ("पुराने", "कोई पुरानी कड़ी नहीं — प्रत्येक चला हुआ पथ अभी भी सक्रिय है", "{n} दिन पहले", "{n} माह पहले", "{n} वर्ष पहले"),
    "ja": ("休眠中", "休眠中のリンクはありません — 通ったパスはすべて活動中です", "{n}日前", "{n}ヶ月前", "{n}年前"),
    "ko": ("오래됨", "오래된 링크 없음 — 모든 이동 경로가 신선합니다", "{n}일 전", "{n}개월 전", "{n}년 전"),
    "pt": ("Desatualizados", "Sem ligações desatualizadas — cada caminho percorrido está fresco", "há {n}d", "há {n}m", "há {n}a"),
    "ru": ("Устаревшие", "Нет устаревших ссылок — каждый пройденный путь свежий", "{n} дн. назад", "{n} мес. назад", "{n} г. назад"),
    "tr": ("Eskimiş", "Eskimiş bağlantı yok — her geçilen yol hâlâ taze", "{n} gün önce", "{n} ay önce", "{n} yıl önce"),
    "ur": ("متروک", "کوئی متروک لنک نہیں — ہر چلا ہوا راستہ اب بھی تازہ ہے", "{n} دن پہلے", "{n} مہینے پہلے", "{n} سال پہلے"),
    "zh": ("已冷落", "没有冷落的链接 — 每条走过的路径都仍在活跃", "{n}天前", "{n}个月前", "{n}年前"),
}

root = Path(__file__).resolve().parents[2] / "src" / "lib" / "i18n"
for locale, (stale, no_stale, d, m, y) in TRANSLATIONS.items():
    p = root / f"{locale}.json"
    data = json.loads(p.read_text(encoding="utf-8"))
    ld = data.setdefault("linkDashboard", {})
    ld["stale"] = stale
    ld["noStale"] = no_stale
    ld["staleDays"] = d
    ld["staleMonths"] = m
    ld["staleYears"] = y
    p.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{locale}: stale={stale!r}")
