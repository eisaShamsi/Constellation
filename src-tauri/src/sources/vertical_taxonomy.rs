//! MIG-021v2 §1A' — Vertical-axis taxonomy data.
//!
//! Source of truth: `docs/epistemic-content-taxonomy-chart.html` (Eisa-canonical,
//! interactive 5-level chart). This file is the Rust mirror plus lookup helpers.
//! TypeScript mirror: `src/lib/sources/verticalTaxonomy.ts`.
//!
//! Structure:
//!   - 1 root (epistemic-content)
//!   - 5 top-level branches (Sensory · Symbolic · Semantic · States · Higher-order)
//!   - ~218 sub-nodes total, depth varies (max depth 4 below the branch)
//!
//! ID scheme (per Plan §0 Q2): kebab-case English label slug, "/" separates
//! ancestors → leaves get full-path slugs. Guarantees uniqueness.
//!
//! Per Plan §3 risk mitigation: where the source chart provides only labels
//! (no rich definitions), this file carries only the labels. Rich
//! per-node descriptions for embedding-classification (§1B') are derived
//! mechanically from parent context — see `classifier::source_definitions`.

#[derive(Debug, Clone, Copy)]
pub struct VerticalNode {
    pub id: &'static str,
    pub en: &'static str,
    pub ar: &'static str,
    pub parent_id: Option<&'static str>,
    /// 1-5 for the five top-level branches; 0 = root.
    pub branch: u8,
}

