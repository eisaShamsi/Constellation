---
translation_status: AI-generated 2026-06-09 — native-speaker review recommended
language: tr
source: docs/help.uConstellation.World/Sky View/Sky View.md
aliases:
  - Sky View
  - Gökyüzü Görünümü
  - GraphMind
  - Bağlantı yıldız görünümü
  - Ağ görünümü
  - Not bağlantıları
  - 3D grafik
description: Constellation'ın GraphMind motoruyla çalışan etkileşimli Sky View görünümünü kullanarak notlarınız arasındaki bağlantıları görselleştirin ve keşfedin.
---

# Sky View

Sky View, notlarınızı düğümlerden ve bağlantılardan oluşan etkileşimli bir ağ olarak görüntüler; **GraphMind** motoruyla (Pixi.js WebGL) çalışır. Her düğüm bir nottur ve her çizgi notlar arasındaki bir `[[wikilink]]` bağlantısını temsil eder. Bir notun ne kadar çok bağlantısı varsa, düğümü o kadar büyük görünür.

## Sky View'ı açma

| Yöntem | İşlem |
|--------|--------|
| **Mission Control** | `Ctrl+P` tuşuna basın, "star view" yazın |
| **Klavye** | `Ctrl+G` |

Sky View'ı kapatmak için `Escape` tuşuna basın.

> [!note]
> Sky View şerit simgesi sol yuvadan kaldırıldı. Sky View artık klavye kısayolu veya Mission Control üzerinden erişilebilir. Sky View (OrgChart) modu, Not Yönetimi kenar çubuğunda bir sekme olarak mevcuttur.

---

## Grafikle etkileşim

### Temel etkileşimler

| Girdi | Davranış |
|-------|----------|
| **Kaydırma** | Boş alana tıklayıp sürükleyin |
| **Yakınlaştırma** | Fare tekerleği (2D) veya `Ctrl+Tekerlek` (3D) |
| **Düğümleri sürükleme** | Yeniden konumlandırmak için herhangi bir düğüme tıklayıp sürükleyin |
| **Üzerine gelme** | Durum çubuğunda not adını gösterir, bağlı düğümleri ve kenarları vurgular |
| **Bir düğüme tıklama** | O notu düzenleyicide açar |
| **Bir düğüme çift tıklama** | O düğüme yakınlaşır ve ortalar |
| **Bir düğüme sağ tıklama** | Bağlam menüsünü açar |

### Bağlam menüsü

Erişmek için herhangi bir düğüme sağ tıklayın:

| İşlem | Açıklama |
|--------|-------------|
| **Aç** | Notu düzenleyicide açar |
| **Odakla** | Bu düğüme ortalanmış odak moduna girer |
| **Sabitle** | Düğümü mevcut konumunda kilitler. Sabitlemeyi kaldırmak için tekrar tıklayın. |
| **Gizle** | Düğümü grafikten gizler. Gizli düğümleri yeniden göstermek için araç çubuğundaki "Tümünü göster" seçeneğini kullanın. |

---

## 3D gezinme

Sky View tam 3D gezinmeyi destekler — notlarınızın içinde, yıldızların arasında geziniyormuş gibi uçun.

### 3D moduna girme

Grafiği 3D uzayda döndürmek için **orta tıklayıp sürükleyin** (veya **Alt+tıklayıp sürükleyin**). Döndürdükten sonra 3D gezinme kontrolleri etkinleşir.

### 3D kontrolleri

| Girdi | İşlem |
|-------|--------|
| **Orta tıkla-sürükle** | X ve Y eksenleri etrafında döndür |
| **Shift+Orta tıkla-sürükle** | Z ekseni etrafında döndür |
| **W / Yukarı Ok** | İleri uç (ekrana doğru) |
| **S / Aşağı Ok** | Geri uç |
| **A / Sol Ok** | Sola kay |
| **D / Sağ Ok** | Sağa kay |
| **Q** | Aşağı hareket et |
| **E** | Yukarı hareket et |
| **Ctrl+Tekerlek** | Yakınlaştır (görüş alanını değiştir) |
| **Normal Tekerlek** | Kamera yönü boyunca ileri/geri uç |
| **0** | Döndürmeyi düz 2D görünüme sıfırla |
| **Sıfırlama düğmesi** (↺ simgesi) | `0` tuşuna basmakla aynı |

### XYZ eksen göstergesi

