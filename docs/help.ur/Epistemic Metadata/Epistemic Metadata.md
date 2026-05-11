# معرفتی میٹا ڈیٹا

> **ترجمے کا نوٹ:** یہ مدد کا موضوع
> `help.uConstellation.World/Epistemic Metadata/Epistemic Metadata.md` پر موجود
> اصل انگریزی نسخے سے AI کے ذریعے تیار کردہ ترجمہ ہے۔ مادری زبان
> بولنے والوں کی نظرِ ثانی ابھی باقی ہے۔ براہ کرم اصلاحات
> پروجیکٹ کے ریپازٹری کے ذریعے بھیجیں۔

*(MIG-022 §A — gap-analysis §6.1 میں سکیما توسیعات)*

یہ موضوع **اختیاری frontmatter فیلڈز** کے ایک چھوٹے مجموعے کی وضاحت کرتا ہے جنہیں Constellation اب آپ کے نوٹس کی زیادہ بھرپور معرفتی درجہ بندی کے لیے پہچانتا ہے۔ یہ gap analysis (`docs/epistemic-content-gap-analysis.md`) کے جواب میں شامل کی گئیں — یہ تسلیم کہ دو محوری Source × Content Type ماڈل جس کی بنیاد پر Constellation Epistemic Content Engine (CECE) درجہ بندی کرتا ہے، ہر اس چیز کا احاطہ نہیں کرتا جسے آپ ریکارڈ کرنا چاہیں کہ آپ نے جو جانا ہے وہ کیسے جانا۔

یہ فیلڈز **سب اختیاری ہیں**۔ موجودہ نوٹس ان کے بغیر بدستور کام کرتے ہیں۔ آپ انہیں ہاتھ سے (یا مستقبل میں ساختیاتی ایڈیٹر کے ذریعے) اس وقت شامل کرتے ہیں جب نوٹ اس قسم کا علم ہو جو اس اضافی اشارے سے فائدہ اٹھاتا ہے۔

---

## فیلڈز

### `held_by` — *یہ موقف کس کا ہے؟*

ایک مختصر سٹرنگ جو ظاہر کرتی ہے کہ نوٹ جس موقف کو بیان کرتا ہے اسے کون رکھتا ہے۔ پہلے سے طے شدہ `user` (آپ کا اپنا موقف)۔ دیگر ممکنہ اقدار:
- کسی عالم کا نام: `held_by: "al-Shāfiʿī"`
- ایک مذہب: `held_by: "Ḥanafī"`
- ایک تاریخی شخصیت: `held_by: "Aristotle"`

جب آپ ایسا نوٹ لکھتے ہیں جو *کسی اور کا* موقف ریکارڈ کرتا ہے نہ کہ آپ کا، تو `held_by` وہ فیلڈ ہے جو یہ کہتا ہے۔ اس کے بغیر، Constellation خاموشی سے فرض کر لیتا ہے کہ نوٹ کی معرفتی حالت آپ کی اپنی ہے — جو سنجیدہ علمی کام کے لیے اکثر غلط ہوتا ہے۔

### `domain` — *یہ کس موضوع کے بارے میں ہے؟*

تخصصی ٹیگز کی فہرست۔ آپ کے آزاد `tags` فیلڈ (folksonomy / موڈ / پروجیکٹ) سے الگ، `domain` بازیافت اور فلٹر کرنے کے لیے ساختیاتی ضابطہ/موضوع فیلڈ ہے۔ مثالیں:

```yaml
domain: [fiqh, ʿibādāt]
domain: [photography, optics]
domain: [overland-travel, mechanics]
```

`content_type: "proposition"` اور `source: "inference"` کے طور پر درجہ بند نوٹ ایک منطقی نظریہ (domain: `[logic, mathematics]`) ہو سکتا ہے یا فقہی رائے (domain: `[fiqh, ʿibādāt]`) — وہی معرفتی شکل، بہت مختلف بازیافت سیاق۔ `domain` آپ کو یہ کہنے دیتا ہے کہ کون سا۔

