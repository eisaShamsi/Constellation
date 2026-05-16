---
translation_status: AI-generated 2026-05-16 — native-speaker review recommended
language: tr
source: docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md
aliases:
  - Constellation Sight
  - Koordineli Görünümler
  - Çapa Kubbe
description: Constellation Sight tüm bilgi evreninizi katmanlı bir çapa kubbe olarak, yanında aynı notları farklı kanallardan (Güven, Aşama, Eylemler, Köken) yeniden kodlayan dört koordineli mini kubbe ile görselleştirir. Herhangi bir yıldızın üzerine gelin her yerde görün; kenar çubuğu çiplerine tıklayın veya yıldızlara Shift+tıklayın filtreleyin; herhangi bir mini kubbeyi tam boyutta zoom ile incelemek için yükseltin.
---

# Constellation Sight

## Bu nedir?

**Constellation Sight** bilgi evreniniz için **tanı aracıdır**. Merkezi bir **çapa kubbe** her notu **katman** (düşüncenin derinliği) ve **zaman** (ne zaman yazıldığı) ile konumlandırılmış olarak gösterir, yanında dört **mini kubbe** aynı evreni farklı kanallardan yeniden kodlar: **Confidence**, **Stage**, **Acts**, **Provenance**.

Beş tamamlayıcı merceği olan bir soruya yanıt verir: **"Epistemik İçeriğim nasıl şekillenmiş ve organize edilmiştir?"**

Herhangi bir kubbede herhangi bir yıldızın üzerine gelin, aynı not beş yüzeyin tümünde yanar — yıldız etrafında altın halka, kenar çubuğundaki eşleşen çiplerde altın ton. Kenar çubuğunda bir çipe tıklayın, beş görünüm de daralır. Stage mini-kubbesinde bir yıldıza Shift+tıklayın, evren anında o yaşam döngüsü aşamasına filtrelenir. Herhangi bir mini-kubbenin boş alanına tıklayın ve birincil yuvaya tam boyutta "yükseltilir", önceki birincil ise boşalan mini yuvasına iner.

## Neden önemli?

Çoğu not uygulaması ne yazdığınızı gösterir. Constellation Sight bildiklerinizin **şeklini** gösterir.

- Düşünceniz nerede **yoğunlaşmış**? (çapadaki yoğunluk gradyanı)
- Ne hâlâ **erken aşamada** ve ne **istikrarlı temel**? (Stage mini renk gradyanı)
- Hangi notlar **yük taşıyor** ve hangileri **izole**? (Acts mini boyut kodlaması)
- Her fikir nereden geldi — kendi düşünceniz, okuma, dinleme, gelenek? (Provenance mini sektör düzeni)
- Sonuçlarınıza ne kadar **güveniyorsunuz**? (Confidence mini opaklık gradyanı)

Çapanın merkezindeki bir not (yüksek bağlantı → yük taşıyan) ancak Stage mini'de cyan (`spark` — yeni kıvılcımlanmış) tanısal bir şey söylüyor: henüz olgunlaşmamış yük taşıyan bir fikir.

## Nasıl açılır

1. Constellation'ın sol kenarındaki dock'ta **göz simgesine** tıklayın.
2. Çapa kubbe çoğu evrende 2–5 saniye içinde işlenir.
3. Kapatmak için: sağ üstteki **(×)**'e tıklayın veya **Esc** tuşuna basın.

## Ne görürsünüz

### Başlık Şeridi

Sight görünümünün üstü, soldan sağa:
- **"Constellation Sight"** — başlık.
- **"v6.1 — Coordinated Views (Phase 2)"** — sürüm alt başlığı.
- **Altın "X / Y notes" rozeti** — yalnızca bir filtre aktifken görünür.
- **Altın küçük büyük harfli "EXTENDED" rozeti** — yalnızca Genişletilmiş görünüm açıkken görünür.
- **"Reset View" düğmesi** — yalnızca düzen değiştirildiğinde görünür.
- **(×)** kapatma düğmesi — her zaman mevcut.

### Çapa Kubbe (Birincil Yuva)

Ortadaki büyük krem-üzeri-koyu kubbe:
- **Katman halkaları** — 5 eş merkezli daire. En içteki = en temel notlarınız; dış kenar = en yeni kıvılcımlarınız.
- **Takvim kenarı** — dışta 12 ay etiketi.
- **Katman etiketleri** — üstte italik metin.
- **Yıldızlar** — her not küçük krem nokta olarak, katman × zaman ile konumlandırılmış.
- **Bağlantı çizgileri** — notlar arasındaki türlenmiş bağlantı kenarları, yıldızlar altında düşük opaklık.
- **Hover halkası** — imlecin üzerinden geçtiği herhangi bir yıldızın etrafında altın daire.

### Dört Mini Kubbe

Sağ taraf, 2×2 ızgara. Varsayılan olarak gizli; **Ctrl+D** (yalnızca oturum) veya **Ctrl+Shift+D** (kalıcı — Genişletilmiş görünüme bakın) ile görüntülenir.

Her mini aynı evreni bir kodlama aracılığıyla işler:

1. **CONFIDENCE — opaklık.** Daha güvenli notlar daha parlak; geçici olanlar solar.
2. **STAGE — renk (tam disk).** Yaşam döngüsü aşamasına göre kategorik renk:
   - **Cyan** = `spark` (yeni kıvılcımlanmış fikir)
   - **Turuncu** = `birth` (şekil alıyor)
   - **Mor** = `growth` (aktif harekette)
   - **Yeşil** = `maturity` (tamamen oluşmuş)
   - **Sarı** = `renewal` (son zamanlarda yeniden ziyaret edildi)
   - **Gri** = `dormancy` / `archival` (etkin değil / kapalı)
3. **ACTS — boyut (üst onluk).** Bağlantı sayısının üst %10'u = daha büyük noktalar; geri kalanı küçük.
4. **PROVENANCE — 5 sektör.** Yıldızlar 5 açı sektörüne yeniden konumlandırılır: **Self / Read / Heard / Reasoned / Tradition**.

### Derecesi Düşürülmüş Çapa (Mini Yükseltildiğinde)

Herhangi bir mini'yi birincil yuvaya yükseltirseniz, çapa boşalan mini yuvasına iner. Orada **nötr krem noktalar** olarak **"UNIVERSE — primary view"** başlığıyla işlenir.

### Faset Kenar Çubuğu (Sol Kenar)

**6 filtre faset grubu** olan katlanabilir panel, her biri canlı sayımlarla kategorileri görüntüler:

- **Folder** — klasör hiyerarşisi
- **Library** — kitaplık adları
- **Stratum** — Foundation / Roots / Trunk / Branches / Twigs / Edge of Knowing
- **Confidence** — Hypothesis / Evidence / Established / Contested
- **Stage** — Spark / Birth / Growth / Maturity / Dormancy / Renewal / Archival
- **Provenance** — Self / Read / Heard / Reasoned / Tradition

Genişletmek için kenarda **▶**'ye tıklayın. Filtre olarak değiştirmek için herhangi bir çipe tıklayın.

## Etkileşim

| Hareket | Etki |
|---|---|
| **Yıldızın üzerinde hover** | 5 yüzeyin tümündeki aynı yıldızda altın halka + kenar çubuğunda eşleşen çipler altın ton. |
| **Yıldıza basit tıklama** | Notu editörde açar. **"Return to Sight"** düğmesi görünür. |
| **Yıldıza Shift+tıklama** Stage / Confidence / Provenance mini'sinde | O yıldızın kategorisinin filtresini değiştirir. |
| **Yıldıza Shift+tıklama** Acts veya çapada | Etki yok. |
| **Mini'nin boş alanına tıklama** | O mini birincil yuvaya yükselir. |
| **Tekerlek zoom (birincil)** | İmlece doğru zoom. Aralık: 0.5× - 24×. |
| **Boş alana tıkla+sürükle** | Görünümü pan. |
| **Ctrl+0 / Cmd+0** | Zoom + pan'ı sıfırla. |
| **Ctrl+D / Cmd+D** | Mini kubbe görünürlüğünü değiştir — **yalnızca oturum**. |
| **Ctrl+Shift+D / Cmd+Shift+D** | **Genişletilmiş görünüm**'ü değiştir — kalıcı. |
| **Kenar çubuğu çipine tıklama** | Filtre setinde faset kategorisini değiştir. |
| **Reset View düğmesi** | Zoom 1.0'da çapa birincile dön. |
| **Esc** | Sight'ı kapat. |

## Hayalet Modu — Kubbeden Çoklu Seçim

Herhangi bir filtre aktifken, eşleşmeyen yıldızlar kaybolmak yerine **düşük opaklıkta (15%)** görünür kalır. Bu şu anlama gelir:

- Eşleşmeyen yıldızların nerede olduğunu hâlâ **görebilirsiniz**.
- Üzerlerinde hover yapabilirsiniz (altın halka görünür).
- **Kategorilerini filtreye EKLEMEK için Shift+tıklayabilirsiniz**.

## Yoğunluk Modu

Görünür (eşleşen) yıldız sayısı yoğunluk eşiğini (varsayılan **5,000**) aştığında, mini kubbeler **algısal yoğunluk işlemeye** geçer.

## Genişletilmiş Görünüm

**Ctrl+Shift+D** (veya Mac'te **Cmd+Shift+D**) basmak "Genişletilmiş görünüm"ü değiştirir — açıkken, mini kubbeler Sight'ı her açtığınızda varsayılan olarak görünür. Durum Sight kapanışları, uygulama yeniden başlatmaları ve sistem yeniden başlatmaları boyunca kalıcıdır.

## Sight Ne Zaman En Yararlıdır

- **Bilgi şekli denetimi** — yazma oturumundan sonra Sight'ı açın.
- **Kör nokta bul** — kubbenin az notla olan sektörleri keşfedilecek alanlar olabilir.
- **Yük taşıma zayıflığını tespit et** — merkezi konumlu erken aşama renk notu.
- **Filtrele ve incele** — Shift+tıklama evreni daraltır; bir mini'yi tam boyutta kanal çalışmak için yükseltin.
- **Epistemik kökeni izle** — bilginizin nasıl kaynaklandığını görmek için Provenance'ı yükseltin.

## İlgili Yüzeyler

- **Constellation Nervous System (CNS)** — tamamlayıcı görselleştirme (dock'ta Sight göz simgesinin yanındaki nöron simgesi).
- **Constellation Map** — güneş ışını görselleştirme.
- **Sky View** — grafik tabanlı bağlantı görselleştirme.
- **Index paneli** — terim tarayıcısı.
