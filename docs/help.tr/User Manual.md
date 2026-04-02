# Constellation Kullanım Kılavuzu

**Sürüm 0.3.4 | Mart 2026**

Constellation, Markdown not kütüphanelerini yönetmek için tasarlanmış bir Kişisel Bilgi Yönetimi (PKM) masaüstü uygulamasıdır. Tauri v2, SvelteKit ve Rust ile geliştirilmiş olup Windows, macOS ve Linux'ta tam Arapça ve RTL desteğiyle yerel olarak çalışır.

---

## İçindekiler

1. [Başlarken](#başlarken)
2. [Evren ve Kütüphaneler](#evren-ve-kütüphaneler)
3. [Not Oluşturma ve Düzenleme](#not-oluşturma-ve-düzenleme)
4. [Star View (GraphMind)](#star-view-graphmind)
5. [İkinci Ekran](#ikinci-ekran)
6. [Özellikler ve Frontmatter](#özellikler-ve-frontmatter)
7. [Şablonlar](#şablonlar)
8. [Tablolar](#tablolar)
9. [Görevler](#görevler)
10. [İçe Aktarıcı](#içe-aktarıcı)
11. [Takvim](#takvim)
12. [Lens](#lens)
13. [Ayarlar](#ayarlar)
14. [Klavye Kısayolları](#klavye-kısayolları)
15. [RTL ve Arapça Desteği](#rtl-ve-arapça-desteği)
16. [Güvenlik ve Gizlilik](#güvenlik-ve-gizlilik)
17. [Bilişsel Motor](#bilişsel-motor)

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
| **Kenar Çubuğu (Şerit)** | Gezinme düğmeleri: Dosya ağacı, Arama, Star View, Takvim, Şablonlar, Ayarlar |
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

Evrenleri evrenlerin içine yerleştirebilirsiniz. Bir **Alt Evren**, ana evreniniz tarafından başvurulan başka bir evren klasörüdür. Alt evrenlerdeki notlar Star View'da kendi notlarınızla birlikte görünür ve kütüphaneler arası bağlantılar kesikli çizgiler olarak gösterilir.

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

## 4. Star View (GraphMind)

Star View, notlarınızı **GraphMind** motoru (Pixi.js WebGL) tarafından desteklenen etkileşimli bir 3D grafik olarak görselleştirir.

### Star View'ı Açma

- Kenar çubuğundaki grafik simgesine tıklayın
- `Ctrl+G` tuşuna basın
- Mission Control (`Ctrl+P`) > "Star View"

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

Star View, notlarınızı soyutlama düzeyine göre otomatik olarak sekiz bilgi katmanına sınıflandırır:

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

## 5. İkinci Ekran

Yan yana not görüntüleme için ayrı bir pencere açın.

- **Açma**: Kenar çubuğundaki ikinci ekran simgesine tıklayın veya `Ctrl+Shift+N`
- **Senkronizasyon**: Notlar ikinci ekranda bağımsız olarak açılır. Yazı tipi ve tema ayarları her iki pencereye de uygulanır.
- **Not genişliği**: Araç çubuğundaki genişlik kaydırıcısı ile ayarlanabilir

---

## 6. Özellikler ve Frontmatter

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

## 7. Şablonlar

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

## 8. Tablolar

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

## 9. Görevler

Constellation, notlarda görev onay kutularını destekler:

```markdown
- [ ] Tamamlanmamış görev
- [x] Tamamlanmış görev
```

Canlı Önizleme modunda, onay kutuları tıklanabilirdir. Görevler, kütüphaneleriniz genelinde aranabilir ve filtrelenebilir.

---

## 10. İçe Aktarıcı

Diğer PKM araçlarından notları içe aktarın:

- **Obsidian** — tam wikilink uyumluluğuyla kasaları içe aktarır
- **Markdown klasörleri** — `.md` dosyaları içeren herhangi bir klasörü içe aktarın
- **Diğer biçimler** — HTML, metin dosyaları

İçe aktarmayı başlatmak için **Ayarlar > İçe Aktarıcı** bölümüne gidin.

---

## 11. Takvim

Takvim görünümü, notları tarihe göre düzenlenmiş olarak gösterir:

- `date` özelliği olan notlar ilgili günlerinde görünür
- Herhangi bir tarih için günlük notlar oluşturulabilir
- Ok düğmeleriyle aylar arasında gezinin

Takvimi kenar çubuğundan açın.

---

## 12. Lens

Lens, notlarınızın filtrelenmiş görünümlerini sağlar:

- Etiketlere, klasörlere, özelliklere göre filtreleyin
- Ada, tarihe veya özel özelliklere göre sıralayın
- Hızlı erişim için lens yapılandırmalarını kaydedin

---

## 13. Ayarlar

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

## 14. Klavye Kısayolları

### Genel

| Kısayol | İşlem |
|---------|-------|
| `Ctrl+N` | Yeni not |
| `Ctrl+O` | Star Jump (hızlı açma) |
| `Ctrl+P` | Mission Control |
| `Ctrl+G` | Star View'ı aç |
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

### Star View

| Kısayol | İşlem |
|---------|-------|
| `Ctrl+F` | Ara ve vurgula |
| `Ctrl+L` | Düzen modunu değiştir |
| `Space` | Odak modunu aç/kapat |
| `0` | 3D döndürmeyi sıfırla |
| `W/A/S/D/Q/E` | 3D'de uç |
| `Escape` | Star View'ı kapat |

---

## 15. RTL ve Arapça Desteği

Constellation, Arapça, İbranice, Farsça, Urduca ve diğer RTL yazı sistemleri için birinci sınıf destek sunar:

- **Otomatik algılama**: Not yönü içerikten otomatik olarak algılanır
- **Arayüz**: Arapça/İbranice dil seçildiğinde tam RTL arayüz
- **Düzenleyici**: Doğru imleç hareketi ve seçim ile RTL metin düzenleme
- **Star View**: Arapça etiketler uygun yazı tipi geri dönüşü ile sağdan sola oluşturulur
- **Açıklama**: Öğeler, içerik diline göre nokta/metin sırasını çevirir
- **Yazı tipi betikleri**: Ayarlar'da Arapça, İbranice ve CJK yazı tiplerini bağımsız olarak yapılandırın

### Arapça İçin Kurulum

1. **Ayarlar > Genel > Dil** bölümüne gidin ve Arapça'yı seçin
2. İsteğe bağlı olarak **Ayarlar > Genel > Yazı tipi betikleri** bölümünde özel bir Arapça yazı tipi ayarlayın
3. Arapça içerikli notlar otomatik olarak RTL olarak görüntülenecektir

---

## 16. Güvenlik ve Gizlilik

- **Tüm veriler yerelde kalır** — bulut senkronizasyonu yok, telemetri yok, izleme yok
- **Markdown dosyaları** — notlarınız tamamen size ait düz metin dosyalarıdır
- **Hesap gerekmez** — Constellation tamamen çevrimdışı çalışır
- **İsteğe bağlı güncellemeler** — Ayarlar üzerinden güncellemeleri elle kontrol edin
- **Açık kaynak** — kodu [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation) adresinden inceleyin

---

## 17. Bilişsel Motor

Bilişsel Motor, Constellation'ın notlarınızı analiz eden ve fikirleriniz arasındaki gizli kalıpları ve ilişkileri ortaya çıkaran yerleşik zeka sistemidir. Temel felsefesi:

> "Verilerinizin miktarı önemli değil. Önemli olan kaç kaynak sakladığınız değil, onlardan bilginizi nasıl şekillendirdiğiniz ve anlamlı tek bir farkındalıkta nasıl birleştirdiğinizdir."

Bilişsel Motor altı entegre araçtan oluşur: Tipli bağlantılar, Bilgi katmanları, Olgunluk döngüsü, Gerilim algılayıcı, Köken zinciri ve Dışsallaştırma motoru.

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
