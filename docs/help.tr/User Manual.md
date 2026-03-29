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

*Constellation Kullanım Kılavuzu — Sürüm 0.3.4 — Mart 2026*
*uconstellation.world*
