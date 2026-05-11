#!/usr/bin/env python3
"""
MIG-022 §E.3.c — Backfill sources.evidence.* keys across 13 non-en/non-ar locales.

Same gap shape as V3-§10.D.2 (settings.classifier) and §E.1.1
(sources.review): 11 keys present in en+ar but missing from the
13 backfilled locales. Eisa surfaced this gap on the Stage 2.1
Spanish UI screenshot ("Reported by another knower" rendered in
English on the SOURCES strip).

Translations are deep tradition-specific (mutawatir, qiyas,
arthapatti, anupalabdhi, kashf, al-wahy, etc.). For non-Arabic
locales: keep the parenthetical Arabic/Sanskrit transliterations
as scholar-friendly anchors (they're recognized terms in
comparative epistemology); translate only the leading explanation.
"""
import json
from pathlib import Path

I18N_DIR = Path(r"E:\مشاريع كلاود\Constellation\src\lib\i18n")

TRANSLATIONS = {
    "de": {
        "perception": "Direkte sinnliche Wahrnehmung",
        "inference": "Schlussfolgerung von Prämissen zur Konklusion",
        "testimony": "Bericht eines anderen Wissenden",
        "mass-transmission": "Konvergenter Mehrzeugen-Konsens (al-tawatur)",
        "comparison": "Wissen durch Analogie (qiyas / upamana)",
        "postulation": "Schluss auf die beste Erklärung (arthapatti)",
        "non-apprehension": "Wissen über Abwesenheit (anupalabdhi)",
        "memory": "Erinnerung an vergangene Erfahrung (smrti)",
        "innate-disposition": "Vorerfahrungs-Intuition (fitrah / nous)",
        "inspiration": "Mystische oder kreative Erkenntnis (al-ilham)",
        "revelation": "Heilige-Text- oder prophetische Überlieferung (al-wahy)",
    },
    "es": {
        "perception": "Observación sensorial directa",
        "inference": "Razonamiento de premisas a conclusión",
        "testimony": "Reportado por otro conocedor",
        "mass-transmission": "Consenso convergente de múltiples testigos (al-tawatur)",
        "comparison": "Conocimiento por analogía (qiyas / upamana)",
        "postulation": "Inferencia a la mejor explicación (arthapatti)",
        "non-apprehension": "Conocimiento de la ausencia (anupalabdhi)",
        "memory": "Recuerdo de experiencia pasada (smrti)",
        "innate-disposition": "Intuición pre-experiencial (fitrah / nous)",
        "inspiration": "Aprehensión mística o creativa (al-ilham)",
        "revelation": "Transmisión de texto sagrado o profética (al-wahy)",
    },
    "fa": {
        "perception": "مشاهده‌ی حسی مستقیم",
        "inference": "استدلال از مقدمات به نتیجه",
        "testimony": "گزارش‌شده توسط دانا‌ی دیگر",
        "mass-transmission": "اجماع شاهدان متعدد متقارب (تواتر)",
        "comparison": "دانش از طریق قیاس (قیاس / اوپامانا)",
        "postulation": "استنباط به‌سوی بهترین تبیین (آرتاپَتی)",
        "non-apprehension": "دانش به عدم (انوپَلَبدی)",
        "memory": "یادآوری تجربه‌ی گذشته (سمریتی)",
        "innate-disposition": "شهود پیش‌تجربی (فطرت / نوس)",
        "inspiration": "ادراک عرفانی یا خلاق (الهام)",
        "revelation": "تبلیغ متن مقدس یا نبوی (وحی)",
    },
    "fr": {
        "perception": "Observation sensorielle directe",
        "inference": "Raisonnement des prémisses à la conclusion",
        "testimony": "Rapporté par un autre connaisseur",
        "mass-transmission": "Consensus convergent à témoins multiples (al-tawatur)",
        "comparison": "Connaissance par analogie (qiyas / upamana)",
        "postulation": "Inférence à la meilleure explication (arthapatti)",
        "non-apprehension": "Connaissance de l'absence (anupalabdhi)",
        "memory": "Souvenir d'une expérience passée (smrti)",
        "innate-disposition": "Intuition pré-expérientielle (fitrah / nous)",
        "inspiration": "Appréhension mystique ou créative (al-ilham)",
        "revelation": "Transmission scripturaire ou prophétique (al-wahy)",
    },
    "he": {
        "perception": "תצפית חושית ישירה",
        "inference": "הסקה מהנחות למסקנה",
        "testimony": "דווח על ידי יודע אחר",
        "mass-transmission": "הסכמה מתכנסת של עדים רבים (אל-תוואתור)",
        "comparison": "ידע על ידי אנלוגיה (קיאס / אופמאנה)",
        "postulation": "הסקה להסבר הטוב ביותר (ארתאפאתי)",
        "non-apprehension": "ידע על היעדרות (אנופלאבדהי)",
        "memory": "היזכרות בחוויה מן העבר (סמריטי)",
        "innate-disposition": "אינטואיציה טרום-חווייתית (פיטרה / נוּס)",
        "inspiration": "תפיסה מיסטית או יצירתית (אל-איל'האם)",
        "revelation": "מסירת טקסט קדוש או נבואי (אל-ווחי)",
    },
    "hi": {
        "perception": "प्रत्यक्ष इंद्रिय अवलोकन",
        "inference": "आधार से निष्कर्ष तक तर्क (अनुमान)",
        "testimony": "किसी अन्य ज्ञाता द्वारा रिपोर्ट (शब्द)",
        "mass-transmission": "अभिसारी बहु-साक्षी सहमति (अल-तवातुर)",
        "comparison": "सादृश्य से ज्ञान (क़ियास / उपमान)",
        "postulation": "सर्वोत्तम व्याख्या तक अनुमान (अर्थापत्ति)",
        "non-apprehension": "अनुपस्थिति का ज्ञान (अनुपलब्धि)",
        "memory": "अतीत के अनुभव की स्मृति (स्मृति)",
        "innate-disposition": "पूर्व-अनुभव अंतर्ज्ञान (फितरह / नूस)",
        "inspiration": "रहस्यमय या रचनात्मक बोध (अल-इल्हाम)",
        "revelation": "पवित्र-पाठ या प्रवृति-संचरण (अल-वही)",
    },
    "ja": {
        "perception": "直接的な感覚的観察",
        "inference": "前提から結論への推論",
        "testimony": "他の知者による報告",
        "mass-transmission": "収束する多証人の合意 (al-tawatur)",
        "comparison": "類推による知識 (qiyas / upamana)",
        "postulation": "最良の説明への推論 (arthapatti)",
        "non-apprehension": "不在の知識 (anupalabdhi)",
        "memory": "過去の経験の想起 (smrti)",
        "innate-disposition": "経験以前の直観 (fitrah / nous)",
        "inspiration": "神秘的または創造的な把握 (al-ilham)",
        "revelation": "聖典または預言的伝承 (al-wahy)",
    },
    "ko": {
        "perception": "직접적인 감각 관찰",
        "inference": "전제에서 결론으로의 추론",
        "testimony": "다른 지식인의 보고",
        "mass-transmission": "수렴적 다중 증인 합의 (al-tawatur)",
        "comparison": "유추에 의한 지식 (qiyas / upamana)",
        "postulation": "최선의 설명으로의 추론 (arthapatti)",
        "non-apprehension": "부재의 지식 (anupalabdhi)",
        "memory": "과거 경험의 회상 (smrti)",
        "innate-disposition": "경험 이전의 직관 (fitrah / nous)",
        "inspiration": "신비적 또는 창조적 통찰 (al-ilham)",
        "revelation": "성스러운 문헌 또는 예언적 전승 (al-wahy)",
    },
    "pt": {
        "perception": "Observação sensorial direta",
        "inference": "Raciocínio das premissas à conclusão",
        "testimony": "Relatado por outro conhecedor",
        "mass-transmission": "Consenso convergente de múltiplas testemunhas (al-tawatur)",
        "comparison": "Conhecimento por analogia (qiyas / upamana)",
        "postulation": "Inferência à melhor explicação (arthapatti)",
        "non-apprehension": "Conhecimento da ausência (anupalabdhi)",
        "memory": "Recordação de experiência passada (smrti)",
        "innate-disposition": "Intuição pré-experiencial (fitrah / nous)",
        "inspiration": "Apreensão mística ou criativa (al-ilham)",
        "revelation": "Transmissão de texto sagrado ou profética (al-wahy)",
    },
    "ru": {
        "perception": "Непосредственное чувственное наблюдение",
        "inference": "Рассуждение от посылок к выводу",
        "testimony": "Сообщено другим знающим",
        "mass-transmission": "Сходящийся консенсус многих свидетелей (al-tawatur)",
        "comparison": "Знание через аналогию (qiyas / upamana)",
        "postulation": "Заключение к наилучшему объяснению (arthapatti)",
        "non-apprehension": "Знание об отсутствии (anupalabdhi)",
        "memory": "Воспоминание о прошлом опыте (smrti)",
        "innate-disposition": "Доопытная интуиция (fitrah / nous)",
        "inspiration": "Мистическое или творческое постижение (al-ilham)",
        "revelation": "Передача священного текста или пророческая (al-wahy)",
    },
    "tr": {
        "perception": "Doğrudan duyusal gözlem",
        "inference": "Öncüllerden sonuca akıl yürütme",
        "testimony": "Başka bir bilen tarafından bildirilen (haber)",
        "mass-transmission": "Yakınsayan çok-tanıklı uzlaşı (tevatür)",
        "comparison": "Analoji ile bilgi (kıyas / upamana)",
        "postulation": "En iyi açıklamaya çıkarım (arthapatti)",
        "non-apprehension": "Yokluk bilgisi (anupalabdhi)",
        "memory": "Geçmiş deneyimin hatırlanması (smrti)",
        "innate-disposition": "Deneyim öncesi sezgi (fıtrat / nous)",
        "inspiration": "Mistik veya yaratıcı kavrayış (ilham)",
        "revelation": "Kutsal metin veya peygamberî aktarım (vahiy)",
    },
    "ur": {
        "perception": "براہ راست حسی مشاہدہ",
        "inference": "مقدمات سے نتیجے تک استدلال",
        "testimony": "کسی دوسرے جاننے والے کی روایت (خبر)",
        "mass-transmission": "متعدد گواہوں کی متفق اجماع (تواتر)",
        "comparison": "مماثلت سے علم (قیاس / اپمان)",
        "postulation": "بہترین وضاحت تک استنباط (ارتھاپتی)",
        "non-apprehension": "غیر موجودگی کا علم (انوپلابدھی)",
        "memory": "گزشتہ تجربے کی یاد (سمرتی)",
        "innate-disposition": "تجرباتی پیش وجدان (فطرت / نوس)",
        "inspiration": "عرفانی یا تخلیقی ادراک (الہام)",
        "revelation": "مقدس متن یا نبوی نقل (وحی)",
    },
    "zh": {
        "perception": "直接的感官观察",
        "inference": "从前提到结论的推理",
        "testimony": "另一位知者的报告",
        "mass-transmission": "多证人趋同共识 (al-tawatur)",
        "comparison": "通过类比的知识 (qiyas / upamana)",
        "postulation": "最佳解释推论 (arthapatti)",
        "non-apprehension": "对不在的认识 (anupalabdhi)",
        "memory": "对过去经验的回忆 (smrti)",
        "innate-disposition": "经验前直观 (fitrah / nous)",
        "inspiration": "神秘或创造性领悟 (al-ilham)",
        "revelation": "圣典或先知传承 (al-wahy)",
    },
}


def main():
    expected_keys = set(TRANSLATIONS["de"].keys())  # canonical 11
    for locale, trans in TRANSLATIONS.items():
        path = I18N_DIR / f"{locale}.json"
        if not path.exists():
            print(f"SKIP {locale}: file not found")
            continue
        if set(trans.keys()) != expected_keys:
            missing = expected_keys - set(trans.keys())
            extra = set(trans.keys()) - expected_keys
            print(f"FAIL {locale}: missing={missing} extra={extra}")
            continue
        with path.open("r", encoding="utf-8") as f:
            data = json.load(f)
        sources = data.setdefault("sources", {})
        existing = sources.setdefault("evidence", {})
        added = 0
        for key in trans:
            if key not in existing:
                existing[key] = trans[key]
                added += 1
        with path.open("w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
            f.write("\n")
        print(f"OK   {locale}: sources.evidence backfilled (+{added} keys)")


if __name__ == "__main__":
    main()
