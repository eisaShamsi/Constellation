"""Add `linkTypes` translations (the eight KNOWN_LINK_TYPES names) to
every locale so the Backlinks / Outgoing Links badges, plus the
Settings → Living Link Pills preview labels, show the localized
name of each typed link instead of the raw English identifier.

The internal key stays English (lowercase, ASCII, e.g. `derives-from`)
for data & logic — only the DISPLAY uses these translations.
"""
import json, os

TR = {
    'en': {'supports':'supports','contradicts':'contradicts','causes':'causes','exemplifies':'exemplifies','generalizes':'generalizes','derives-from':'derives-from','part-of':'part-of','associative':'associative'},
    'ar': {'supports':'يدعم','contradicts':'يناقض','causes':'يسبب','exemplifies':'يمثّل','generalizes':'يعمّم','derives-from':'مشتق من','part-of':'جزء من','associative':'ترابطي'},
    'de': {'supports':'unterstützt','contradicts':'widerspricht','causes':'verursacht','exemplifies':'veranschaulicht','generalizes':'verallgemeinert','derives-from':'abgeleitet-von','part-of':'teil-von','associative':'assoziativ'},
    'es': {'supports':'apoya','contradicts':'contradice','causes':'causa','exemplifies':'ejemplifica','generalizes':'generaliza','derives-from':'deriva-de','part-of':'parte-de','associative':'asociativo'},
    'fa': {'supports':'پشتیبانی','contradicts':'نقض','causes':'سبب','exemplifies':'مثال','generalizes':'تعمیم','derives-from':'مشتق از','part-of':'بخشی از','associative':'تداعی'},
    'fr': {'supports':'soutient','contradicts':'contredit','causes':'cause','exemplifies':'illustre','generalizes':'généralise','derives-from':'dérivé-de','part-of':'partie-de','associative':'associatif'},
    'he': {'supports':'תומך','contradicts':'סותר','causes':'גורם','exemplifies':'מדגים','generalizes':'מכליל','derives-from':'נגזר-מ','part-of':'חלק-מ','associative':'אסוציאטיבי'},
    'hi': {'supports':'समर्थन','contradicts':'विरोधाभास','causes':'कारण','exemplifies':'उदाहरण','generalizes':'सामान्यीकरण','derives-from':'व्युत्पन्न','part-of':'का-हिस्सा','associative':'साहचर्य'},
    'ja': {'supports':'支持','contradicts':'矛盾','causes':'原因','exemplifies':'例示','generalizes':'一般化','derives-from':'派生','part-of':'一部','associative':'連想'},
    'ko': {'supports':'지지','contradicts':'모순','causes':'유발','exemplifies':'예시','generalizes':'일반화','derives-from':'파생','part-of':'일부','associative':'연관'},
    'pt': {'supports':'apoia','contradicts':'contradiz','causes':'causa','exemplifies':'exemplifica','generalizes':'generaliza','derives-from':'deriva-de','part-of':'parte-de','associative':'associativo'},
    'ru': {'supports':'поддерживает','contradicts':'противоречит','causes':'вызывает','exemplifies':'иллюстрирует','generalizes':'обобщает','derives-from':'происходит-от','part-of':'часть-от','associative':'ассоциативный'},
    'tr': {'supports':'destekler','contradicts':'çelişir','causes':'neden-olur','exemplifies':'örnekler','generalizes':'genelleştirir','derives-from':'türetilir','part-of':'parçası','associative':'ilişkili'},
    'ur': {'supports':'حمایت','contradicts':'تضاد','causes':'سبب','exemplifies':'مثال','generalizes':'عمومی','derives-from':'ماخوذ','part-of':'حصہ','associative':'متعلق'},
    'zh': {'supports':'支持','contradicts':'反驳','causes':'导致','exemplifies':'举例','generalizes':'概括','derives-from':'源自','part-of':'部分','associative':'关联'},
}

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', 'src', 'lib', 'i18n'))
for lang, keys in TR.items():
    path = os.path.join(ROOT, lang + '.json')
    with open(path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    data['linkTypes'] = keys
    with open(path, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write('\n')
    print('updated', lang)