3D modundayken, sol alt köşede renk kodlu bir eksen kılavuzu belirir:

| Eksen | Renk | Yön |
|------|-------|-----------|
| **X** | Kırmızı | Sol–Sağ |
| **Y** | Yeşil | Yukarı–Aşağı |
| **Z** | Mavi | İleri–Geri (derinlik) |

Gösterge kamerayla birlikte döner, böylece yönelimi her zaman bilirsiniz.

### 3D'de üzerine gelme ve tıklama

3D'de gezinirken düğümlerin üzerine gelebilir ve onlara tıklayabilirsiniz. Not adı durum çubuğunda görünür ve tıklamak notu açar — tıpkı 2D modundaki gibi.

---

## Düzen modları

Sky View üç düzen algoritması sunar. Aralarında geçiş yapmak için `Ctrl+L` tuşuna basın veya araç çubuğundaki düzen düğmesini kullanın.

| Mod | Açıklama | En uygun |
|------|-------------|----------|
| **Organik** | Kuvvet yönelimli düzen. Kümeler, bağlantı yoğunluğundan doğal olarak ortaya çıkar. | Genel keşif — varsayılan mod. |
| **Hiyerarşik** | Yukarıdan aşağıya yönlü çevrimsiz grafik (DAG). | Üst–alt ilişkileri olan yapılandırılmış kütüphaneler. |
| **Zamansal** | Düğümler, oluşturulma tarihine göre yatay bir zaman ekseni boyunca düzenlenir. | Notların ne zaman oluşturulduğunu ve kütüphanenin nasıl büyüdüğünü görmek. |

Modlar arasında geçiş, mekânsal yöneliminizi koruyan akıcı, animasyonlu bir geçiş tetikler.

> [!tip]
> Hiyerarşik mod, ağaç benzeri bir yapıyı izleyen notlar için özellikle kullanışlıdır (ör. alt konulara bağlanan İçerik Haritaları). Zamansal mod, entelektüel zaman çizelgenizi ortaya çıkarır — ilgili not kümelerinin ne zaman oluşturulduğunu.

---

## Odak modu

Odak modu, yalnızca belirli bir notu ve onun komşuluğunu gösterir. Dinamik, etkileşimli bir yerel grafiktir.

### Odak moduna girme

- **Bir düğüme sağ tıklayın** → **Odakla**
- Mevcut etkin notta odak modunu açıp kapatmak için **Boşluk tuşuna basın**

### Odak kontrolleri

Odak modundayken, üstte bir kontrol çubuğu belirir:

| Kontrol | Açıklama |
|---------|-------------|
| **Derinlik kaydırıcısı** (1–5) | Kaç adım bağlantının gösterileceği. 1 = yalnızca doğrudan bağlantılar, 5 = beş seviye derinlik. |
| **Yön filtresi** (↔ / ← / →) | Tüm bağlantıları, yalnızca geleni veya yalnızca gideni göster. |
| **Çıkış düğmesi** (×) | Tam Sky View'a geri dön |

### Gezinme izi

Odak modunda düğümler arasında tıkladıkça, üstte gezinme yolunuzu gösteren bir iz belirir. O notun yerel grafiğine geri dönmek için herhangi bir ize tıklayın.

> [!tip]
> Bir notun komşuluğunu kademeli olarak keşfetmek için odak modunu derinlik kaydırıcısıyla birleştirin. Doğrudan bağlantıları görmek için derinlik 1'de başlayın, ardından ikinci ve üçüncü derece ilişkileri keşfetmek için artırın.

---

## Aramayla vurgulama

Arama çubuğunu açmak için `Ctrl+F` tuşuna basın. Eşleşen notları vurgulamak için bir sorgu yazın.

Bir filtreden farklı olarak, aramayla vurgulama eşleşmeyen düğümleri kaldırmadan **soluklaştırır**. Eşleşen düğümler vurgulanırken, tüm grafik yapısını ve mekânsal bağlamı korursunuz.

> [!tip]
> Arama hem tam grafikte hem de odak modunda çalışır. 3D modundayken de arama yapabilirsiniz.

---

## Ayarlar paneli

Ayarlar panelini açmak için araç çubuğundaki dişli simgesine (⚙) tıklayın. Üç sekmesi vardır:

### Grafik Görünümü

