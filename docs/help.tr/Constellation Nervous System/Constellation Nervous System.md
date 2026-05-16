---
translation_status: AI-generated 2026-05-16 — native-speaker review recommended
language: tr
source: docs/help.uConstellation.World/Constellation Nervous System/Constellation Nervous System.md
aliases:
  - Constellation Nervous System
  - CNS
  - Constellation Sinir Sistemi
description: Constellation Nervous System (CNS) evreninizin bağlantı-geçiş görünümüdür. Notlarınız arasındaki bağlantı grafiğini analiz eder ve Evren Sağlığı metrikleri, topluluklar, kümeler arasındaki üst köprüler ve "Kör Noktalar" yapısal boşlukları yüzeye çıkarır. CNS, Constellation Sight'a tamamlayıcı görünümdür — Sight evreninizin duyusal formuysa, CNS onun sinir bağlantılarıdır.
---

# Constellation Nervous System (CNS)

## Bu nedir?

**Constellation Nervous System**, evreninizin **bağlantı-geçiş** görünümüdür. Constellation Sight notlarınızın *şeklini* (katman × zaman × kanal kodlama) gösterirken, CNS *kabloları* gösterir — onları bağlayan türlenmiş bağlantı grafiği ve o grafikte gizli yapısal kalıplar.

Yanıtlar: **"Evrenimdeki fikirler nasıl bağlı ve boşluklar nerede?"**

Görünüm dört analitik yüzey etrafında inşa edilmiştir:
- **Evren Sağlığı** — bilginizin ne kadar bağlı, dengeli ve modüler olduğu için genel ve metrik bazında puanlar.
- **Topluluklar** — yoğun şekilde birbirine bağlı notların grupları ("ideolojik kümeler").
- **Üst Köprüler** — aksi takdirde ayrılmış toplulukları birbirine bağlayan az sayıda not ("yük taşıyan bağlayıcılar").
- **Kör Noktalar** — bağlantı beklediğiniz ancak henüz sahip olmadığınız yapısal boşluklar.

"Nervous System" adı anatomiktir: sinirler, bir organizmanın uzak parçaları arasında sinyal taşıyan bağlantı yollarıdır. CNS, türlenmiş bağlantı grafiğinizi aynı şekilde ele alır.

## Neden önemli?

Çoğu not uygulaması bağlantıları tesisat olarak ele alır (buradan oraya atla). Constellation bunları **bilgi mimarisi** olarak ele alır:

- Çok sayıda gelen bağlantısı olan bir not **yük taşır** — birçok fikir ona bağlıdır.
- İki topluluğu köprüleyen bir not **sentez noktasıdır**.
- Zayıf iç bağlantısı olan bir topluluk **kırılgandır**.
- "Kör Nokta" yapının bağlantısı OLMASI GEREKEN ancak olmayan bir yerdir — keşfedilmeye değer bir hipotez.

## Nasıl açılır

1. Constellation sol kenarındaki dock'ta **nöron simgesine** (küçük dallanmış sinir hücresi şekli — ortada hücre gövdesi, üç dendrit dalı ve sinaptik terminaller) tıklayın.
2. CNS tam pencere kaplamasında, yerçekimi kuyusu stilinde açılır — kuvvet yönlendirmeli grafik, her not bir düğüm, her türlenmiş bağlantı bir kenar.
3. Kapatmak için: üstteki **(×)**'e tıklayın veya **Esc** tuşuna basın.

## Ne görürsünüz

### Evren Sağlığı Kartı

Evreninizin genel bağlantı sağlığını gösteren özet paneli, bileşik puanın altın halkası (ör. **91 / 100**) ve dört metrik:

- **Modularity** — notlarınızın farklı topluluklara ne kadar düzgün kümelendiği.
- **Dominance** — bir topluluğun evrene hakim olup olmadığı.
- **Entropy** — topluluk boyutlarının çeşitliliği.
- **Connectivity** — not başına ortalama bağlantı.

Her metriğin renkli durum hapı vardır: **HEALTHY** (yeşil) / **CAUTION** (sarı) / **IMBALANCED** (kırmızı).

### Yerçekimi Kuyusu

Ana görselleştirme: notlar düğüm olarak yüzer, bağlantılar onları birlikte çeker, itme onları ayırır. Topluluklar kümelerde kendi kendine organize olur.

- **Düğüm boyutu** = bağlantı sayısı.
- **Düğüm rengi** = topluluk üyeliği.
- **Kenar** = iki not arasındaki türlenmiş bağlantı.

### Üst Köprüler

En farklı toplulukları bağlayan notların listesi — bunlar sentez noktalarınızdır.

### Topluluklar

Tespit edilen not kümelerinin listesi.

### Kör Noktalar (Yapısal Boşluklar)

Önerilen eksik bağlantılar — grafik algoritmasının bağlanması GEREKEN düşündüğü not çiftleri.

## Etkileşim

CNS, **tek tıklama önizleme / çift tıklama açma** modelini kullanır (Sight'ın tek tıklama açmasından farklı):

| Hareket | Etki |
|---|---|
| **Düğüme tek tıklama** | Onu seçer. Sağ kenar paneli kaydırarak başlık, topluluk, merkeziyet sıralaması, gelen/giden bağlantıları gösterir. Not açılmaz. |
| **Düğüme çift tıklama** | Notu editörde açar. **"Return to CNS"** düğmesi görünür. |
| **Düğüm üzerinde hover** | Başlıklı araç ipucu. |
| **Boş alana tıklama** | Seçimi temizler. |
| **Fare tekerleği** | Zoom in/out. |
| **Tıkla + sürükle** | Pan. |
| **Listede topluluğa tıklama** | Kuyuda topluluğun notlarını vurgular. |
| **Üst Köprü girişine tıklama** | Köprü notuna odaklanır. |
| **Esc** | CNS'yi kapatır. |

Tek tıklama önizlemesi kasıtlıdır: birçok notun ayrıntılarını (ve bağlantılarını) editörde her birini açmaya bağlı kalmadan tarayabilirsiniz.

## CNS Ne Zaman En Yararlıdır

- **Bağlantı yoğunluğunuzu denetleyin** — Universe Health bir bakışta okuma verir.
- **Sentez noktalarınızı bulun** — Top Bridges en çok mimari iş yapan notları gösterir.
- **Var olduğunu bilmediğiniz toplulukları keşfedin** — grafikten ortaya çıkan kümeler.
- **Kör Noktaları yamalayın** — grafiğin iki notun bağlanması GEREKEN ancak olmadığını önerdiği zaman.
- **Yeniden organizasyon planlayın** — topluluklar doğal olarak klasör yapısına eşlenir.

## CNS vs Sight — Ne Zaman Hangisini Kullanmalı

- **Sight** = "Evrenim nasıl **ŞEKİLLENMİŞ**?" Mekansal / kategorik analiz.
- **CNS** = "Evrenim nasıl **BAĞLI**?" Ağ / topolojik analiz.

Tamamlayıcıdırlar: Sight yüzeyi okur; CNS altındaki kabloları okur.

## İlgili Yüzeyler

- **Constellation Sight** — kardeş görselleştirme (dock'taki göz simgesi).
- **Sky View** — grafik görünümü de, ancak farklı inşa edilmiş.
- **Backlinks / Outgoing Links panelleri** — not başına bağlantı listeleri.