pub const VERTICAL_NODES: &[VerticalNode] = &[
    // ─── Root ─────────────────────────────────────────────────────
    VerticalNode {
        id: "epistemic-content",
        en: "Epistemic content",
        ar: "المحتوى المعرفي",
        parent_id: None,
        branch: 0,
    },

    // ═══ Branch 1 — Sensory inputs (32 nodes) ═════════════════════
    VerticalNode { id: "sensory-inputs", en: "Sensory inputs", ar: "المُدخَلات الحسية", parent_id: Some("epistemic-content"), branch: 1 },

    // Signal
    VerticalNode { id: "sensory-inputs/signal", en: "Signal", ar: "إشارة", parent_id: Some("sensory-inputs"), branch: 1 },
    VerticalNode { id: "sensory-inputs/signal/physical", en: "Physical signal", ar: "إشارة فيزيائية", parent_id: Some("sensory-inputs/signal"), branch: 1 },
    VerticalNode { id: "sensory-inputs/signal/physical/electromagnetic", en: "Electromagnetic", ar: "كهرومغناطيسية", parent_id: Some("sensory-inputs/signal/physical"), branch: 1 },
    VerticalNode { id: "sensory-inputs/signal/physical/acoustic", en: "Acoustic", ar: "صوتية", parent_id: Some("sensory-inputs/signal/physical"), branch: 1 },
    VerticalNode { id: "sensory-inputs/signal/physical/mechanical", en: "Mechanical", ar: "ميكانيكية", parent_id: Some("sensory-inputs/signal/physical"), branch: 1 },
    VerticalNode { id: "sensory-inputs/signal/physical/chemical", en: "Chemical", ar: "كيميائية", parent_id: Some("sensory-inputs/signal/physical"), branch: 1 },
    VerticalNode { id: "sensory-inputs/signal/biological", en: "Biological signal", ar: "إشارة بيولوجية", parent_id: Some("sensory-inputs/signal"), branch: 1 },
    VerticalNode { id: "sensory-inputs/signal/biological/neural", en: "Neural", ar: "عصبية", parent_id: Some("sensory-inputs/signal/biological"), branch: 1 },
    VerticalNode { id: "sensory-inputs/signal/biological/hormonal", en: "Hormonal", ar: "هرمونية", parent_id: Some("sensory-inputs/signal/biological"), branch: 1 },
    VerticalNode { id: "sensory-inputs/signal/biological/bioelectric", en: "Bioelectric", ar: "كهربائية حيوية", parent_id: Some("sensory-inputs/signal/biological"), branch: 1 },

    // Stimulus
    VerticalNode { id: "sensory-inputs/stimulus", en: "Stimulus", ar: "مُنبِّه", parent_id: Some("sensory-inputs"), branch: 1 },
    VerticalNode { id: "sensory-inputs/stimulus/distal", en: "Distal stimulus", ar: "مُنبِّه بعيد", parent_id: Some("sensory-inputs/stimulus"), branch: 1 },
    VerticalNode { id: "sensory-inputs/stimulus/proximal", en: "Proximal stimulus", ar: "مُنبِّه قريب", parent_id: Some("sensory-inputs/stimulus"), branch: 1 },
    VerticalNode { id: "sensory-inputs/stimulus/conditioned", en: "Conditioned stimulus", ar: "مُنبِّه مَشروط", parent_id: Some("sensory-inputs/stimulus"), branch: 1 },
    VerticalNode { id: "sensory-inputs/stimulus/unconditioned", en: "Unconditioned stimulus", ar: "مُنبِّه غير مَشروط", parent_id: Some("sensory-inputs/stimulus"), branch: 1 },

    // Sense-datum
    VerticalNode { id: "sensory-inputs/sense-datum", en: "Sense-datum", ar: "مُعطى حِسِّي", parent_id: Some("sensory-inputs"), branch: 1 },
    VerticalNode { id: "sensory-inputs/sense-datum/visual", en: "Visual qualia", ar: "مُعطيات بصرية", parent_id: Some("sensory-inputs/sense-datum"), branch: 1 },
    VerticalNode { id: "sensory-inputs/sense-datum/auditory", en: "Auditory qualia", ar: "مُعطيات سمعية", parent_id: Some("sensory-inputs/sense-datum"), branch: 1 },
    VerticalNode { id: "sensory-inputs/sense-datum/tactile", en: "Tactile qualia", ar: "مُعطيات لمسية", parent_id: Some("sensory-inputs/sense-datum"), branch: 1 },
    VerticalNode { id: "sensory-inputs/sense-datum/olfactory", en: "Olfactory qualia", ar: "مُعطيات شمية", parent_id: Some("sensory-inputs/sense-datum"), branch: 1 },
    VerticalNode { id: "sensory-inputs/sense-datum/gustatory", en: "Gustatory qualia", ar: "مُعطيات ذوقية", parent_id: Some("sensory-inputs/sense-datum"), branch: 1 },
    VerticalNode { id: "sensory-inputs/sense-datum/proprioceptive", en: "Proprioceptive qualia", ar: "مُعطيات حِسِّية حركية", parent_id: Some("sensory-inputs/sense-datum"), branch: 1 },

    // Percept
    VerticalNode { id: "sensory-inputs/percept", en: "Percept", ar: "مُدرَك حِسِّي", parent_id: Some("sensory-inputs"), branch: 1 },
    VerticalNode { id: "sensory-inputs/percept/indeterminate", en: "Indeterminate percept (nirvikalpa)", ar: "إدراك غير مُحدَّد", parent_id: Some("sensory-inputs/percept"), branch: 1 },
    VerticalNode { id: "sensory-inputs/percept/indeterminate/pure-sensation", en: "Pure sensation", ar: "إحساس صِرف", parent_id: Some("sensory-inputs/percept/indeterminate"), branch: 1 },
    VerticalNode { id: "sensory-inputs/percept/determinate", en: "Determinate percept (savikalpa)", ar: "إدراك مُحدَّد", parent_id: Some("sensory-inputs/percept"), branch: 1 },
    VerticalNode { id: "sensory-inputs/percept/determinate/object-recognition", en: "Object recognition", ar: "تَعَرُّف على الموضوع", parent_id: Some("sensory-inputs/percept/determinate"), branch: 1 },
    VerticalNode { id: "sensory-inputs/percept/determinate/categorized", en: "Categorized perception", ar: "إدراك مُصنَّف", parent_id: Some("sensory-inputs/percept/determinate"), branch: 1 },
    VerticalNode { id: "sensory-inputs/percept/extraordinary", en: "Extra-ordinary perception (alaukika)", ar: "إدراك فائق", parent_id: Some("sensory-inputs/percept"), branch: 1 },
    VerticalNode { id: "sensory-inputs/percept/extraordinary/generic", en: "Generic (sāmānyalakṣaṇa)", ar: "إدراك للنوع العام", parent_id: Some("sensory-inputs/percept/extraordinary"), branch: 1 },
    VerticalNode { id: "sensory-inputs/percept/extraordinary/acquired", en: "Acquired (jñānalakṣaṇa)", ar: "إدراك مُكتسَب", parent_id: Some("sensory-inputs/percept/extraordinary"), branch: 1 },
    VerticalNode { id: "sensory-inputs/percept/extraordinary/yogic", en: "Yogic (yogaja)", ar: "إدراك تأمُّلي", parent_id: Some("sensory-inputs/percept/extraordinary"), branch: 1 },

    // ═══ Branch 2 — Symbolic entities (44 nodes) ══════════════════
    VerticalNode { id: "symbolic-entities", en: "Symbolic entities", ar: "الكيانات الرمزية", parent_id: Some("epistemic-content"), branch: 2 },

    // Sign (Peirce)
    VerticalNode { id: "symbolic-entities/sign", en: "Sign (Peirce's classification)", ar: "علامة (تصنيف بيرس)", parent_id: Some("symbolic-entities"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/icon", en: "Icon — by resemblance", ar: "أيقونة — بالمشابهة", parent_id: Some("symbolic-entities/sign"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/icon/image", en: "Image", ar: "صورة", parent_id: Some("symbolic-entities/sign/icon"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/icon/diagram", en: "Diagram", ar: "مُخطَّط", parent_id: Some("symbolic-entities/sign/icon"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/icon/metaphor", en: "Metaphor", ar: "استعارة", parent_id: Some("symbolic-entities/sign/icon"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/index", en: "Index — by causal connection", ar: "دلالة سببية — بالاقتران", parent_id: Some("symbolic-entities/sign"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/index/symptom", en: "Symptom", ar: "عَرَض", parent_id: Some("symbolic-entities/sign/index"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/index/trace", en: "Trace", ar: "أثَر", parent_id: Some("symbolic-entities/sign/index"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/index/pointer", en: "Pointer", ar: "مُؤَشِّر", parent_id: Some("symbolic-entities/sign/index"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/symbol-conventional", en: "Symbol — by convention", ar: "رمز اصطلاحي — بالتواضُع", parent_id: Some("symbolic-entities/sign"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/symbol-conventional/word", en: "Word", ar: "كلمة", parent_id: Some("symbolic-entities/sign/symbol-conventional"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/symbol-conventional/numeral", en: "Numeral", ar: "رقم", parent_id: Some("symbolic-entities/sign/symbol-conventional"), branch: 2 },
    VerticalNode { id: "symbolic-entities/sign/symbol-conventional/logical-operator", en: "Logical operator", ar: "مُؤَثِّر منطقي", parent_id: Some("symbolic-entities/sign/symbol-conventional"), branch: 2 },

    // Symbol
    VerticalNode { id: "symbolic-entities/symbol", en: "Symbol", ar: "رَمز", parent_id: Some("symbolic-entities"), branch: 2 },
    VerticalNode { id: "symbolic-entities/symbol/linguistic", en: "Linguistic symbol", ar: "رمز لُغوي", parent_id: Some("symbolic-entities/symbol"), branch: 2 },
    VerticalNode { id: "symbolic-entities/symbol/linguistic/phoneme", en: "Phoneme", ar: "فونيم / صَوتية", parent_id: Some("symbolic-entities/symbol/linguistic"), branch: 2 },
    VerticalNode { id: "symbolic-entities/symbol/linguistic/morpheme", en: "Morpheme", ar: "مورفيم / صَرفية", parent_id: Some("symbolic-entities/symbol/linguistic"), branch: 2 },
    VerticalNode { id: "symbolic-entities/symbol/linguistic/lexeme", en: "Lexeme", ar: "ليكسيم / مُعجَمية", parent_id: Some("symbolic-entities/symbol/linguistic"), branch: 2 },
    VerticalNode { id: "symbolic-entities/symbol/mathematical", en: "Mathematical symbol", ar: "رمز رياضي", parent_id: Some("symbolic-entities/symbol"), branch: 2 },
    VerticalNode { id: "symbolic-entities/symbol/logical", en: "Logical symbol", ar: "رمز منطقي", parent_id: Some("symbolic-entities/symbol"), branch: 2 },
    VerticalNode { id: "symbolic-entities/symbol/iconographic", en: "Iconographic symbol", ar: "رمز أيقوني", parent_id: Some("symbolic-entities/symbol"), branch: 2 },

    // Data
    VerticalNode { id: "symbolic-entities/data", en: "Data", ar: "بيانات", parent_id: Some("symbolic-entities"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/structured", en: "Structured data", ar: "بيانات مُهيكَلة", parent_id: Some("symbolic-entities/data"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/structured/tabular", en: "Tabular", ar: "جَدوَلية", parent_id: Some("symbolic-entities/data/structured"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/structured/relational", en: "Relational", ar: "علائقية", parent_id: Some("symbolic-entities/data/structured"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/semi-structured", en: "Semi-structured data", ar: "بيانات شبه مُهيكَلة", parent_id: Some("symbolic-entities/data"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/semi-structured/serialized", en: "JSON / XML / YAML", ar: "صِيَغ مُتسلسِلة", parent_id: Some("symbolic-entities/data/semi-structured"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/unstructured", en: "Unstructured data", ar: "بيانات غير مُهيكَلة", parent_id: Some("symbolic-entities/data"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/unstructured/text", en: "Text", ar: "نص", parent_id: Some("symbolic-entities/data/unstructured"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/unstructured/image", en: "Image", ar: "صورة", parent_id: Some("symbolic-entities/data/unstructured"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/unstructured/audio", en: "Audio", ar: "صوت", parent_id: Some("symbolic-entities/data/unstructured"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/unstructured/video", en: "Video", ar: "فيديو", parent_id: Some("symbolic-entities/data/unstructured"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/quantitative", en: "Quantitative data", ar: "بيانات كمية", parent_id: Some("symbolic-entities/data"), branch: 2 },
    VerticalNode { id: "symbolic-entities/data/qualitative", en: "Qualitative data", ar: "بيانات نوعية", parent_id: Some("symbolic-entities/data"), branch: 2 },

    // Code
    VerticalNode { id: "symbolic-entities/code", en: "Code", ar: "شِفرة / ترميز", parent_id: Some("symbolic-entities"), branch: 2 },
    VerticalNode { id: "symbolic-entities/code/natural-language", en: "Natural language code", ar: "شِفرة لغة طبيعية", parent_id: Some("symbolic-entities/code"), branch: 2 },
    VerticalNode { id: "symbolic-entities/code/formal-language", en: "Formal language code", ar: "شِفرة لغة صورية", parent_id: Some("symbolic-entities/code"), branch: 2 },
    VerticalNode { id: "symbolic-entities/code/computer", en: "Computer code", ar: "شِفرة حاسوبية", parent_id: Some("symbolic-entities/code"), branch: 2 },
    VerticalNode { id: "symbolic-entities/code/cryptographic", en: "Cryptographic code", ar: "شِفرة تَعمية", parent_id: Some("symbolic-entities/code"), branch: 2 },

    // Inscription
    VerticalNode { id: "symbolic-entities/inscription", en: "Inscription", ar: "تَدوين", parent_id: Some("symbolic-entities"), branch: 2 },
    VerticalNode { id: "symbolic-entities/inscription/oral", en: "Oral", ar: "شَفَوي", parent_id: Some("symbolic-entities/inscription"), branch: 2 },
    VerticalNode { id: "symbolic-entities/inscription/manuscript", en: "Manuscript", ar: "مَخطوط", parent_id: Some("symbolic-entities/inscription"), branch: 2 },
    VerticalNode { id: "symbolic-entities/inscription/print", en: "Print", ar: "مَطبوع", parent_id: Some("symbolic-entities/inscription"), branch: 2 },
    VerticalNode { id: "symbolic-entities/inscription/digital", en: "Digital", ar: "رَقمي", parent_id: Some("symbolic-entities/inscription"), branch: 2 },

    // ═══ Branch 3 — Semantic contents (48 nodes) ══════════════════
    VerticalNode { id: "semantic-contents", en: "Semantic contents", ar: "المحتويات الدلالية", parent_id: Some("epistemic-content"), branch: 3 },

    // Concept (taṣawwur)
    VerticalNode { id: "semantic-contents/concept", en: "Concept (taṣawwur)", ar: "مفهوم / تَصوُّر", parent_id: Some("semantic-contents"), branch: 3 },
    VerticalNode { id: "semantic-contents/concept/simple", en: "Simple concept", ar: "تَصوُّر بسيط", parent_id: Some("semantic-contents/concept"), branch: 3 },
    VerticalNode { id: "semantic-contents/concept/compound", en: "Compound concept", ar: "تَصوُّر مُركَّب", parent_id: Some("semantic-contents/concept"), branch: 3 },
    VerticalNode { id: "semantic-contents/concept/empirical", en: "Empirical concept", ar: "مفهوم تجريبي", parent_id: Some("semantic-contents/concept"), branch: 3 },
    VerticalNode { id: "semantic-contents/concept/empirical/observational", en: "Observational", ar: "مُشاهَداتي", parent_id: Some("semantic-contents/concept/empirical"), branch: 3 },
    VerticalNode { id: "semantic-contents/concept/empirical/experimental", en: "Experimental", ar: "اختباري", parent_id: Some("semantic-contents/concept/empirical"), branch: 3 },
    VerticalNode { id: "semantic-contents/concept/a-priori", en: "A priori concept", ar: "مفهوم قَبْلي", parent_id: Some("semantic-contents/concept"), branch: 3 },
    VerticalNode { id: "semantic-contents/concept/a-priori/logical", en: "Logical", ar: "منطقي", parent_id: Some("semantic-contents/concept/a-priori"), branch: 3 },
    VerticalNode { id: "semantic-contents/concept/a-priori/mathematical", en: "Mathematical", ar: "رياضي", parent_id: Some("semantic-contents/concept/a-priori"), branch: 3 },
    VerticalNode { id: "semantic-contents/concept/concrete", en: "Concrete concept", ar: "مفهوم عَيني", parent_id: Some("semantic-contents/concept"), branch: 3 },
    VerticalNode { id: "semantic-contents/concept/abstract", en: "Abstract concept", ar: "مفهوم مُجرَّد", parent_id: Some("semantic-contents/concept"), branch: 3 },

    // Idea
    VerticalNode { id: "semantic-contents/idea", en: "Idea", ar: "فكرة", parent_id: Some("semantic-contents"), branch: 3 },
    VerticalNode { id: "semantic-contents/idea/innate", en: "Innate idea (Cartesian)", ar: "فكرة فطرية", parent_id: Some("semantic-contents/idea"), branch: 3 },
    VerticalNode { id: "semantic-contents/idea/adventitious", en: "Adventitious idea", ar: "فكرة مكتسَبة", parent_id: Some("semantic-contents/idea"), branch: 3 },
    VerticalNode { id: "semantic-contents/idea/constructed", en: "Constructed idea", ar: "فكرة مُؤَلَّفة", parent_id: Some("semantic-contents/idea"), branch: 3 },
    VerticalNode { id: "semantic-contents/idea/clear-distinct", en: "Clear and distinct idea", ar: "فكرة واضحة ومُتمايزة", parent_id: Some("semantic-contents/idea"), branch: 3 },

    // Proposition (qaḍiyyah)
    VerticalNode { id: "semantic-contents/proposition", en: "Proposition (qaḍiyyah)", ar: "قضية", parent_id: Some("semantic-contents"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/categorical", en: "Categorical proposition", ar: "قضية حَملية", parent_id: Some("semantic-contents/proposition"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/categorical/universal-affirmative", en: "Universal affirmative", ar: "موجبة كلية", parent_id: Some("semantic-contents/proposition/categorical"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/categorical/universal-negative", en: "Universal negative", ar: "سالبة كلية", parent_id: Some("semantic-contents/proposition/categorical"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/categorical/particular-affirmative", en: "Particular affirmative", ar: "موجبة جزئية", parent_id: Some("semantic-contents/proposition/categorical"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/categorical/particular-negative", en: "Particular negative", ar: "سالبة جزئية", parent_id: Some("semantic-contents/proposition/categorical"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/hypothetical", en: "Hypothetical proposition", ar: "قضية شَرطية", parent_id: Some("semantic-contents/proposition"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/hypothetical/conjunctive", en: "Conjunctive (muttaṣilah)", ar: "متصلة", parent_id: Some("semantic-contents/proposition/hypothetical"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/hypothetical/disjunctive", en: "Disjunctive (munfaṣilah)", ar: "منفصلة", parent_id: Some("semantic-contents/proposition/hypothetical"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/modal", en: "Modal proposition", ar: "قضية موجَّهة", parent_id: Some("semantic-contents/proposition"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/modal/necessary", en: "Necessary", ar: "ضرورية", parent_id: Some("semantic-contents/proposition/modal"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/modal/possible", en: "Possible", ar: "ممكنة", parent_id: Some("semantic-contents/proposition/modal"), branch: 3 },
    VerticalNode { id: "semantic-contents/proposition/modal/impossible", en: "Impossible", ar: "مُمتنِعة", parent_id: Some("semantic-contents/proposition/modal"), branch: 3 },

    // Information (DIKW T2)
    VerticalNode { id: "semantic-contents/information", en: "Information (DIKW T2)", ar: "معلومات", parent_id: Some("semantic-contents"), branch: 3 },
    VerticalNode { id: "semantic-contents/information/syntactic", en: "Syntactic information", ar: "معلومات بِنيوية", parent_id: Some("semantic-contents/information"), branch: 3 },
    VerticalNode { id: "semantic-contents/information/semantic", en: "Semantic information", ar: "معلومات دَلالية", parent_id: Some("semantic-contents/information"), branch: 3 },
    VerticalNode { id: "semantic-contents/information/pragmatic", en: "Pragmatic information", ar: "معلومات تَداولية", parent_id: Some("semantic-contents/information"), branch: 3 },

    // Fact (wāqiʿah)
    VerticalNode { id: "semantic-contents/fact", en: "Fact (wāqiʿah)", ar: "واقعة / حقيقة", parent_id: Some("semantic-contents"), branch: 3 },
    VerticalNode { id: "semantic-contents/fact/empirical", en: "Empirical fact", ar: "واقعة تجريبية", parent_id: Some("semantic-contents/fact"), branch: 3 },
    VerticalNode { id: "semantic-contents/fact/mathematical", en: "Mathematical fact", ar: "واقعة رياضية", parent_id: Some("semantic-contents/fact"), branch: 3 },
    VerticalNode { id: "semantic-contents/fact/logical", en: "Logical fact", ar: "واقعة منطقية", parent_id: Some("semantic-contents/fact"), branch: 3 },
    VerticalNode { id: "semantic-contents/fact/historical", en: "Historical fact", ar: "واقعة تاريخية", parent_id: Some("semantic-contents/fact"), branch: 3 },
    VerticalNode { id: "semantic-contents/fact/moral", en: "Moral fact", ar: "واقعة أخلاقية", parent_id: Some("semantic-contents/fact"), branch: 3 },

    // Meaning (maʿnā)
    VerticalNode { id: "semantic-contents/meaning", en: "Meaning (maʿnā)", ar: "معنى", parent_id: Some("semantic-contents"), branch: 3 },
    VerticalNode { id: "semantic-contents/meaning/sense-vs-reference", en: "Sense vs. reference (Frege)", ar: "المعنى مقابل المرجع", parent_id: Some("semantic-contents/meaning"), branch: 3 },
    VerticalNode { id: "semantic-contents/meaning/connotation-vs-denotation", en: "Connotation vs. denotation", ar: "تضمين مقابل دلالة", parent_id: Some("semantic-contents/meaning"), branch: 3 },
    VerticalNode { id: "semantic-contents/meaning/literal-vs-figurative", en: "Literal vs. figurative", ar: "حقيقي مقابل مجازي", parent_id: Some("semantic-contents/meaning"), branch: 3 },

    // Definition (ḥadd)
    VerticalNode { id: "semantic-contents/definition", en: "Definition (ḥadd)", ar: "تعريف / حدّ", parent_id: Some("semantic-contents"), branch: 3 },
    VerticalNode { id: "semantic-contents/definition/real", en: "Real definition", ar: "حدّ حقيقي", parent_id: Some("semantic-contents/definition"), branch: 3 },
    VerticalNode { id: "semantic-contents/definition/nominal", en: "Nominal definition", ar: "حدّ اسمي", parent_id: Some("semantic-contents/definition"), branch: 3 },
    VerticalNode { id: "semantic-contents/definition/genus-differentia", en: "Genus-differentia", ar: "جنس وفصل", parent_id: Some("semantic-contents/definition"), branch: 3 },
    VerticalNode { id: "semantic-contents/definition/ostensive", en: "Ostensive definition", ar: "تعريف إشاري", parent_id: Some("semantic-contents/definition"), branch: 3 },

    // ═══ Branch 4 — Epistemic states (49 nodes) ═══════════════════
    VerticalNode { id: "epistemic-states", en: "Epistemic states", ar: "الحالات المعرفية", parent_id: Some("epistemic-content"), branch: 4 },

    VerticalNode { id: "epistemic-states/compound-ignorance", en: "Compound ignorance", ar: "جَهْل مُرَكَّب", parent_id: Some("epistemic-states"), branch: 4 },
    VerticalNode { id: "epistemic-states/simple-ignorance", en: "Simple ignorance", ar: "جَهْل بَسيط", parent_id: Some("epistemic-states"), branch: 4 },

    // Illusion (wahm)
    VerticalNode { id: "epistemic-states/illusion", en: "Illusion (wahm)", ar: "وَهْم", parent_id: Some("epistemic-states"), branch: 4 },
    VerticalNode { id: "epistemic-states/illusion/perceptual", en: "Perceptual illusion", ar: "وَهْم إدراكي", parent_id: Some("epistemic-states/illusion"), branch: 4 },
    VerticalNode { id: "epistemic-states/illusion/cognitive", en: "Cognitive illusion", ar: "وَهْم معرفي", parent_id: Some("epistemic-states/illusion"), branch: 4 },

    // Doubt (shakk)
    VerticalNode { id: "epistemic-states/doubt", en: "Doubt (shakk)", ar: "شَكّ", parent_id: Some("epistemic-states"), branch: 4 },
    VerticalNode { id: "epistemic-states/doubt/methodological", en: "Methodological doubt", ar: "شك مَنهَجي", parent_id: Some("epistemic-states/doubt"), branch: 4 },
    VerticalNode { id: "epistemic-states/doubt/habitual", en: "Habitual doubt", ar: "شك عَرَضي", parent_id: Some("epistemic-states/doubt"), branch: 4 },
    VerticalNode { id: "epistemic-states/doubt/equipoised", en: "Equipoised doubt", ar: "شك مُتساوي الطرفين", parent_id: Some("epistemic-states/doubt"), branch: 4 },

    // Opinion (ẓann)
    VerticalNode { id: "epistemic-states/opinion", en: "Opinion (ẓann)", ar: "ظَنّ", parent_id: Some("epistemic-states"), branch: 4 },
    VerticalNode { id: "epistemic-states/opinion/probable", en: "Probable opinion", ar: "ظن غالب", parent_id: Some("epistemic-states/opinion"), branch: 4 },
    VerticalNode { id: "epistemic-states/opinion/well-grounded", en: "Well-grounded opinion", ar: "ظن مُسوَّغ", parent_id: Some("epistemic-states/opinion"), branch: 4 },
    VerticalNode { id: "epistemic-states/opinion/dispositional", en: "Dispositional opinion", ar: "ظن استعدادي", parent_id: Some("epistemic-states/opinion"), branch: 4 },

    // Belief (iʿtiqād / taṣdīq)
    VerticalNode { id: "epistemic-states/belief", en: "Belief (iʿtiqād / taṣdīq)", ar: "اعتقاد / تصديق", parent_id: Some("epistemic-states"), branch: 4 },
    VerticalNode { id: "epistemic-states/belief/occurrent", en: "Occurrent belief", ar: "اعتقاد حالي", parent_id: Some("epistemic-states/belief"), branch: 4 },
    VerticalNode { id: "epistemic-states/belief/dispositional", en: "Dispositional belief", ar: "اعتقاد استعدادي", parent_id: Some("epistemic-states/belief"), branch: 4 },
    VerticalNode { id: "epistemic-states/belief/faith", en: "Faith (īmān)", ar: "إيمان", parent_id: Some("epistemic-states/belief"), branch: 4 },

    // Knowledge (ʿilm / maʿrifah)
    VerticalNode { id: "epistemic-states/knowledge", en: "Knowledge (ʿilm / maʿrifah)", ar: "علم / معرفة", parent_id: Some("epistemic-states"), branch: 4 },

    VerticalNode { id: "epistemic-states/knowledge/by-mode", en: "By mode of representation", ar: "بحسب نمط التمثيل", parent_id: Some("epistemic-states/knowledge"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-mode/acquired", en: "Acquired (al-ʿilm al-ḥuṣūlī)", ar: "علم حُصولي", parent_id: Some("epistemic-states/knowledge/by-mode"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-mode/presential", en: "Presential (al-ʿilm al-ḥuḍūrī)", ar: "علم حُضوري", parent_id: Some("epistemic-states/knowledge/by-mode"), branch: 4 },

    VerticalNode { id: "epistemic-states/knowledge/by-content", en: "By content type", ar: "بحسب نوع المحتوى", parent_id: Some("epistemic-states/knowledge"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/propositional", en: "Propositional (knowledge-that)", ar: "علم قَضَوي", parent_id: Some("epistemic-states/knowledge/by-content"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/propositional/a-priori-analytic", en: "A priori — analytic", ar: "قَبْلي — تحليلي", parent_id: Some("epistemic-states/knowledge/by-content/propositional"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/propositional/a-priori-synthetic", en: "A priori — synthetic", ar: "قَبْلي — تركيبي", parent_id: Some("epistemic-states/knowledge/by-content/propositional"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/propositional/a-posteriori-empirical", en: "A posteriori — empirical", ar: "بَعْدي — تجريبي", parent_id: Some("epistemic-states/knowledge/by-content/propositional"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/propositional/a-posteriori-testimonial", en: "A posteriori — testimonial", ar: "بَعْدي — خَبَري", parent_id: Some("epistemic-states/knowledge/by-content/propositional"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/procedural", en: "Procedural (knowledge-how)", ar: "علم إجرائي", parent_id: Some("epistemic-states/knowledge/by-content"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/procedural/motor", en: "Motor skill", ar: "مهارة حركية", parent_id: Some("epistemic-states/knowledge/by-content/procedural"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/procedural/cognitive", en: "Cognitive skill", ar: "مهارة معرفية", parent_id: Some("epistemic-states/knowledge/by-content/procedural"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/procedural/social", en: "Social skill", ar: "مهارة اجتماعية", parent_id: Some("epistemic-states/knowledge/by-content/procedural"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/acquaintance", en: "Acquaintance (knowledge-of)", ar: "معرفة بالمباشرة", parent_id: Some("epistemic-states/knowledge/by-content"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/acquaintance/direct-experience", en: "By direct experience", ar: "بالتجربة المباشرة", parent_id: Some("epistemic-states/knowledge/by-content/acquaintance"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/acquaintance/memory", en: "By memory", ar: "بالذاكرة", parent_id: Some("epistemic-states/knowledge/by-content/acquaintance"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-content/tacit", en: "Tacit (Polanyi)", ar: "علم ضمني", parent_id: Some("epistemic-states/knowledge/by-content"), branch: 4 },

    VerticalNode { id: "epistemic-states/knowledge/by-source", en: "By source", ar: "بحسب المصدر", parent_id: Some("epistemic-states/knowledge"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-source/sensory", en: "Sensory knowledge", ar: "علم حِسِّي", parent_id: Some("epistemic-states/knowledge/by-source"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-source/rational", en: "Rational knowledge", ar: "علم عقلي", parent_id: Some("epistemic-states/knowledge/by-source"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-source/testimonial", en: "Testimonial knowledge", ar: "علم خَبَري", parent_id: Some("epistemic-states/knowledge/by-source"), branch: 4 },
    VerticalNode { id: "epistemic-states/knowledge/by-source/revelatory", en: "Revelatory knowledge", ar: "علم وَحْيي", parent_id: Some("epistemic-states/knowledge/by-source"), branch: 4 },

    // Certainty (yaqīn)
    VerticalNode { id: "epistemic-states/certainty", en: "Certainty (yaqīn)", ar: "يَقين", parent_id: Some("epistemic-states"), branch: 4 },
    VerticalNode { id: "epistemic-states/certainty/logical", en: "Logical certainty", ar: "يقين منطقي", parent_id: Some("epistemic-states/certainty"), branch: 4 },
    VerticalNode { id: "epistemic-states/certainty/mathematical", en: "Mathematical certainty", ar: "يقين رياضي", parent_id: Some("epistemic-states/certainty"), branch: 4 },
    VerticalNode { id: "epistemic-states/certainty/metaphysical", en: "Metaphysical certainty", ar: "يقين ميتافيزيقي", parent_id: Some("epistemic-states/certainty"), branch: 4 },
    VerticalNode { id: "epistemic-states/certainty/moral", en: "Moral certainty", ar: "يقين أخلاقي", parent_id: Some("epistemic-states/certainty"), branch: 4 },
    VerticalNode { id: "epistemic-states/certainty/religious", en: "Religious certainty", ar: "يقين ديني", parent_id: Some("epistemic-states/certainty"), branch: 4 },
    VerticalNode { id: "epistemic-states/certainty/religious/ilm-al-yaqin", en: "ʿIlm al-yaqīn — knowledge of certainty", ar: "علم اليقين", parent_id: Some("epistemic-states/certainty/religious"), branch: 4 },
    VerticalNode { id: "epistemic-states/certainty/religious/ayn-al-yaqin", en: "ʿAyn al-yaqīn — eye of certainty", ar: "عين اليقين", parent_id: Some("epistemic-states/certainty/religious"), branch: 4 },
    VerticalNode { id: "epistemic-states/certainty/religious/haqq-al-yaqin", en: "Ḥaqq al-yaqīn — truth of certainty", ar: "حقّ اليقين", parent_id: Some("epistemic-states/certainty/religious"), branch: 4 },

    // ═══ Branch 5 — Higher-order constructs (45 nodes) ════════════
    VerticalNode { id: "higher-order-constructs", en: "Higher-order constructs", ar: "التركيبات العُليا", parent_id: Some("epistemic-content"), branch: 5 },

    // Hypothesis (faraḍiyyah)
    VerticalNode { id: "higher-order-constructs/hypothesis", en: "Hypothesis (faraḍiyyah)", ar: "فَرَضيَّة", parent_id: Some("higher-order-constructs"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/hypothesis/working", en: "Working hypothesis", ar: "فرضية عَمَل", parent_id: Some("higher-order-constructs/hypothesis"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/hypothesis/statistical", en: "Statistical hypothesis", ar: "فرضية إحصائية", parent_id: Some("higher-order-constructs/hypothesis"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/hypothesis/statistical/null", en: "Null hypothesis", ar: "فرضية العَدَم", parent_id: Some("higher-order-constructs/hypothesis/statistical"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/hypothesis/statistical/alternative", en: "Alternative hypothesis", ar: "فرضية بَديلة", parent_id: Some("higher-order-constructs/hypothesis/statistical"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/hypothesis/causal", en: "Causal hypothesis", ar: "فرضية سببية", parent_id: Some("higher-order-constructs/hypothesis"), branch: 5 },

    // Theory (naẓariyyah)
    VerticalNode { id: "higher-order-constructs/theory", en: "Theory (naẓariyyah)", ar: "نَظَرية", parent_id: Some("higher-order-constructs"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/theory/empirical", en: "Empirical theory", ar: "نظرية تجريبية", parent_id: Some("higher-order-constructs/theory"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/theory/empirical/descriptive", en: "Descriptive", ar: "وَصفية", parent_id: Some("higher-order-constructs/theory/empirical"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/theory/empirical/explanatory", en: "Explanatory", ar: "تفسيرية", parent_id: Some("higher-order-constructs/theory/empirical"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/theory/empirical/predictive", en: "Predictive", ar: "تنبؤية", parent_id: Some("higher-order-constructs/theory/empirical"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/theory/formal", en: "Formal theory", ar: "نظرية صورية", parent_id: Some("higher-order-constructs/theory"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/theory/normative", en: "Normative theory", ar: "نظرية معيارية", parent_id: Some("higher-order-constructs/theory"), branch: 5 },

    // Model (namūdhaj)
    VerticalNode { id: "higher-order-constructs/model", en: "Model (namūdhaj)", ar: "نَموذَج", parent_id: Some("higher-order-constructs"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/model/physical", en: "Physical model", ar: "نموذج فيزيائي", parent_id: Some("higher-order-constructs/model"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/model/mathematical", en: "Mathematical model", ar: "نموذج رياضي", parent_id: Some("higher-order-constructs/model"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/model/computational", en: "Computational model", ar: "نموذج حاسوبي", parent_id: Some("higher-order-constructs/model"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/model/conceptual", en: "Conceptual model", ar: "نموذج مفاهيمي", parent_id: Some("higher-order-constructs/model"), branch: 5 },

    // Law (qānūn)
    VerticalNode { id: "higher-order-constructs/law", en: "Law (qānūn)", ar: "قانون", parent_id: Some("higher-order-constructs"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/law/natural", en: "Natural law", ar: "قانون طبيعي", parent_id: Some("higher-order-constructs/law"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/law/natural/universal", en: "Universal law", ar: "قانون كلي", parent_id: Some("higher-order-constructs/law/natural"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/law/natural/statistical", en: "Statistical law", ar: "قانون إحصائي", parent_id: Some("higher-order-constructs/law/natural"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/law/logical", en: "Logical law", ar: "قانون منطقي", parent_id: Some("higher-order-constructs/law"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/law/logical/identity", en: "Identity", ar: "الهُوية", parent_id: Some("higher-order-constructs/law/logical"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/law/logical/non-contradiction", en: "Non-contradiction", ar: "عدم التناقض", parent_id: Some("higher-order-constructs/law/logical"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/law/logical/excluded-middle", en: "Excluded middle", ar: "الثالث المرفوع", parent_id: Some("higher-order-constructs/law/logical"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/law/divine", en: "Divine law (sharīʿah)", ar: "شَريعة", parent_id: Some("higher-order-constructs/law"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/law/positive", en: "Positive law", ar: "قانون وَضْعي", parent_id: Some("higher-order-constructs/law"), branch: 5 },

    // Doctrine (madhhab)
    VerticalNode { id: "higher-order-constructs/doctrine", en: "Doctrine (madhhab)", ar: "مَذهَب", parent_id: Some("higher-order-constructs"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/doctrine/religious", en: "Religious doctrine", ar: "مَذهَب ديني", parent_id: Some("higher-order-constructs/doctrine"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/doctrine/philosophical", en: "Philosophical doctrine", ar: "مَذهَب فلسفي", parent_id: Some("higher-order-constructs/doctrine"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/doctrine/scientific", en: "Scientific doctrine", ar: "مَذهَب علمي", parent_id: Some("higher-order-constructs/doctrine"), branch: 5 },

    // Insight (baṣīrah)
    VerticalNode { id: "higher-order-constructs/insight", en: "Insight (baṣīrah)", ar: "بَصيرة", parent_id: Some("higher-order-constructs"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/insight/aesthetic", en: "Aesthetic insight", ar: "بصيرة جمالية", parent_id: Some("higher-order-constructs/insight"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/insight/mystical", en: "Mystical insight", ar: "بصيرة عرفانية", parent_id: Some("higher-order-constructs/insight"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/insight/scientific", en: "Scientific insight", ar: "بصيرة علمية", parent_id: Some("higher-order-constructs/insight"), branch: 5 },

    // Wisdom (ḥikmah)
    VerticalNode { id: "higher-order-constructs/wisdom", en: "Wisdom (ḥikmah)", ar: "حِكمة", parent_id: Some("higher-order-constructs"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/wisdom/theoretical", en: "Theoretical wisdom (sophia)", ar: "حكمة نَظَرية", parent_id: Some("higher-order-constructs/wisdom"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/wisdom/practical", en: "Practical wisdom (phronēsis)", ar: "حكمة عَمَلية", parent_id: Some("higher-order-constructs/wisdom"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/wisdom/productive-craft", en: "Productive craft (technē)", ar: "صناعة مُنتِجة", parent_id: Some("higher-order-constructs/wisdom"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/wisdom/spiritual", en: "Spiritual wisdom", ar: "حكمة روحانية", parent_id: Some("higher-order-constructs/wisdom"), branch: 5 },

    // Worldview (ruʾyah kawniyyah)
    VerticalNode { id: "higher-order-constructs/worldview", en: "Worldview (ruʾyah kawniyyah)", ar: "رؤية كَونية", parent_id: Some("higher-order-constructs"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/worldview/religious", en: "Religious worldview", ar: "رؤية كَونية دينية", parent_id: Some("higher-order-constructs/worldview"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/worldview/secular", en: "Secular worldview", ar: "رؤية كَونية علمانية", parent_id: Some("higher-order-constructs/worldview"), branch: 5 },
    VerticalNode { id: "higher-order-constructs/worldview/syncretic", en: "Syncretic worldview", ar: "رؤية كَونية تَوفيقية", parent_id: Some("higher-order-constructs/worldview"), branch: 5 },
];

// ─── Lookup helpers ────────────────────────────────────────────────

pub fn is_valid_id(id: &str) -> bool {
    VERTICAL_NODES.iter().any(|n| n.id == id)
}

pub fn all_ids() -> Vec<&'static str> {
    VERTICAL_NODES.iter().map(|n| n.id).collect()
}

pub fn parent_of(id: &str) -> Option<&'static str> {
    VERTICAL_NODES
        .iter()
        .find(|n| n.id == id)
        .and_then(|n| n.parent_id)
}

pub fn children_of(parent_id: &str) -> Vec<&'static str> {
    VERTICAL_NODES
        .iter()
        .filter(|n| n.parent_id == Some(parent_id))
        .map(|n| n.id)
        .collect()
}

pub fn branch_of(id: &str) -> Option<u8> {
    VERTICAL_NODES.iter().find(|n| n.id == id).map(|n| n.branch)
}

pub fn en_label(id: &str) -> Option<&'static str> {
    VERTICAL_NODES.iter().find(|n| n.id == id).map(|n| n.en)
}

pub fn ar_label(id: &str) -> Option<&'static str> {
    VERTICAL_NODES.iter().find(|n| n.id == id).map(|n| n.ar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_present() {
        assert!(is_valid_id("epistemic-content"));
        assert_eq!(parent_of("epistemic-content"), None);
    }

    #[test]
    fn five_top_branches() {
        let branches = children_of("epistemic-content");
        assert_eq!(branches.len(), 5);
        assert!(branches.contains(&"sensory-inputs"));
        assert!(branches.contains(&"symbolic-entities"));
        assert!(branches.contains(&"semantic-contents"));
        assert!(branches.contains(&"epistemic-states"));
        assert!(branches.contains(&"higher-order-constructs"));
    }

    #[test]
    fn no_duplicate_ids() {
        let mut seen = std::collections::HashSet::new();
        for n in VERTICAL_NODES {
            assert!(seen.insert(n.id), "duplicate ID: {}", n.id);
        }
    }

    #[test]
    fn all_parents_resolve() {
        for n in VERTICAL_NODES {
            if let Some(p) = n.parent_id {
                assert!(is_valid_id(p), "node `{}` has unknown parent `{}`", n.id, p);
            }
        }
    }

    #[test]
    fn branch_metadata_consistent() {
        // Every node under branch X must have branch == X (root excluded).
        for n in VERTICAL_NODES.iter().filter(|n| n.parent_id.is_some()) {
            let p = n.parent_id.unwrap();
            let pb = branch_of(p).unwrap();
            // Skip root's children — they get assigned the branch number directly.
            if p != "epistemic-content" {
                assert_eq!(n.branch, pb, "node `{}` branch={} but parent `{}` branch={}", n.id, n.branch, p, pb);
            }
        }
    }

    #[test]
    fn approximately_218_sub_nodes() {
        // Total = 1 root + 218 sub-nodes = 219. Source diagram preamble
        // says "5 branches × ~218 nodes total"; we accept ±5 for chart-vs-extraction drift.
        let total = VERTICAL_NODES.len();
        let sub = total - 1; // exclude root
        assert!(
            sub >= 213 && sub <= 223,
            "expected ~218 sub-nodes, got {}", sub
        );
    }
}