| Kontrol | Açıklama | Varsayılan |
|---------|-------------|---------|
| **Düğüm boyutu** | Tüm düğümleri daha büyük veya daha küçük ölçeklendir | 1.5 |
| **Etiket görünürlüğü** | Etiketlerin ne zaman görüneceği: Üzerine gelince, Her zaman veya Hiçbir zaman | Üzerine gelince |
| **Etiket yazı boyutu** | Not adı etiketlerinin boyutu | 12 |
| **Bağlantı kalınlığı** | Kenar çizgilerinin genişliği | 1 |
| **Yetim notları göster** | Bağlantısı olmayan notları dahil et | Açık |

> **Tuval arka plan rengi.** Baloncukların arkasındaki renk, bu panelde değil, **Ayarlar → Style Setter → Sky View → Tuval → Arka plan** içinde ayarlanır. Kenar çubuklarınızdan/panellerinizden bağımsızdır, böylece arayüzün geri kalanını değiştirmeden grafiğe kendine özgü bir arka plan verebilirsiniz — örneğin baloncukları öne çıkarmak için koyu bir renk. Ayarlanmadan bırakıldığında, tuval panel yüzeyiyle eşleşir. Bkz. *Görünüm ve Temalar → Sky View tuvali*.

### Fizik

| Kontrol | Açıklama | Varsayılan |
|---------|-------------|---------|
| **İtme kuvveti** | Düğümlerin birbirini ne kadar güçlü ittiği | 50 |
| **Bağlantı kuvveti** | Bağlı düğümlerin birbirini ne kadar güçlü çektiği | 0.05 |
| **Bağlantı mesafesi** | Bağlı düğümler arasındaki hedef mesafe | 30 |
| **Simülasyonu yeniden ısıt** | Kuvvet düzenini mevcut durumdan yeniden başlat | — |

### Yapay Zekâ

Anlamsal yapay zekâ bağlantıları için ayarlar (Faz 2 — yerel gömme modeli gerektirir).

| Kontrol | Açıklama |
|---------|-------------|
| **Anlamsal bağlantıları göster** | Yapay zekânın tespit ettiği kesik çizgili kenarları aç/kapat |
| **Güven eşiği** | Anlamsal bağlantıları benzerlik puanına göre filtrelemek için kaydırıcı |

---

## Açıklama (Lejant)

Açıklama sağ alt köşede belirir ve kütüphaneleriniz için renk atamalarını gösterir.

### Renk modu değiştirme

Düğümlerin nasıl renklendirileceğini değiştirmek için açıklamanın üst kısmındaki **Kütüphane** veya **Klasör** düğmelerine tıklayın:

| Mod | Renklendirme |
|------|----------|
| **Kütüphane** | Her kütüphane benzersiz bir renk alır |
| **Klasör** | Her üst düzey klasör benzersiz bir renk alır |

### Görünürlük onay kutuları

Her açıklama girişinin bir onay kutusu vardır. Düğümlerini grafikten gizlemek için bir kütüphanenin veya klasörün işaretini kaldırın. Bu, bilgi tabanınızın belirli alt kümelerine odaklanmanızı sağlar.

> [!tip]
> Klasör modundayken, klasör sayısı parantez içinde gösterilir. Uzun klasör listeleri kaydırılabilir.

---

## Durum çubuğu

Sol alttaki durum çubuğu şunları gösterir:

- **Düğüm sayısı** — toplam görünür düğüm
- **Kenar sayısı** — toplam görünür kenar
- **İçerik Haritası sayısı** — İçerik Haritalarının (yüksek bağlantılı hub notları) sayısı
- **Üzerine gelinen not adı** — bir düğümün üzerine geldiğinizde belirir

---

## Klavye kısayolları

| Kısayol | İşlem |
|----------|--------|
| `Ctrl+G` | Sky View'ı aç |
| `Escape` | Sky View'ı kapat |
| `Ctrl+F` | Aramayla vurgulamayı aç/kapat |
| `Ctrl+L` | Düzen modunu döngüle (Organik → Hiyerarşik → Zamansal) |
| `Space` | Etkin notta odak modunu aç/kapat |
| `0` | 3D döndürmeyi düz 2D'ye sıfırla |
| `W/A/S/D` | 3D uzayda uç (döndürüldüğünde) |
| `Q/E` | 3D uzayda aşağı/yukarı hareket et |

---

## RTL desteği

Sky View; Arapça, İbranice ve diğer sağdan-sola yazılar için birinci sınıf destek sağlar:

