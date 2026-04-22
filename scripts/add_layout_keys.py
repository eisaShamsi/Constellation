"""Add three layout.* i18n keys to all 15 locales."""
import json
import os
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src" / "lib" / "i18n"

TRANSLATIONS = {
    "en": {"disabledFullPage": "Disabled in full-page view", "disabledInSkyInspect": "Disabled while inspecting a Sky View note", "exitSkyViewMode": "Exit Sky View inspect mode"},
    "ar": {"disabledFullPage": "معطل في عرض الصفحة الكاملة", "disabledInSkyInspect": "معطل أثناء فحص ملاحظة من عرض السماء", "exitSkyViewMode": "الخروج من وضع فحص عرض السماء"},
    "de": {"disabledFullPage": "In der Vollbildansicht deaktiviert", "disabledInSkyInspect": "Während der Sky-View-Inspektion deaktiviert", "exitSkyViewMode": "Sky-View-Inspektion beenden"},
    "es": {"disabledFullPage": "Desactivado en vista de página completa", "disabledInSkyInspect": "Desactivado al inspeccionar una nota de Sky View", "exitSkyViewMode": "Salir del modo inspección de Sky View"},
    "fa": {"disabledFullPage": "در نمای تمام‌صفحه غیرفعال است", "disabledInSkyInspect": "هنگام بررسی یادداشت Sky View غیرفعال است", "exitSkyViewMode": "خروج از حالت بررسی Sky View"},
    "fr": {"disabledFullPage": "Désactivé en vue pleine page", "disabledInSkyInspect": "Désactivé lors de l'inspection d'une note Sky View", "exitSkyViewMode": "Quitter le mode inspection Sky View"},
    "he": {"disabledFullPage": "מושבת בתצוגת עמוד מלא", "disabledInSkyInspect": "מושבת בעת בחינת פתק Sky View", "exitSkyViewMode": "יציאה ממצב בחינת Sky View"},
    "hi": {"disabledFullPage": "पूर्ण-पृष्ठ दृश्य में अक्षम", "disabledInSkyInspect": "स्काई व्यू नोट का निरीक्षण करते समय अक्षम", "exitSkyViewMode": "स्काई व्यू निरीक्षण मोड से बाहर निकलें"},
    "ja": {"disabledFullPage": "全画面表示中は無効", "disabledInSkyInspect": "Sky View ノートの検査中は無効", "exitSkyViewMode": "Sky View 検査モードを終了"},
    "ko": {"disabledFullPage": "전체 페이지 보기에서 비활성화됨", "disabledInSkyInspect": "Sky View 노트 검사 중 비활성화됨", "exitSkyViewMode": "Sky View 검사 모드 종료"},
    "pt": {"disabledFullPage": "Desativado na visualização de página inteira", "disabledInSkyInspect": "Desativado ao inspecionar uma nota do Sky View", "exitSkyViewMode": "Sair do modo de inspeção Sky View"},
    "ru": {"disabledFullPage": "Отключено в полноэкранном режиме", "disabledInSkyInspect": "Отключено при инспекции заметки Sky View", "exitSkyViewMode": "Выйти из режима инспекции Sky View"},
    "tr": {"disabledFullPage": "Tam sayfa görünümünde devre dışı", "disabledInSkyInspect": "Sky View notu incelenirken devre dışı", "exitSkyViewMode": "Sky View inceleme modundan çık"},
    "ur": {"disabledFullPage": "مکمل صفحہ ویو میں غیر فعال", "disabledInSkyInspect": "Sky View نوٹ کا معائنہ کرتے وقت غیر فعال", "exitSkyViewMode": "Sky View معائنہ موڈ سے باہر نکلیں"},
    "zh": {"disabledFullPage": "全页视图中已禁用", "disabledInSkyInspect": "检查 Sky View 笔记时已禁用", "exitSkyViewMode": "退出 Sky View 检查模式"},
}

for lang, keys in TRANSLATIONS.items():
    fp = LOCALES_DIR / f"{lang}.json"
    with open(fp, "r", encoding="utf-8") as f:
        data = json.load(f)
    if "layout" not in data:
        print(f"WARN: no 'layout' namespace in {lang}.json, skipping")
        continue
    for k, v in keys.items():
        data["layout"][k] = v
    with open(fp, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"  {lang}.json: +{len(keys)} keys")

print("done")
