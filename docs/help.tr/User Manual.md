# Constellation Kullanım Kılavuzu

**Sürüm 0.3.4 | Mart 2026**

Constellation, Markdown not kütüphanelerini yönetmek için tasarlanmış bir Kişisel Bilgi Yönetimi (PKM) masaüstü uygulamasıdır. Tauri v2, SvelteKit ve Rust ile geliştirilmiş olup Windows, macOS ve Linux'ta tam Arapça ve RTL desteğiyle yerel olarak çalışır.

---

## İçindekiler

1. [Başlarken](#başlarken)
2. [Evren ve Kütüphaneler](#evren-ve-kütüphaneler)
3. [Not Oluşturma ve Düzenleme](#not-oluşturma-ve-düzenleme)
4. [Arama](#arama)
5. [Sky View (GraphMind)](#star-view-graphmind)
6. [Bölünmüş Görünüm](#bölünmüş-görünüm)
7. [Dizin](#dizin)
8. [İkinci Ekran](#ikinci-ekran)
9. [Özellikler ve Frontmatter](#özellikler-ve-frontmatter)
10. [Şablonlar](#şablonlar)
11. [Tablolar](#tablolar)
12. [Görevler](#görevler)
13. [İçe Aktarıcı](#içe-aktarıcı)
14. [Takvim](#takvim)
15. [Lens](#lens)
16. [Ayarlar](#ayarlar)
17. [Klavye Kısayolları](#klavye-kısayolları)
18. [RTL ve Arapça Desteği](#rtl-ve-arapça-desteği)
19. [Güvenlik ve Gizlilik](#güvenlik-ve-gizlilik)
20. [Bilgi Haritası](#bilgi-haritası)
21. [Bilişsel Motor](#bilişsel-motor)

---

## 1. Başlarken

### Kurulum

En son yükleyiciyi [Constellation sürüm sayfasından](https://github.com/eisaShamsi/Constellation/releases) indirin:

- **Windows**: `.exe` (NSIS) veya `.msi` yükleyici
- **macOS**: `.dmg` disk kalıbı
- **Linux**: `.AppImage` veya `.deb` paketi

### İlk Açılış

Constellation'ı ilk kez açtığınızda, **Evren Kurulum Sihirbazı** sizi şu adımlarla yönlendirir:

1. **Dilinizi seçin** — 15 dil desteklenmektedir
2. **Bir kütüphane oluşturun veya içe aktarın** — mevcut bir Markdown dosya klasörüne işaret edin ya da sıfırdan başlayın
3. **Evreninize bir ad verin** — evren, tüm kütüphanelerinizin kapsayıcısıdır

### Arayüz Genel Bakışı

| Öğe | Açıklama |
|-----|----------|
| **Kenar Çubuğu (Şerit)** | Gezinme düğmeleri: Dosya ağacı, Arama, Sky View, Takvim, Şablonlar, Ayarlar |
| **Dosya Ağacı** | Kütüphanelerinizdeki notları ve klasörleri tarayın |
| **Düzenleyici** | Markdown notlarınızı okuyun ve düzenleyin |
| **Sekme Çubuğu** | Birden fazla notu sekmelerde açın |
| **Durum Çubuğu** | Kelime sayısı, karakter sayısı, okuma süresi |

---

## 2. Evren ve Kütüphaneler

### Evren Nedir?

**Evren**, tüm kütüphanelerinizi barındıran en üst düzey kapsayıcıdır. Çalışma alanınız veya kütüphane koleksiyonunuz olarak düşünebilirsiniz.

### Kütüphane Nedir?

**Kütüphane**, bilgisayarınızda Markdown (`.md`) dosyaları içeren bir klasördür. Bir evrende birden fazla kütüphane bulundurabilirsiniz — örneğin, biri iş notları ve biri kişisel notlar için.

### Kütüphane Yönetimi

- **Kütüphane ekleme**: Ayarlar > Kütüphaneler > Kütüphane Ekle, veya bir klasörü uygulamaya sürükleyip bırakın
- **Kütüphane kaldırma**: Ayarlar > Kütüphaneler > kütüphane adının yanındaki kaldır düğmesine tıklayın
- **Kütüphane ayarları**: Her kütüphanenin kendi görünüm ayarları (yazı tipleri, renkler) olabilir

### Alt Evrenler

Evrenleri evrenlerin içine yerleştirebilirsiniz. Bir **Alt Evren**, ana evreniniz tarafından başvurulan başka bir evren klasörüdür. Alt evrenlerdeki notlar Sky View'da kendi notlarınızla birlikte görünür ve kütüphaneler arası bağlantılar kesikli çizgiler olarak gösterilir.

### Otomatik yeniden açma

Constellation son aktif evreninizi hatırlar ve başlatıldığında otomatik olarak yeniden açar. Evren taşınmışsa veya yolu değişmişse, Constellation bunu algılar ve yolu otomatik olarak düzeltir.

### Taşınabilir Evrenler

Constellation evrenleri tamamen taşınabilirdir. Evren klasörünü herhangi bir konuma taşıyabilirsiniz — farklı bir sürücü, USB bellek veya başka bir bilgisayar — ve Constellation yeniden açıldığında tüm dahili yolları otomatik olarak algılar ve düzeltir.

Bir evreni taşımak için:
1. Constellation'ı kapatın
2. Evren klasörünü yeni konuma taşıyın veya kopyalayın
3. Constellation'ı açın → Karşılama ekranı görünür (eski yol artık geçerli değil)
4. **Mevcut Evreni Aç**'ı seçin ve yeni konumu gösterin
5. Tüm notlar ve kütüphaneler anında görünür — yollar otomatik olarak düzeltilir

Evren klasör yapısı Obsidian modelini takip eder: notlar doğrudan kök klasörde yer alır, yapılandırma `.constellation/` içinde bulunur.

---

## 3. Not Oluşturma ve Düzenleme

### Not Oluşturma

| Yöntem | İşlem |
|--------|-------|
| **Klavye** | `Ctrl+N` |
| **Dosya Ağacı** | Bir klasöre sağ tıklayın > Yeni Not |
| **Mission Control** | `Ctrl+P` > "New note" |

### Düzenleyici Modları

Constellation, **Ayarlar > Düzenleyici > Düzenleyici türü** bölümünden seçilebilen iki düzenleyici modu sunar:

#### Markdown Düzenleyici (CodeMirror)

İleri düzey kullanıcılar için varsayılan düzenleyici. Doğrudan Markdown yazın:

- **Canlı Önizleme** — yazarken biçimlendirmeyi satır içinde oluşturur
- **Kaynak Modu** — ham Markdown sözdizimini gösterir
- **Biçimlendirme araç çubuğu** — metin seçiminde görünür
- **Eğik çizgi komutları** — hızlı eklemeler için `/` yazın
- **Wikilink otomatik tamamlama** — notları bağlamak için `[[` yazın
- **Çoklu imleç** — `Alt+Click` veya `Ctrl+D`

#### Belge Düzenleyici (TipTap)

Görsel araç çubuğuyla WYSIWYG kelime işlemci deneyimi:

- Kalın, İtalik, Altı Çizili, Üstü Çizili, Vurgulama
- Başlıklar (H1–H3), Metin hizalama
- Madde işaretli listeler, Numaralı listeler, Görev listeleri
- Alıntılar, Kod blokları, Yatay çizgiler
- Tablolar (ekleme, satır ve sütun ekleme/kaldırma)
- Bağlantılar ve Görseller

Her iki düzenleyici de standart Markdown dosyaları olarak kaydeder. Veri kaybı olmadan istediğiniz zaman aralarında geçiş yapabilirsiniz.

### Bilgi Kutuları (Callouts)

Notlar, uyarılar, ipuçları ve diğer açıklamalar için stillendirilmiş bilgi kutusu blokları oluşturun:

```markdown
> [!note] Önemli bilgi
> Bilgi kutusunun içeriği buraya yazılır.

> [!warning] Dikkatli olun
> Bu işlem geri alınamaz.

> [!tip]- Genişletmek için tıklayın
> Daraltılabilir bilgi kutusu içeriği.
```

Desteklenen türler: `note`, `tip`, `warning`, `danger`, `success`, `question`, `failure`, `bug`, `example`, `quote`, `abstract`. Her türün kendine özgü bir rengi ve simgesi vardır. Daraltılabilir yapmak için türden sonra `-` ekleyin (daraltılmış başlar), veya `+` (genişletilmiş başlar).

### Vurgulama Sözdizimi

Metni vurgulamak için çift eşittir işaretiyle sarın:

```markdown
Bu, notunuzdaki ==vurgulanan metin== örneğidir.
```

Canlı Önizleme'de `==` işaretleri gizlenir ve metin sarı arka planla görüntülenir.

### Kod Blokları

Çitli kod blokları bir arka plan rengi ve dil etiketiyle görüntülenir:

````markdown
```javascript
const greeting = "Hello, world!";
```
````

Dil adı kod bloğunun üzerinde bir rozet olarak görünür.

### Görsel Yerleştirme

Notlarınıza doğrudan görsel yerleştirin:

```markdown
![Alternatif metin](https://example.com/image.png)   — harici URL
![[photo.jpg]]                                         — kütüphaneden yerel dosya
```

Canlı Önizleme'de görseller satır içi olarak işlenir. Yerel görseller kütüphane klasörünüzde olmalıdır. Harici görseller internet bağlantısı gerektirir.

### Tablo Araç Çubuğu

İmleciniz bir markdown tablosunun içindeyken, kayan bir araç çubuğu görünür:

- **+ Satır / + Sütun** — satır veya sütun ekleme
- **- Satır / - Sütun** — satır veya sütun kaldırma
- **Hizalama** — sütun başına sola, ortaya veya sağa hizalama
- **Sıralama** — geçerli sütuna göre artan veya azalan sıralama
- **Tab / Shift+Tab** — tablo hücreleri arasında gezinme

### Metin Biçimlendirme Kısayolları

| Kısayol | İşlem |
|---------|-------|
| `Ctrl+B` | Kalın |
| `Ctrl+I` | İtalik |
| `Ctrl+Shift+S` | Üstü Çizili |
| `Ctrl+Shift+H` | Vurgulama |
| `Ctrl+K` | Wikilink ekle |
| `Ctrl+Z` | Geri al |
| `Ctrl+Shift+Z` | Yinele |

### Notları Bağlama

Not otomatik tamamlamayı açmak için `[[` yazın. Bir not adı yazmaya başlayın ve önerilerden seçin. Bağlantılar tıklanabilir wikilink'ler olarak görünür: `[[Note Name]]`.

Belirli başlıklara da bağlantı verebilirsiniz: `[[Note Name#Heading]]`.

---

## 4. Arama

Constellation, SQLite FTS5 tabanlı BM25 sıralama, yapılandırılmış sorgu filtreleri ve Arapçaya optimize edilmiş normalizasyon ile hibrit çok dilli bir arama motoruna sahiptir. Arama, kenar çubuğu araç çubuğundan erişilebilir.

### Nasıl aranır

Kenar çubuğu araç çubuğundaki arama simgesine tıklayın veya arama modunu etkinleştirmek için `Ctrl+Shift+F` tuşuna basın. Sorgunuzu yazın ve sonuçlar kısa bir gecikmeden (300ms) sonra görünür. Aramayı temizlemek ve dosya ağacına dönmek için `Escape` tuşuna basın veya `×` düğmesine tıklayın.

### Arama sözdizimi

| Sözdizimi | Örnek | Ne bulur |
|-----------|-------|----------|
| Serbest metin | `proje yönetimi` | Başlık veya gövdede bu kelimeleri içeren notlar |
| Etiket filtresi | `#araştırma` | `#araştırma` etiketi olan notlar |
| Özellik filtresi | `status=aktif` | Frontmatter özelliği `status` değeri `aktif` olan notlar |
| Vikibağ filtresi | `links to [[İklim]]` | `[[İklim]]`'e bağlantı içeren notlar |
| Kütüphane kapsamı | `in:Kütüphanem` | Sonuçları belirli bir kütüphaneyle sınırlar |
| Birleşik | `#araştırma status=aktif ekonomi` | Tüm filtreler birlikte uygulanır |

### Eşleme türü rozetleri

Her arama sonucu, eşleşmenin nasıl bulunduğunu gösteren renkli bir rozet görüntüler. Rozet, erişilebilirlik için yerelleştirilmiş bir harf gösterir (renk körlüğü için güvenli):

| Rozet | Renk | Anlam |
|-------|------|-------|
| **B** | Mavi | Başlık eşleşmesi — arama terimi notun adında görünür |
| **İ** | Yeşil | İçerik eşleşmesi — arama terimi notun gövdesinde görünür |
| **A** | Mor | Anlamsal eşleme — kavramsal olarak ilişkili (embedding modeli gerektirir) |
| **Ö** | Kehribar | Özellik eşleşmesi — frontmatter özellik filtresiyle bulundu |
| **#** | Pembe | Etiket eşleşmesi — etiket filtresiyle bulundu |
| **V** | Açık mavi | Vikibağ eşleşmesi — vikibağ filtresiyle bulundu |

Rozet harfleri desteklenen 15 dilin tümüne yerelleştirilmiştir.

### Sabitlenmiş sonuçlar (Sonuçlar arasında gezinme)

Arama sonuçları birine tıkladıktan sonra görünür kalır. Açılan not sonuç listesinde vurgulanır, böylece hangi sonucu görüntülediğinizi görebilirsiniz. Yeniden arama yapmadan başka bir sonuca tıklayarak ona gidin.

Aramayı temizlemek için `Escape` tuşuna basın veya `×` düğmesine tıklayın.

### Klavye ile gezinme

| Tuş | Eylem |
|-----|-------|
| `Aşağı ok` | Sonraki sonucu seç |
| `Yukarı ok` | Önceki sonucu seç |
| `Enter` | Seçili sonucu aç |
| `Escape` | Aramayı temizle ve dosya ağacına dön |

### Arama terimi vurgulama

Arama sonuçlarından bir notu açtığınızda, arama teriminin tüm geçişleri editörde vurgulanır. Bu, Arapça harekeli harf tanıma ile çalışır — "ادارة" araması "إدارة" ve tüm hareke varyantlarını vurgular.

### Arama geçmişi

Boş olduğunda arama alanına tıklayarak son aramalarınızı (son 20 sorgu) görün. Her girişte sorgu metni ve ne zaman yapıldığı gösterilir. Herhangi bir girişe tıklayarak o aramayı anında yeniden çalıştırın. Tüm geçmişi silmek için alttaki "Geçmişi temizle" bağlantısını kullanın.

Arama geçmişi cihazınızda yerel olarak depolanır ve uygulama yeniden başlatmaları arasında korunur.

---

## 5. Sky View (GraphMind)

Sky View, notlarınızı **GraphMind** motoru (Pixi.js WebGL) tarafından desteklenen etkileşimli bir 3D grafik olarak görselleştirir.

### Sky View'ı Açma

- Kenar çubuğundaki grafik simgesine tıklayın
- `Ctrl+G` tuşuna basın
- Mission Control (`Ctrl+P`) > "Sky View"

### Gezinme

| Girdi | İşlem |
|-------|-------|
| **Tıkla + sürükle** | Grafiği kaydır |
| **Kaydırma** | Yakınlaştır/uzaklaştır |
| **Bir düğüme tıkla** | Notu aç |
| **Bir düğüme sağ tıkla** | Bağlam menüsü (Aç, Odakla, Sabitle, Gizle) |
| **Orta tıkla + sürükle** | 3D'de döndür |
| **W/A/S/D** | 3D uzayda uç |
| **0** | Döndürmeyi 2D'ye sıfırla |
| **Ctrl+F** | Ara ve vurgula |
| **Space** | Odak modunu aç/kapat |

### Düzen Modları

`Ctrl+L` tuşuna basarak aralarında geçiş yapın:

- **Organik** — kümelerin doğal olarak ortaya çıktığı kuvvet yönlendirmeli düzen
- **Hiyerarşik** — yukarıdan aşağıya ağaç düzeni
- **Zamansal** — notlar oluşturma tarihine göre bir zaman çizelgesinde düzenlenir

### Odak Modu

Bir düğüme sağ tıklayın > **Odakla** ile yalnızca komşuluğunu görün. Ayarlayın:

- **Derinlik** (1–5 atlama) — kaç bağlantı seviyesinin gösterileceği
- **Yön** (↔/←/→) — tüm bağlantılar, yalnızca gelen veya yalnızca giden

### 3D Gezinme

Döndürmek için orta tuşla tıklayıp sürükleyin. Yıldız alanında uçmak için W/A/S/D/Q/E tuşlarını kullanın. Köşedeki XYZ eksen göstergesi yönünüzü gösterir. Sıfırlamak için `0` tuşuna basın.

### Ayarlar

Dişli simgesine tıklayın:

- **Görünüm**: Düğüm boyutu, etiket görünürlüğü, yazı tipi boyutu, bağlantı kalınlığı, yalnızları göster
- **Fizik**: İtme kuvveti, bağlantı kuvveti, bağlantı mesafesi
- **AI**: Anlamsal bağlantı eşiği (Aşama 2)

### Açıklama

Sağ alt köşedeki açıklama, görünürlüğü değiştirmek için onay kutuları ile kütüphane/klasör renklerini gösterir.

### Bilgi Katmanları

Sky View, notlarınızı soyutlama düzeyine göre otomatik olarak sekiz bilgi katmanına sınıflandırır:

| Katman | Açıklama |
|--------|----------|
| **Anlık Görüntü** | Hızlı, geçici notlar |
| **Günlük** | Tarihli olaylar ve günlük kayıtları |
| **Konu** | Tek bir fikir hakkında atomik kavramlar |
| **Harita** | Diğer konuları birbirine bağlayan organizasyon notları |
| **Çerçeve** | Modeller ve düşünce çerçeveleri |
| **İlke** | Doğrulanmış kurallar ve aksiyomlar |
| **İnanç** | Temel değerler ve inançlar |
| **Eser** | Tamamlanmış ve nihai çalışmalar |

Katman, notun frontmatter bilgisi, yapısı ve bağlantılarından otomatik olarak belirlenir. Frontmatter'a `stratum` özelliği ekleyerek sınıflandırmayı elle geçersiz kılabilirsiniz.

### Olgunluk Yaşam Döngüsü

Her not, gelişim derecesini yansıtan bir olgunluk yaşam döngüsünden geçer:

- **Tohum** — İlk fikir veya ham taslak
- **Fidan** — Not şekillenmeye başlar ve bazı bağlantıları vardır
- **Yaprak Dökmeyen** — Olgun, gözden geçirilmiş ve iyi bağlantılı not
- **Kanonik** — Alanında nihai ve yetkili bir referans

Olgunluk seviyesi, bağlantı sayısı, gözden geçirme tarihi ve düzenleme sıklığına göre otomatik olarak güncellenir. Ayrıca frontmatter'daki `maturity` özelliği ile elle de ayarlayabilirsiniz.

---

## 6. Bölünmüş Görünüm

Bölünmüş görünüm, birden fazla notu ana pencerede yan yana düzenlemenizi sağlar.

### Bölünmüş Görünümü Açma

- **Komut Paleti**: `Ctrl+P` ve ardından "Split View" yazın
- **Klavye kısayolu**: Modlar arasında geçiş yapmak için atanmış kısayolu kullanın
- **Döngü**: Kapalı → Dikey (yan yana) → Yatay (üst ve alt) → Kapalı

### Bölünmüş Görünümde Düzenleme

Her panel, aşağıdakilere sahip tamamen bağımsız bir düzenleyicidir:
- Tam araç çubuğu (kalın, italik, başlıklar, hizalama vb.)
- Breadcrumb gezinmesi (kütüphane / not adı)
- Özellikler paneli ve aşama açılır menüsü
- Kaydetme desteği (`Ctrl+S` odaklanmış paneli kaydeder)
- Başlık düzenleme ve dosya yeniden adlandırma

### Panelleri Yeniden Boyutlandırma

Paneller arasındaki ayırıcıyı sürükleyerek yeniden boyutlandırın. Her ayırıcı bağımsızdır — 3 veya daha fazla not açıkken, diğerlerini etkilemeden herhangi bir komşu çifti yeniden boyutlandırabilirsiniz. Hem dikey hem de yatay modlarda çalışır.

### Odak

Herhangi bir panele tıklayarak odaklayın. Odaklanan panel klavye kısayollarını alır ve sağ kenar çubuğu panelleri (Özellikler, Geri Bağlantılar vb.) tarafından izlenir.

---

## 7. Dizin

Dizin, tüm kütüphaneleriniz genelinde kapsamlı bir terim sözlüğüdür — her anlamlı kelime, oluşum sayılarıyla birlikte alfabetik olarak sıralanmıştır.

### Dizini Açma

- **Dock düğmesi**: Sol docktaki Dizin simgesine (kitap) tıklayın
- **Komut Paleti**: `Ctrl+P` ve ardından "Index" yazın

### Çok Dilli NLP İşlem Hattı

Dizin, dizinlemeden önce metni dil farkındalıklı bir işlem hattından geçirir:

- **Arapça**: Lucene Light10 algoritması — teşkili kaldırır, hemzeyi birleştirir, belirli tanımlığı (الـ) kaldırır, dilbilgisi eklerini kaldırır
- **İbranice**: Önek kaldırma (ב/ל/מ/ה/ו/כ/ש)
- **İngilizce**: Porter benzeri kök bulma (çoğullar, fiil formları, ekler)
- **Fransızca/İspanyolca/Portekizce/Almanca**: Dile özgü ek kaldırma
- **Rusça/Türkçe/Hintçe/Farsça**: Morfolojik ek kaldırma
- **Tüm 15 dil**: Etkisiz kelime filtreleme (tanımlıklar, edatlar, bağlaçlar)

### Göz Atma

- **Dil sekmeleri**: Tümü, Arapça, İbranice, İngilizce veya # (özel karakterler) arasında geçiş yapın
- **Alfabe çubuğu**: Bir harfe tıklayarak o harfle başlayan terimleri filtreleyin — terim sayısı güncellenerek kaç eşleşme olduğunu gösterir
- **Aynı harfe tekrar tıklayın** filtreyi temizleyip tüm terimleri göstermek için
- **Sıralama modları**: Alfabetik (varsayılan) veya sıklığa göre (en yaygınlar önce)

### Dizinden Düzenleme

Bir terimin referanslarındaki herhangi bir nota tıklayarak Dizinin yanında bölünmüş bir önizleme panelinde açın. Önizleme paneli tam bir düzenleyicidir — düzenleyebilir, kaydedebilir, özellikleri değiştirebilir ve aşamayı yükseltebilirsiniz. Arama terimi notta vurgulanır ve otomatik olarak kaydırılır.

Notu normal bir sekme olarak açmak için `Ctrl+Tıklama` yapın. Sekme çubuğunda "Dizine Dön" düğmesi görünür — Dizinde kaldığınız yere tam olarak dönmek için tıklayın.

### İkinci Ekran Entegrasyonu

İkinci Ekran açıkken:
- **Bir terime tıklayın** → İkinci Ekran o terimi içeren tüm notları bölünmüş görünümde gösterir (not listesi + düzenleyici)
- **Birden fazla terime Ctrl+Tıklama** → İkinci Ekran her terimi kendi sütununda karşılaştırma modunda gösterir

---

## 8. İkinci Ekran

İkinci Ekran, geçerli kenar çubuğu modunuza uyum sağlayan mod tabanlı bir eşlik penceresidir.

- **Açma**: Kenar çubuğundaki ikinci ekran simgesine tıklayın veya `Ctrl+Shift+2`
- **Otomatik kapanma**: Ana pencereyi kapattığınızda ikinci ekran otomatik olarak kapanır

### Mod Tabanlı Eşlik

İkinci ekran, ana penceredeki aktif kenar çubuğu moduna göre içeriğini değiştirir:

| Kenar Çubuğu Modu | İkinci Ekran Gösterir |
|---|---|
| **Dosya Gezgini** | Evren Paneli — istatistikler, kütüphane dağılımı, alt evrenler, etiketler, son düzenlenen/açılan notlar |
| **Gezgin** | Notlara göz atmak için tam Gezgin görünümü |
| **Gökyüzü Görünümü** | Dizin yapısıyla Gökyüzü Görünümü ağacı |
| **Sky View** | Geri bağlantılar, ileri bağlantılar, etiketler ve yerel grafik ile Sky View eşliği |

### Evren Paneli (Dosya Gezgini Modu)

Ana pencere Dosya Gezgini modundayken, ikinci ekran aşağıdakileri içeren bir panel görüntüler:

- **İstatistik kartları** — Evren adı, alt evren sayısı, toplam kütüphane, klasör ve not sayısı
- **Alt Evrenler** — Her alt evren, bağlı kütüphaneleri ve klasör/not sayılarıyla
- **Kütüphaneler** — Her kütüphane, renk kodlu istatistik kutularında klasör/not sayılarıyla
- **Son Düzenlenen** — Geçerli oturumda değiştirdiğiniz notlar (kaydettiğinizde izlenir)
- **Son Açılan** — Geçerli oturumda açtığınız ancak düzenlemediğiniz notlar
- **Etiketler** — Tüm kütüphanelerdeki tüm etiketler sayıya göre sıralanmış; bir etikete tıklayarak onu kullanan tüm notları görün

### Pano etkileşimi

Ana pencerede pano etkinken, öğelere tıklamak onları ikinci ekrana gönderir:

- **Son Düzenlenen/Açılan**: Bir nota tıklayarak ikinci ekranda tam düzenleyici olarak açın
- **Etiketler**: Bir etikete tıklayarak onu kullanan tüm notları bölünmüş görünümde gösterin — solda not listesi, sağda tam düzenleyici

İkinci ekrandaki tüm düzenlemeler otomatik olarak ana pencereyle senkronize edilir.

### İkinci Ekranda Not Düzenleme

İkinci ekran tam not düzenlemeyi destekler — ana penceredeki gibi yazın, kaydedin, yeniden adlandırın ve özellikleri değiştirin. Değişiklikler otomatik olarak ana pencereyle senkronize edilir.

### Ayar Senkronizasyonu

Tüm görsel ayarlar anında ikinci ekrana yansıtılır — yeniden başlatma gerekmez:

- **Dil**: Arayüz dili değişiklikleri hemen uygulanır
- **Tema**: Açık/koyu/sistem modu anında değişir
- **Yazı tipleri**: Arayüz yazı tipi, metin yazı tipi, monospace yazı tipi ve scripte özgü yazı tipleri
- **Yazı tipi boyutu**: Hem arayüz hem de düzenleyici yazı tipi boyutları
- **Düzenleyici**: Okunabilir satır uzunluğu, satır numaraları, kayan araç çubuğu
- **Vurgu rengi**: Tema vurgu rengi değişiklikleri

---

## 9. Özellikler ve Frontmatter

Notların üst kısmında YAML frontmatter bulunabilir:

```yaml
---
tags: [project, active]
date: 2026-03-19
status: in-progress
---
```

Constellation özellik türlerini otomatik olarak algılar:

| Tür | Örnek |
|-----|-------|
| **Metin** | `author: John` |
| **Sayı** | `priority: 5` |
| **Tarih** | `date: 2026-03-19` |
| **Liste** | `tags: [a, b, c]` |
| **Onay Kutusu** | `done: true` |
| **Bağlantı** | `related: [[Other Note]]` |

Özellik görüntülemeyi **Ayarlar > Düzenleyici > Belgede özellikler** (Görünür / Gizli / Kaynak) bölümünden değiştirin.

---

## 10. Şablonlar

Yeniden kullanılabilir not şablonları oluşturun:

1. Kütüphanenizde şablonlar için bir klasör oluşturun
2. **Ayarlar > Şablonlar** bölümünde şablon klasör yolunu ayarlayın
3. Yeni bir not oluştururken şablon seçiciden bir şablon seçin

Şablonlar değişkenleri destekler:

| Değişken | Yerine geçen |
|----------|--------------|
| `{{date}}` | Geçerli tarih |
| `{{time}}` | Geçerli saat |
| `{{title}}` | Not başlığı |
| `{{clipboard}}` | Pano içeriği |

---

## 11. Tablolar

### Markdown Tabloları

Bir Markdown tablosunu elle yazın veya `/table` eğik çizgi komutunu kullanın:

```markdown
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
```

### Tablo Araç Çubuğu

İmleciniz bir tablonun içindeyken, aşağıdakileri içeren bir kayan araç çubuğu görünür:

- Satır ve sütun ekleme/kaldırma
- Sütun hizalama (sol, orta, sağ)
- `Tab` / `Shift+Tab` ile hücreler arasında gezinme

### Belge Düzenleyicide Tablolar

Belge düzenleyici (TipTap) görsel bir tablo deneyimi sunar:

- Eklemek için tablo düğmesine tıklayın
- Satır/sütun yönetimi için açılır menüyü kullanın
- Kenarlıkları sürükleyerek sütunları yeniden boyutlandırın

---

## 12. Görevler

Constellation, notlarda görev onay kutularını destekler:

```markdown
- [ ] Tamamlanmamış görev
- [x] Tamamlanmış görev
```

Canlı Önizleme modunda, onay kutuları tıklanabilirdir. Görevler, kütüphaneleriniz genelinde aranabilir ve filtrelenebilir.

---

## 13. İçe Aktarıcı

Diğer PKM araçlarından notları içe aktarın:

- **Obsidian** — tam wikilink uyumluluğuyla kasaları içe aktarır
- **Markdown klasörleri** — `.md` dosyaları içeren herhangi bir klasörü içe aktarın
- **Diğer biçimler** — HTML, metin dosyaları

İçe aktarmayı başlatmak için **Ayarlar > İçe Aktarıcı** bölümüne gidin.

---

## 14. Takvim

Takvim görünümü, notları tarihe göre düzenlenmiş olarak gösterir:

- `date` özelliği olan notlar ilgili günlerinde görünür
- Herhangi bir tarih için günlük notlar oluşturulabilir
- Ok düğmeleriyle aylar arasında gezinin

Takvimi kenar çubuğundan açın.

---

## 15. Lens

Lens, notlarınızın filtrelenmiş görünümlerini sağlar:

- Etiketlere, klasörlere, özelliklere göre filtreleyin
- Ada, tarihe veya özel özelliklere göre sıralayın
- Hızlı erişim için lens yapılandırmalarını kaydedin

---

## 16. Ayarlar

Kenar çubuğundaki dişli simgesinden veya `Ctrl+,` ile ayarlara erişin.

### Genel

- Dil (15 dil)
- Tema (Açık / Koyu)
- Arayüz yazı tipi, Metin yazı tipi, Mono yazı tipi, Yazı tipi boyutu
- Yazı tipi teması — hazır yazı tipi kombinasyonları (Daktilo, Klasik, Modern vb.) hızlı geçiş için

### Düzenleyici

- Düzenleyici türü (Markdown / Belge)
- Varsayılan görünüm (Okuma / Düzenleme)
- Canlı Önizleme modu
- Satır numaraları, Girinti kılavuzları, Yazım denetimi
- Otomatik parantez eşleştirme, Akıllı listeler

### Kütüphaneler

- Kütüphane ekleme/kaldırma
- Kütüphane bazında görünüm ayarları
- Ek dosya klasörü konumu

### Güncellemeler

- Güncellemeleri kontrol et
- Özel depo güncellemeleri için GitHub jetonu

---

## 17. Klavye Kısayolları

### Genel

| Kısayol | İşlem |
|---------|-------|
| `Ctrl+N` | Yeni not |
| `Ctrl+O` | Star Jump (hızlı açma) |
| `Ctrl+P` | Mission Control |
| `Ctrl+G` | Sky View'ı aç |
| `Ctrl+,` | Ayarlar |
| `Ctrl+Shift+F` | Kütüphanede ara |
| `Ctrl+Shift+N` | İkinci ekran |

### Düzenleyici

| Kısayol | İşlem |
|---------|-------|
| `Ctrl+B` | Kalın |
| `Ctrl+I` | İtalik |
| `Ctrl+K` | Wikilink ekle |
| `Ctrl+Z` | Geri al |
| `Ctrl+Shift+Z` | Yinele |
| `Ctrl+D` | Sonraki eşleşmeyi seç |
| `Ctrl+/` | Yorumu aç/kapat |
| `Tab` | Girinti / sonraki tablo hücresi |

### Sky View

| Kısayol | İşlem |
|---------|-------|
| `Ctrl+F` | Ara ve vurgula |
| `Ctrl+L` | Düzen modunu değiştir |
| `Space` | Odak modunu aç/kapat |
| `0` | 3D döndürmeyi sıfırla |
| `W/A/S/D/Q/E` | 3D'de uç |
| `Escape` | Sky View'ı kapat |

---

## 18. RTL ve Arapça Desteği

Constellation, Arapça, İbranice, Farsça, Urduca ve diğer RTL yazı sistemleri için birinci sınıf destek sunar:

- **Otomatik algılama**: Not yönü içerikten otomatik olarak algılanır
- **Arayüz**: Arapça/İbranice dil seçildiğinde tam RTL arayüz
- **Düzenleyici**: Doğru imleç hareketi ve seçim ile RTL metin düzenleme
- **Sky View**: Arapça etiketler uygun yazı tipi geri dönüşü ile sağdan sola oluşturulur
- **Açıklama**: Öğeler, içerik diline göre nokta/metin sırasını çevirir
- **Yazı tipi betikleri**: Ayarlar'da Arapça, İbranice ve CJK yazı tiplerini bağımsız olarak yapılandırın

### Arapça İçin Kurulum

1. **Ayarlar > Genel > Dil** bölümüne gidin ve Arapça'yı seçin
2. İsteğe bağlı olarak **Ayarlar > Genel > Yazı tipi betikleri** bölümünde özel bir Arapça yazı tipi ayarlayın
3. Arapça içerikli notlar otomatik olarak RTL olarak görüntülenecektir

---

## 19. Güvenlik ve Gizlilik

- **Tüm veriler yerelde kalır** — bulut senkronizasyonu yok, telemetri yok, izleme yok
- **Markdown dosyaları** — notlarınız tamamen size ait düz metin dosyalarıdır
- **Hesap gerekmez** — Constellation tamamen çevrimdışı çalışır
- **İsteğe bağlı güncellemeler** — Ayarlar üzerinden güncellemeleri elle kontrol edin
- **Açık kaynak** — kodu [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation) adresinden inceleyin

---

## 20. Bilgi Haritası

Bilgi Haritası, tum bilgi evreninizin yapisini, yogunlugunu ve olgunlugunu gosteren radyal bir sunburst gorsellestirmesidir.

### Haritayi Acma

- **Dock dugmesi**: Sol dokta Bilgi Haritasi simgesine tiklayin
- **Komut Paleti**: `Ctrl+P` ardindan "Constellation Map" yazin

### Ne Gorursunuz

- **Merkez**: Evren adiniz, toplam not ve kelime sayisiyla
- **Ilk halka**: Kutuphaneler (her biri kendi rengiyle). Evreninizde alt evrenler varsa, burada da gorunurler.
- **Derin halkalar**: Her kutuphanedeki klasorler ve alt klasorler
- **En dis segmentler**: Bireysel notlar

### Renk Modlari

Acilir menuyle uc mod arasinda gecis yapin:
- **Olgunluk**: tohum (gri) → fide (acik yesil) → her dem yesil (yesil) → kanonik (altin) → solmus
- **Katman**: L1 (mavi) → L8 (kirmizi) — bilgi karmasikligini gosterir
- **Kutuphane**: tum segmentler ust kutuphanenin rengini devralir

### Derinlesme Gezintisi

Yakinlastirmak icin herhangi bir klasor segmentine tiklayin. Icerik yolu izinizi gosterir. Geri donmek icin herhangi bir yol ogesine tiklayin veya Escape'e basin. Editorde acmak icin not segmentine tiklayin.

### Haritaya Don

Haritadan bir not actiktan sonra, sekme cubugunda "Haritaya Don" dugmesi gorunur. Tam olarak bulundugunuz yere donmek icin tiklayin — ayni derinlesme seviyesi korunur.

---

## 21. Bilişsel Motor

Bilişsel Motor, Constellation'ın notlarınızı analiz eden ve fikirleriniz arasındaki gizli kalıpları ve ilişkileri ortaya çıkaran yerleşik zeka sistemidir. Temel felsefesi:

> "Verilerinizin miktarı önemli değil. Önemli olan kaç kaynak sakladığınız değil, onlardan bilginizi nasıl şekillendirdiğiniz ve anlamlı tek bir farkındalıkta nasıl birleştirdiğinizdir."

Bilişsel Motor dokuz entegre araçtan oluşur: Tipli bağlantılar, Bilgi katmanları, Olgunluk döngüsü, Gerilim algılayıcı, Köken zinciri, Dışsallaştırma motoru, Gözden Geçirme Nabzı, Yollar ve Çoklu Lens Görünümleri.

---

### 17.1 Tipli bağlantılar

#### Nedir?

Tipli bağlantılar, iki not arasındaki ilişkinin türünü tanımlayan wiki bağlantılarıdır. Sadece `[[not]]` yazmak yerine, `[[not|ilişki-türü]]` yazarak bağlantının doğasını ifade edersiniz — ondan mı türetilmiş? Onu mu çürütüyor? Onu mu genişletiyor?

#### Neden önemli?

Normal bir bağlantı yalnızca "bir bağlantı var" der ama ne tür olduğunu söylemez. Tipli bağlantılar, not ağınızı referans yığınından düşünce yapılarını, bağımlılıkları ve çıkarımları görünür kılan gerçek bir bilgi haritasına dönüştürür.

#### Nasıl kullanılır

1. Düzenleyicide bir not açın
2. İlişki türü ile wiki bağlantısı yazın: `[[Hedef not|derives-from]]`
3. Desteklenen türler: `derives-from` (türetilmiş), `supports` (destekler), `contradicts` (çelişir), `extends` (genişletir), `exemplifies` (örneklendirir), `questions` (sorgular)
4. Sağ kenar çubuğundaki not özelliklerinden de tür ekleyebilirsiniz

#### Nerede görülür?

- **Yıldız Görünümü (GraphMind)**: Düğümler arasında renkli ve etiketli çizgiler olarak
- **Sağ kenar çubuğu**: "Geri bağlantılar" sekmesinde her bağlantının türü ile
- **Köken sekmesi**: Bilgi soy ağacı oluşturmak için kullanılır

---

### 17.2 Bilgi katmanları

#### Nedir?

Bilişsel Motor her notu otomatik olarak sekiz bilgi katmanından birine sınıflandırır: Anlık Görüntü, Günlük, Konu, Harita, Çerçeve, İlke, İnanç, Eser. Sınıflandırma notun yapısına, içeriğine ve bağlantı sayısına dayanır.

#### Neden önemli?

Her notun türünü bilmek kütüphanenizdeki bilgi dengesini ortaya koyar. Notlarınızın çoğu gelip geçici anlık görüntüler mi yoksa ilkelere ve çerçevelere mi evrilmiş? İçeriğin doğasına dair bu farkındalık, yalnızca bilgi biriktirmek yerine gerçek bilgi inşa etmenin ilk adımıdır.

#### Nasıl kullanılır

1. Sınıflandırma otomatik olarak yapılır — herhangi bir işlem gerekmez
2. Otomatik sınıflandırmayı geçersiz kılmak için frontmatter'a `stratum` özelliğini ekleyin:
   ```yaml
   ---
   stratum: framework
   ---
   ```
3. Kullanılabilir değerler: `snapshot`, `log`, `topic`, `map`, `framework`, `principle`, `conviction`, `artifact`

#### Nerede görülür?

- **Sağ kenar çubuğu**: Not özellikleri bölümünde "Katman" altında
- **Yıldız Görünümü**: Katmana göre farklı düğüm renkleri olarak
- **Ayarlar > Bilişsel Motor**: Otomatik sınıflandırmayı etkinleştirme/devre dışı bırakma

---

### 17.3 Olgunluk döngüsü

#### Nedir?

Motor her notun olgunluk seviyesini dört aşamada takip eder: **Tohum** → **Fidan** → **Yaprak Dökmeyen** → **Kanonik**. Her not tohum olarak başlar ve içerik, bağlantı ve gözden geçirme eklendikçe kademeli olarak büyür.

#### Neden önemli?

Olgunluk, ham bir fikir ile rafine edilmiş bilgi arasındaki farkı ortaya koyar. Bugünün tohumu, yeterli ilgiyi gösterirseniz yarının referansı olabilir. Olgunluk takibi, daha fazla geliştirme ve ilgiyi hak eden notları belirlemenize yardımcı olur.

#### Nasıl kullanılır

1. Olgunluk şunlara göre otomatik olarak değişir: kelime sayısı, gelen ve giden bağlantı sayısı, son değişiklik tarihi
2. Olgunluğu elle ayarlamak için frontmatter'a `maturity` özelliğini ekleyin:
   ```yaml
   ---
   maturity: evergreen
   ---
   ```
3. Kullanılabilir değerler: `seed` (Tohum), `sapling` (Fidan), `evergreen` (Yaprak Dökmeyen), `canonical` (Kanonik)

#### Nerede görülür?

- **Sağ kenar çubuğu**: Başlığın yanındaki simge mevcut olgunluk aşamasını gösterir
- **Yıldız Görünümü**: Düğüm boyutu olarak — not ne kadar olgunsa düğüm o kadar büyük
- **Ayarlar > Bilişsel Motor**: Olgunluk takibini etkinleştirme/devre dışı bırakma

---

### 17.4 Gerilim algılayıcı

#### Nedir?

Gerilim algılayıcı bağlantılı notları inceler ve iki veya daha fazla not arasında iddialar veya sonuçlar çeliştiğinde sizi uyarır. `contradicts` türündeki tipli bağlantı analizine ve notlar arasındaki tematik benzerliğe dayanır.

#### Neden önemli?

Gerilimler mutlaka hata değildir — daha derin düşünmeye bir davettir. Kütüphanenizde iki fikir birbiriyle çeliştiğinde, bu anlayışınızın evrildiği veya keşfedilmeye değer bir karmaşıklığın var olduğu anlamına gelir. Gerilim algılama, bilinçsizce çelişkili temeller üzerine bilgi inşa etmenizi önler.

#### Nasıl kullanılır

1. Çelişen notlar arasına `contradicts` tipli bağlantı ekleyin: `[[Diğer not|contradicts]]`
2. Motor, içerik analizi yoluyla örtük gerilimleri de algılar
3. Kenar çubuğundan algılanan gerilimlerin listesini inceleyin

#### Nerede görülür?

- **Sağ kenar çubuğu**: Çelişkiler algılandığında "Gerilimler" sekmesinde
- **Yıldız Görünümü**: Çelişen düğümler arasında kırmızı noktalı çizgiler olarak
- **Bildirim paneli**: Yeni gerilim algılandığında uyarılar

---

### 17.5 Köken zinciri

#### Nedir?

Köken zinciri her fikrin kökenini izler — nereden geldiğini ve neyden türetildiğini. `[[not|derives-from]]` bağlantılarını kullanarak orijinal kaynaktan mevcut formülasyona kadar bilginin gelişim yolunu gösteren bir soy ağacı oluşturur.

#### Neden önemli?

Fikirlerinizin nereden geldiğini bilmek, alınan bilgiyi (kitaplar, makaleler, konferanslardan) keşfedilen bilgiden (kendi çıkarımlarınız ve düşünceleriniz) ayırır. Bilginin kaynağına dair bu farkındalık, fikirlerinizin güvenilirliğini değerlendirmenize ve düşüncenizin zaman içinde nasıl şekillendiğini anlamanıza yardımcı olur.

#### Nasıl kullanılır

1. Bir kaynaktan türetilen not oluştururken bağlantı ekleyin: `[[Orijinal kaynak|derives-from]]`
2. Çok seviyeli zincirler oluşturulabilir: not ← türetilmiş ← türetilmiş ← orijinal kaynak
3. Dış kaynakları frontmatter'a `source-type: received` ekleyerek sınıflandırın

#### Nerede görülür?

- **Sağ kenar çubuğu**: "Köken" sekmesi tam soy ağacını gösterir
- **Yıldız Görünümü**: Bağlantılardaki ok yönleri olarak (kaynaktan türetilene)
- **Not özellikleri**: Köken zincirine göre "alınmış" veya "keşfedilmiş" sınıflandırması

### 17.6 Dışsallaştırma motoru

#### Nedir?

Notlarınızın ham yakalamadardan kristalleşmiş içgörülere nasıl olgunlaştığını izleyen aşamalı bir biçimlendirme hattıdır. Her nota dört aşamadan biri atanabilir:

| Aşama | Simge | Anlam |
|-------|-------|-------|
| Geçici | 🌱 | Hızlı yakalama, geçici düşünce |
| Literatür | 📖 | Bir kaynaktan kendi sözlerinizle yeniden yazılmış |
| Kalıcı | 🔗 | Atomik fikir, tek kavram, grafiğinize bağlı |
| Sentez | ✨ | Birden fazla kalıcı notu birleştiren özgün içgörü |

#### Neden önemli?

Çoğu uygulama tüm notlara eşit davranır. Dışsallaştırma motoru bu farkı görünür kılar — kütüphanenizin ne kadarının ham yakalama, ne kadarının gerçek anlayış olduğunu bir bakışta görebilirsiniz.

#### Nasıl kullanılır

1. İçerik haritası çubuğunda (editörün üstünde), aşama açılır menüsünü kullanarak bir aşama seçin.
2. Veya Özellikleri genişletin ve oradaki aşama açılır menüsünü kullanın. Her ikisi de dosya ağacıyla anında senkronize olur.
3. Bir notu terfi ettirmek için açılır menüyü bir aşamadan diğerine değiştirin. Odak modunda, altta "Kalıcıya Terfi Et"e tıklayın.
4. Bir aşamayı kaldırmak için açılır menüden "— Aşama —" seçin.

#### Nerede görülür?

- **İçerik haritası çubuğu**: emoji + aşama adı içeren açılır menü
- **Özellikler paneli**: `stage` özelliği mevcut olduğunda açılır menü
- **Dosya ağacı**: not adının yanında emoji simgesi
- **Odak modu alt bilgisi**: "Kalıcıya Terfi Et" düğmesi

### 17.7 Gözden Geçirme Nabzı

#### Nedir?

Gözden Geçirme Nabzı, notları artan aralıklarla dikkatinize geri getiren aralıklı yeniden yüzeyleme sistemidir: son gözden geçirmeden 1 gün, sonra 3, sonra 7, sonra 14, sonra 30 gün sonra. Ayrıca `#assumption` veya `#model` etiketli notları zihinsel model kontrol noktaları olarak izler ve yakalanmış ancak hiç tekrar ziyaret edilmemiş notlar için bir "Hiç gözden geçirilmemiş" kuyruğu tutar.

#### Neden önemli?

Bilgi, tekrar ziyaret edilmeden zayıflar. Bugün bir not yazarsınız ve üç hafta sonra var olduğunu unutursunuz. Aralıklı tekrar, bilişsel bilimde bu zayıflamayla savaşmak için en yerleşik tekniktir. Gözden Geçirme Nabzı bu ilkeyi gerçek notlarınıza uygular.

#### Nasıl kullanılır

1. Sol kenar çubuğunda **Gözden Geçirme Nabzı** sekmesine tıklayın. Üç bölüm göreceksiniz: Gözden geçirilmesi gereken, Zihinsel model kontrol noktaları (`#assumption` / `#model`), ve Hiç gözden geçirilmemiş.
2. Herhangi bir nota tıklayarak açın ve okuyun.
3. Üç eylemden birini seçin:
   - **Gözden geçirildi** (onay işareti) — bir sonraki aralıkta bir sonraki gözden geçirmeyi planlar (1 → 3 → 7 → 14 → 30 gün).
   - **7 gün ertele** (göz simgesi) — aralığı ilerletmeden notu 7 gün erteler.
   - **Kaldır** (arşiv simgesi) — notu gözden geçirme kuyruğundan kalıcı olarak kaldırır.
4. Komut Paletini açın ve "Review due notes" yazın.

#### Nerede görülür?

- **Sol kenar çubuğu**: Bekleyen not sayısını gösteren rozetli Gözden Geçirme Nabzı sekmesi
- **Komut Paleti**: Hızlı erişim için "Review due notes" komutu

### 17.8 Yollar

#### Nedir?

Yollar, notların adlandırılmış ve sıralı dizileridir — bir kitabın bölümleri veya bilginiz boyunca rehberli bir turun durakları gibi. Bir notun frontmatter'ına `trail: true` eklenerek ve not gövdesinde wikilinklerin sırayla listelenmesiyle tanımlanır.

#### Neden önemli?

Bilgi her zaman bir ağ değildir. Bazen bir yoldur — bir öğrenme dizisi, argüman ilerlemesi, bir anlatı. Yollar bu sırayı açıkça yakalar ve doğrusal olmayan kütüphanenize doğrusal bir boyut ekler.

#### Nasıl kullanılır

1. Frontmatter'da `trail: true` olan yeni bir not oluşturun.
2. Not gövdesinde, wikilinkleri istenen sırada listeleyin.
3. Bir yola ait bir notu açtığınızda, içerik haritası çubuğu yol adını ve konumu gösteren bir gösterge gösterir (ör. "Yolum 2/5"). Ok düğmeleri önceki ve sonraki nota gitmenizi sağlar.
4. Komut Paletini açın ve "Open Trail" yazarak tüm yolları görüntüleyin.

#### Nerede görülür?

- **İçerik haritası çubuğu**: Ad, konum ve gezinme okları içeren yol göstergesi
- **Komut Paleti**: Tüm yolları listeleyen "Open Trail" komutu

### 17.9 Çoklu Lens Görünümleri

#### Nedir?

Çoklu Lens Görünümleri, klasör yapısını değiştirmeden veya notları çoğaltmadan kütüphanenizi farklı sınıflandırma şemaları aracılığıyla görüntülemenizi sağlar. Bir "lens", notları bir özelliğe veya etikete göre yeniden düzenleyen sanal bir gruplamadır. Yerleşik lensler: "Aşamaya göre" (Geçici/Literatür/Kalıcı/Sentez) ve "Konuya göre" (etiketlere göre gruplama). Ayarlarda özel lensler oluşturulabilir.

#### Neden önemli?

Klasör yapıları tek bir hiyerarşi dayatır, ancak bilgi tek bir ağaca sığmaz. Çoklu Lens Görünümleri, dosyaları taşımadan farklı bakış açıları arasında geçiş yapmanızı sağlar. Aynı notlar, farklı organizasyonel lenslerle görüntülenir.

#### Nasıl kullanılır

1. Kenar çubuğunda, dosya ağacının üstündeki **lens açılır menüsünü** bulun (varsayılan "Klasörler").
2. Bir lens seçin: "Aşamaya göre", "Konuya göre" veya özel bir lens. Kenar çubuğu anında yeniden düzenlenir.
3. Varsayılan dosya ağacına dönmek için "Klasörler"i seçin.
4. Özel lens oluşturmak için: **Ayarlar > Bilgi Yönetimi** açın, **Lens Oluştur** tıklayın, adlandırın ve gruplama için frontmatter özelliğini seçin.
5. Veya Komut Paletini kullanın: "Create Lens" yazın.

#### Nerede görülür?

- **Kenar çubuğu açılır menüsü**: Dosya ağacının üstünde lens seçici
- **Ayarlar > Bilgi Yönetimi**: Özel lensleri oluşturun, düzenleyin ve silin
- **Komut Paleti**: "Create Lens" komutu

### Bilişsel Motor Ayarları

Bilişsel Motor'un tüm araçları **Ayarlar > Bilişsel Motor** bölümünden yapılandırılabilir:

- **Katman sınıflandırması** — Otomatik sınıflandırmayı etkinleştir veya devre dışı bırak
- **Olgunluk takibi** — Olgunluk yaşam döngüsü takibini etkinleştir veya devre dışı bırak
- **Tipli bağlantılar** — Bağlantı algılama hassasiyet eşiğini ayarla (0.0 – 1.0)
- **Gerilim algılayıcı** — Otomatik gerilim algılamayı etkinleştir veya devre dışı bırak
- **Elle geçersiz kılma** — Otomatik sınıflandırmayı geçersiz kılmak için frontmatter'a `stratum` ve `maturity` özelliklerini ekleyin

---

*Constellation Kullanım Kılavuzu — Sürüm 0.3.4 — Mart 2026*
*uconstellation.world*