- **Düğüm etiketleri** yazı yönünü otomatik algılar — Arapça başlıklar sağdan sola işlenir
- **Açıklama öğeleri** içerik diline göre nokta/metin sırasını ters çevirir
- **İpuçları ve paneller** RTL düzenine uyar
- **Arapça yazı tipi yedeği** — birincil yazı tipinde Arapça karakter kapsamı eksik olduğunda etiketler sistem Arapça yazı tiplerini (Noto Naskh Arabic, Segoe UI) kullanır

---

## Resim içinde Resim (PiP) yer paylaşımı

Sky View açıkken ve Not Yönetimi kenar çubuğunda bir alt evrene, kütüphaneye veya klasöre tıkladığınızda, ana grafiğin üzerinde yeniden boyutlandırılabilir bir yer paylaşımı olarak bir **Resim içinde Resim (PiP)** penceresi belirir.

### PiP neyi gösterir

PiP, yalnızca seçilen kapsama ait düğümleri içeren filtrelenmiş bir alt grafiği görüntüler. Örneğin, bir kütüphaneye tıklamak yalnızca o kütüphanenin notlarını ve aralarındaki bağlantıları gösterir.

### PiP özellikleri

| Özellik | Açıklama |
|---------|-------------|
| **Filtrelenmiş grafik** | Yalnızca seçilen kapsamdaki düğümler görünür |
| **Filtrelenmiş açıklama** | PiP'in yalnızca ilgili girişleri gösteren kendi açıklaması vardır |
| **Yeniden boyutlandırılabilir** | PiP penceresini yeniden boyutlandırmak için kenarlarını veya köşelerini sürükleyin |
| **Yeniden konumlandırılabilir** | PiP'i ekranda herhangi bir yere taşımak için başlık çubuğunu sürükleyin |

### Modlar arası seçim eşitleme

Herhangi bir kenar çubuğu modunda (Ağaç, Liste veya OrgChart) bir alt evrene, kütüphaneye, klasöre veya nota tıklamak, Sky View grafiğindeki ilgili düğümleri vurgular. Bu çift yönlü eşitleme, kenar çubuğunda gezinirken mekânsal farkındalığı korumanıza yardımcı olur.

---

## Bilgi Katmanları

Sky View, düğümleri bilgi seviyelerine (1-8) göre otomatik olarak boyutlandırır:

- Küçük noktalar: basit notlar (Veri Birimi, Enformasyon)
- Orta düğümler: bağlı notlar (Önerme, Kavram)
- Büyük parlayan hub'lar: sentez notları (Teori, Paradigma, Dünya görüşü)

Daha üst seviyedeki düğümlerin, görsel kontrast için tamamlayıcı renkli bir parıltı halesi vardır. Bu, bir kütüphanede 20+ not olduğunda etkinleşir.

---

## Not Olgunluğu

Düğümler, olgunluğu gösteren renkli bir halka görüntüler:

- Halka yok: Tohum (yeni not)
- Açık yeşil halka: Fidan (büyüyen)
- Koyu yeşil halka: Her dem yeşil (iyi yerleşmiş)
- Altın halka: Kanonik (yetkili referans)

Olgunluk ayrıca dosya ağacında (sol kenarlık) ve sekme çubuğunda (renkli nokta) da gösterilir.

---

## Köken Parıltısı

Sky View'daki düğümler, bilginin kökenini gösteren hafif bir renk parıltısı gösterir:

- **Mavi parıltı**: Alınan bilgi — notun kaynak zinciri harici bir referansa kadar izlenir (ön bilgilerinde url, yazar veya doi bulunan bir not)
- **Kehribar parıltı**: Keşfedilen bilgi — notun kaynak zinciri kullanıcının kendi notlarından kaynaklanır

---

## Teknik notlar

Sky View, özel bir Web Worker'da çalışan d3-force simülasyonuna sahip bir Pixi.js WebGL işleyicisi olan **GraphMind** motoruyla çalışır. Bu mimari şunları sağlar:

- **60fps işleme** — binlerce düğümle bile
- **Engellemeyen düzen** — kuvvet simülasyonu arayüzü asla dondurmaz
- **Üzerine gelme yalnızca görseldir** — üzerine gelmek asla fizik yeniden hesaplamasını tetiklemez
- **Simülasyon yerleştikten sonra durur** — düğümler konumlarını bulduğunda fizik motoru tamamen durur. Yalnızca bir düğümü sürüklemek veya ayarları değiştirmek onu yeniden başlatır.