### `function` — *یہ نوٹ کس لیے ہے؟*

ایک واحد سٹرنگ جو نوٹ کے مطلوبہ استعمال کی شناخت کرتی ہے۔ تسلیم شدہ اقدار:

- `reference` — ضرورت پڑنے پر پڑھیں (تعریف، حوالہ، حقیقت جسے آپ بعد میں دیکھیں گے)
- `seed` — تخمیر میں (ابتدائی مرحلے کا خیال جسے آپ ابھی تیار کر رہے ہیں)
- `actionable` — اس کے ساتھ کچھ کریں (ایک کام، فالو اپ، فیصلہ)
- `shipped` — مکمل پروڈکٹ (شائع شدہ مضمون، تسلیم شدہ تجزیہ، بند لوپ)

CECE کے content-type محور سے الگ (جو کہتا ہے کہ یہ *کس قسم* کا علم ہے) — `function` کہتا ہے کہ آپ نوٹ کے ساتھ *کیا* کریں گے۔

### `provenance_civilization` — *کس روایت کی اصطلاحات کام میں ہیں؟*

ایک اختیاری سٹرنگ جو نوٹ کی اصطلاحات کے تہذیبی نقشِ پا کی شناخت کرتی ہے۔ روایت سے مخصوص corpora کے مقابلے میں بازیافت کے لیے مفید۔ مثالیں:

- `provenance_civilization: "sunni-usuli"` — سنّی *uṣūl al-fiqh* روایت (al-Bukhārī، al-Ghazālī، al-Āmidī)
- `provenance_civilization: "analytic-western"` — Frege کے بعد کا تجزیاتی فلسفہ
- `provenance_civilization: "nyaya"` — pramāṇa معرفتیات کا ہندوستانی Nyāya مکتب
- `provenance_civilization: "buddhist-pramana"` — بدھ معرفتیاتی روایت (Dignāga، Dharmakīrti)

اکثر نوٹس کو اس کی ضرورت نہیں ہوتی۔ جب آپ کے پاس، مثلاً، ایسا نوٹ ہو جو سنّی *uṣūl* اور تجزیاتی اینگلو-امریکی معرفتیات دونوں پر استوار ہو، تو بنیادی نقشِ پا ریکارڈ کرنا مستقبل کے آپ کو درست تقابلی مواد بازیافت کرنے میں مدد دیتا ہے۔

### `updated_at` — *آپ کا موقف آخری بار کب بدلا؟*

نوٹ کے معرفتی مواد کی سب سے حالیہ ارادتاً نظرِ ثانی کی ISO تاریخ۔ فائل سسٹم کے `modified` ٹائم سٹیمپ سے الگ (جو ہر سیو کو پکڑتا ہے، حتیٰ کہ ٹائپو کی اصلاح بھی)؛ `updated_at` وہ ٹائم سٹیمپ ہے جو *آپ* تب سیٹ کرتے ہیں جب آپ نے واقعی موقف پر دوبارہ سوچا ہو۔

```yaml
updated_at: 2026-05-09
```

§6.3 کا باقی زمانی محور (نوٹ حالت تاریخ) جب لینڈ ہو تو مفید — تب تک، یہ ایک واحد-سنیپ شاٹ فیلڈ ہے جو "آخری بار جب میں نے اپنی رائے پر نظرِ ثانی کی" ریکارڈ کرتا ہے۔

### `ikhtilāf` — *ساختیاتی علمی اختلاف*

نئے فیلڈز میں سب سے پیچیدہ۔ *ikhtilāf* کو ریکارڈ کرتا ہے — کسی مسئلے پر علماء یا مذاہب کے درمیان ساختیاتی اختلاف — `{school, position}` جوڑوں کی فہرست کے طور پر۔ Constellation اسے ایڈٹ کرنے کے لیے Properties پینل کا ایک حسبِ ضرورت widget فراہم کرتا ہے؛ آپ YAML کو براہِ راست بھی ایڈٹ کر سکتے ہیں۔

مثال:

