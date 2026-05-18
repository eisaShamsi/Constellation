---
id: masadir
name: masādir
family: sunni-islamic-usul
shape: sectoral
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# masādir

**ファミリー**: スンニ派イスラム*uṣūl* · **形状**: 扇形(4象限 + 4拡張チップ)

## ヒーロー・メタファー

ドームはスンニ派*uṣūl al-fiqh*における**4つの権威的証明の源泉**に分割
されます: クルアーン、スンナ、ijmāʿ(学者の合意)、qiyās(類推推論)。
それぞれが一つの証明の異なる*程度*ではなく異なる*種類*の証明であり、
そのためレイアウトは扇形(カテゴリ的スライス)であり同心円(段階的深さ)
ではありません。ドームの下には、4つの補助源泉がチップとして配置されます:
*istiḥsān*(法的選好)、*istiṣḥāb*(継続性の推定)、*maṣlaḥa mursalah*
(制限なき公共利益)、*ʿurf*(慣習的実践)。

pramāṇa と同様、象限は +π/4 回転されました(§θ-fix-1, 2026-05-18)、
垂直軸をStratumラベルとの衝突から開放するため — そのため幾何学的位置は
元々文書化されたNE/SE/SW/NWではなく、現在はE/S/W/Nです。

## スコープ

**この伝統を使うべき時。** スンニ派イスラム法学者的推論として分析される
か、分析され得るコンテンツを扱うとき。導出における証明種別のバランスを
見るのに有用: 議論はクルアーンに強く根ざしているか? 合意に依拠している
か? qiyāsが大半の仕事をしているか? 4つの拡張チップは、古典的uṣūlが見出
しの4源泉以上を認識していることを視覚的に思い出させるものです。

**この伝統を使うべきでない時。** 非イスラム的コンテンツに対しては象限
ラベルが意味をなしません。フレームワークはスンニ派固有でもあります —
十二イマーム派シーア派のuṣūlはqiyāsを ʿaql(理性)に置き換えるもので、
宗教系譜ルール(オリエンテーションv2.09)に基づき意図的に含まれていま
せん。神秘主義的、哲学的、文学的コンテンツには適しません。

## 適用範囲

- スンニ派fiqh導出、*uṣūl al-fiqh*講座、ファトワー分析。
- 法学者的著述におけるクロス源泉バランス監査。
- 古典イスラム法学の証明種別構造を教える。

## 系譜

古典的スンニ派 uṣūl al-fiqh — イスラム法的推論の源泉と方法の学。
4源泉正典はスンニ派4学派(ハナフィー、マーリク、シャーフィイー、ハンバル)
全体で慣習的であり、各源泉の重み付けには内部的バリエーションがあります。
Constellationのレンダリングはal-Ghazālī『*Mustaṣfā*』ラインに従います。

## 批評

ijmāʿを*ijtihādī*(推論派生)クラスターに配置することは、*naṣṣ*(テキ
スト的伝達)クラスターよりも、Ashʿarī/Māturīdī kalāmによって争われて
います。これらの学派はijmāʿを拘束的伝達として扱います。Constellationは
Mustaṣfā準拠の解釈を出荷します; 代替kalām解釈はv4.1の磨き上げ目標です。
4源泉正典はまた、4学派全体の教義的相違を平坦化します — ハナフィー固有
またはマーリク固有のバリアント・レジスターを後に追加できます。

シーア派uṣūlの除外は、製品設計の選択(オリエンテーションv2.09の宗教
系譜ルール)であり、学術的判断ではありません。

## 引用

**一次資料。** Abū Ḥāmid al-Ghazālī, *al-Mustaṣfā min ʿilm al-uṣūl*,
ed. Ḥamza ibn Zuhayr Ḥāfiẓ (Medina: al-Jāmiʿa al-Islāmiyya, 1413/1993).

**現代。** Franz Rosenthal, *Knowledge Triumphant: The Concept of
Knowledge in Medieval Islam* (Leiden: Brill, 1970); Wael B. Hallaq,
*A History of Islamic Legal Theories* (Cambridge: Cambridge University
Press, 1997).

## ノート単位のフロントマター

`masadir_source: quran | sunnah | ijma | qiyas`。Rust側の
`LayoutCacheRow`拡張がランディングすると、このフィールドがデフォルト
配置(現在すべてのノート → クルアーン)を上書きします。拡張チップ源泉
の `istihsan | istishab | maslaha | urf` を介したノート単位のオプトインは
フォローアップです。