```yaml
ikhtilāf:
  - school: Ḥanafī
    position: permissible
  - school: Mālikī
    position: discouraged
  - school: Shāfiʿī
    position: permissible with conditions
  - school: Ḥanbalī
    position: forbidden
```

`ikhtilāf` والا نوٹ کسی ایک معرفتی حالت میں نہیں ہوتا — یہ متعدد actors کے درمیان *ساختیاتی اختلاف* ریکارڈ کرتا ہے۔ اس فیلڈ کے بغیر، Constellation ایسے نوٹ کا اس طرح برتاؤ کرے گا گویا یہ خود ان مواقف میں سے ایک رکھتا ہو، جو غلط ہے۔

Properties پینل ہر قطار کو دو ان پٹس (school + position) اور ایک remove بٹن کے ساتھ ایک ایڈیٹر کارڈ کے طور پر، اور نیچے ایک "Add school" بٹن کے ساتھ render کرتا ہے۔

### `warrant` اور `warrant_notes` — *parsed مگر غیر فعال (ابھی)*

دو فیلڈز جو parsed اور ڈسک پر اسٹور کیے جاتے ہیں مگر **ابھی کسی UI میں ظاہر نہیں ہوتے**:

- `warrant: "mutawātir"` — نوٹ کے دعوے کے warrant کا درجہ لیبل۔ سنّی *uṣūl* درجہ بندی *mutawātir / mashhūr / āḥād* اور حدیث میں خاص طور پر *ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ* استعمال کرتی ہے۔ دیگر روایات کی اپنی درجہ بندی کی اصطلاحات ہیں۔
- `warrant_notes: "transmitted by 30+ companions in al-Bukhārī"` — warrant درجے کی حمایت کرنے والا آزاد متن۔

یہ استعمال کے لیے تیار ہیں جب **Constellation Warrant Research workstream** اپنا classifier جاری کرے (کئی ماہ کا تحقیقی پروجیکٹ؛ gap analysis §6.2 دیکھیں)۔ تب تک آپ انہیں ہاتھ سے بھر سکتے ہیں اور ڈیٹا برقرار رہتا ہے؛ کوئی چیز اسے ظاہر نہیں کرتی۔ مستقبل کی warrant-آگاہ queries اور badges ان اقدار کو براہِ راست پڑھیں گے۔

---

## یہ فیلڈز کہاں ظاہر ہوتے ہیں

جب آپ کسی نوٹ کے frontmatter میں نئے فیلڈز میں سے کوئی بھرتے ہیں، وہ **Properties پینل** (دائیں سائیڈ بار) میں اسی طرح ظاہر ہوتے ہیں جیسے ہر دوسرا YAML فیلڈ — فی کلید ایک قطار، قسم کے مطابق ایڈیٹر کے ساتھ:

- `held_by`، `function`، `provenance_civilization`، `warrant`، `warrant_notes` ← متن ان پٹ
- `domain` ← ٹیگ فہرست (ٹائپ کر کے + Enter سے شامل کریں، ہر ٹیگ پر × سے ہٹائیں)
- `updated_at` ← تاریخ منتخب کنندہ
- `ikhtilāf` ← حسبِ ضرورت widget جس میں `school` / `position` قطاریں + شامل کریں/ہٹائیں بٹن

---

## `supersedes` کے بارے میں کیا؟

`supersedes` تکنیکی طور پر *نوٹس کے درمیان رشتہ* ہے، ایک واحد نوٹ کی خصوصیت نہیں۔ Constellation اسے **typed link** کے طور پر سنبھالتا ہے، YAML scalar کے طور پر نہیں:

```markdown
یہ نوٹ میرے پہلے تجزیے کی جگہ لیتا ہے: [[old-note-id|supersedes]]
```

wikilink پر `|supersedes` لاحقہ Constellation کو بتاتا ہے کہ یہ `supersedes` قسم کا typed-link ہے — اسے ایک منفرد pill رنگ ملتا ہے (slate نیلا-سرمئی)، Backlinks + Outgoing Links پینلوں میں دیگر typed-links کے ساتھ ظاہر ہوتا ہے، اور Living Link Architecture (وزن، lifecycle، traversal counts) میں شرکت کرتا ہے۔

یہ نوٹ-سے-نوٹ تعلقات کو ایک جگہ رکھتا ہے — typed-link سسٹم — بجائے انہیں typed-links اور frontmatter scalars کے درمیان تقسیم کرنے کے۔ یہی `contradicts:` پر لاگو ہوتا ہے (جو pre-MIG-022 اصطلاحات میں پہلے سے ہی typed-link ہے)۔

---

## یہ کیا **نہیں** ہے

یہ فیلڈز آج CECE درجہ بندی کے ذریعے **استعمال نہیں ہوتے**۔ CECE صرف Source × Content Type پر درجہ بندی کرتا ہے؛ نئے میٹا ڈیٹا فیلڈز انسانی-محرک بازیافت، مستقبل کے warrant-آگاہ classifiers، اور زمانی محور (جب جاری ہو) کے لیے ریکارڈ کیے جاتے ہیں۔

خاص طور پر:
- `function: "actionable"` خود کار طور پر Tasks پینل میں کوئی کام *نہیں* بناتا
- `held_by: "al-Shāfiʿī"` CECE کے نوٹ درجہ بندی کے طریقے کو *نہیں* بدلتا
- `domain: [fiqh]` آپ کے سرچ نتائج کو *نہیں* فلٹر کرتا جب تک آپ سرچ query کو اسے شامل کرنے کے لیے نہ لکھیں

یہ فیلڈز **schema** ہیں — تسلیم شدہ اصطلاحات ان فیلڈز کے لیے جنہیں آپ شامل کر سکتے ہیں۔ مستقبل کے MIGs ایسے features جاری کریں گے جو انہیں استعمال کریں گے (warrant classifier، زمانی queries، domain-آگاہ filtering، وغیرہ)۔

---

## ایک عملی مثال

ایک نوٹ جو روزے کے دن کی صحت کے لیے فجر ٹوٹنے پر niyyah کی ضرورت پر سنّی مذاہب کے مواقف ریکارڈ کرتا ہے:

```yaml
---
title: Niyyah for Ramadan fasting
held_by: user
domain: [fiqh, ʿibādāt, sawm]
function: reference
provenance_civilization: sunni-usuli
updated_at: 2026-05-09
warrant: mashhūr
ikhtilāf:
  - school: Ḥanafī
    position: night-before niyyah valid; same-day niyyah valid before zawāl
  - school: Mālikī
    position: night-before niyyah required; one general niyyah for the month suffices
  - school: Shāfiʿī
    position: night-before niyyah required for each obligatory fast
  - school: Ḥanbalī
    position: night-before niyyah required for each obligatory fast
---

کلاسیکی Mālikī موقف (مہینے کے لیے ایک niyyah) کو
[[Ibn-Rushd-bidayah|derives-from]] نے bidāyat al-mujtahid کی niyyah پر
عبارت میں بیان کیا ہے۔ میری موجودہ رائے: [[ramadan-niyyah-personal|supersedes]]
میرے پہلے کے نوٹ کی جس نے Mālikī موقف کو Shāfiʿī کے ساتھ مخلوط کیا تھا۔
```

سات نئے فیلڈز میں سے چھ بھرے گئے ہیں؛ `warrant_notes` چھوڑ دیا گیا (ابھی تک کوئی سلسلہ تفصیل ریکارڈ کرنے کے لیے نہیں ہے)؛ `supersedes` اور `derives-from` متن میں typed-links کے طور پر، YAML scalars کے طور پر نہیں۔

---

*MIG-022 §A — schema توسیعات اس Constellation build میں جاری ہوتی ہیں۔ Warrant Research workstream (الگ Concept Paper، کئی ماہ) warrant classifier جاری کرتا ہے جو `warrant` فیلڈ کو استعمال کرتا ہے۔ زمانی محور (MIG-023، الگ Architect cycle) `updated_at` کے علاوہ وسیع تر نوٹ حالت تاریخ کو استعمال کرتا ہے۔*
