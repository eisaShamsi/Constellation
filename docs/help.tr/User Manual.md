# Constellation Kullanım Kılavuzu

**Sürüm 0.1.0 | Mart 2026**

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
8. [Constellation Sight](#constellation-sight)
9. [İkinci Ekran](#ikinci-ekran)
10. [Özellikler ve Frontmatter](#özellikler-ve-frontmatter)
10b. [Kaynak Gözden Geçirme (CECE)](#10b-kaynak-gözden-geçirme-constellation-epistemic-content-engine--cece)
11. [Şablonlar](#şablonlar)
12. [Tablolar](#tablolar)
13. [Görevler](#görevler)
14. [İçe Aktarıcı](#içe-aktarıcı)
15. [Takvim](#takvim)
16. [Lens](#lens)
17. [Ayarlar](#ayarlar)
18. [Klavye Kısayolları](#klavye-kısayolları)
19. [RTL ve Arapça Desteği](#rtl-ve-arapça-desteği)
20. [Güvenlik ve Gizlilik](#güvenlik-ve-gizlilik)
21. [Bilgi Haritası](#bilgi-haritası)
22. [Bilişsel Motor](#bilişsel-motor)

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
| **Dosya Ağacı (Dosya Gezgini)** | Kütüphanelerinizdeki notları ve klasörleri tarayın — ada göre süzün, sekiz düzende sıralayın, birden fazla öğe seçin ve toplu işlemler uygulayın |
| **Düzenleyici** | Markdown notlarınızı okuyun ve düzenleyin |
| **Sekme Çubuğu** | Birden fazla notu sekmelerde açın |
| **Durum Çubuğu** | Kelime sayısı, karakter sayısı, okuma süresi |

---

### Dosya Gezgini (Dosya Ağacı)

Dosya Gezgini, notlarınızın *yaşadığı* yerdir — kütüphanelerinizin ve klasörlerinizin dosya sistemi görünümü. Bir notu açmak için adına tıklamanın yanı sıra, notlarınızı yönetmenin dört yolunu sunar; hepsi dosya sistemi üzerinde çalışır, notların içeriği üzerinde değil (içeriği aramak için Search Hub'ı kullanın).

**Süzme (ada göre).** Ağacın üstündeki süzme kutusuna yazmaya başlayın; ağaç anında, yalnızca *adı* aradığınız metinle eşleşen notlara ve klasörlere daralır (bir klasör eşleşirse, alt öğeleri korunur). Süzme yalnızca adlar üzerinde çalışır — not içeriğinde asla arama yapmaz (o Search Hub'ın işidir). Süzme **tüm kütüphanelerinizi** kapsar: daraltılmış bir kütüphane bile aranabilir — bir süzme yazdığınızda Constellation her kütüphaneyi yükler ve eşleşmeleri ortaya çıkarır, süzmeyi temizlediğinizde de önceki genişletme durumunu geri yükler. Süzme kutusunu temizlemek için metni silin.

**Sıralama (sekiz düzen).** Ağacın üstündeki sıralama düğmesi ağacın sırasını değiştirir. Sekiz düzen arasında geçiş yapar: **Ad** (A→Z / Z→A), **Değiştirilme tarihi** (en yeni / en eski), **Oluşturulma tarihi** (en yeni / en eski) ve **Boyut** (en büyük / en küçük). Hangi düzeni seçerseniz seçin, **klasörler her zaman üstte** kalır.

**Çoklu seçim.** Bir notu veya klasörü seçime eklemek/çıkarmak için `Ctrl` (macOS'ta `⌘`) tuşunu basılı tutarak tıklayın. Bir aralığı seçmek için `Shift` tuşunu basılı tutarak tıklayın — ilk seçilen öğe ile en son tıklanan öğe arasındaki her şey seçilir. Düz bir tıklama, her zaman olduğu gibi notu açar. Seçili satırlar vurgulanır. Seçimi temizlemek için `Escape` tuşuna basın.

**Toplu işlem çubuğu.** En az bir öğe seçtiğinizde, ağacın üstünde seçilen öğe sayısını gösteren bir çubuk belirir ve üç eylem sunar: **Etiket ekle**, **Taşı** ve **Sil** (silme çöp kutusu destekli — geri alınabilir). Her eylem, tek bir not için kullandığınız aynı güvenli, korumalı işlemlerin üzerinden çalışır — bu yüzden toplu bir etiket ekleme, açık bir notu bozmaz ve toplu bir silme, çöp kutusuna gider. Bağlı bir alt evrenden gelen salt okunur notlar her toplu işlemden atlanır.

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

### Senkronizasyon ve Dış Değişiklikler

Constellation, **File Over App** ilkesini benimser — notlarınız diskte sade `.md` dosyalarıdır ve uygulama bunları değişikliklere karşı izler. Uygulama açıkken bir not Constellation'ın *dışından* gelir veya değişirse — başka bir cihazdan gelen bir Obsidian senkronizasyonu, bir `git pull`, bir bulut senkronizasyon aracı (iCloud / Syncthing / OneDrive) ya da bir kütüphane klasörüne bıraktığınız bir dosya — Constellation bunu yaklaşık bir saniye içinde, **yeniden başlatma gerekmeden**, **otomatik olarak** algılar:

- Not, **dosya ağacında** görünür.
- **Star Jump** (`Ctrl+O`), **Arama**, **Dizin**, **geri bağlantılar** ve kütüphanenin **not sayısı** üzerinden bulunabilir hâle gelir — hepsi kendiliğinden güncellenir.
- Bir klasörü uygulamanın dışından yeniden adlandırırsanız, içindeki notlar yeni konumda bulunabilir kalır ve eski kayıtlar temizlenir.
- Büyük bir toplu iş (çok sayıda notun bir `git pull` ile gelmesi ya da ilk senkronizasyon) arka planda dizinlenir — arama yetişirken yazmanız anlık kalır.

Herhangi bir şey yapmanıza gerek yoktur: Constellation, dosyalarınız diskte değiştikçe arama dizinini onlarla eşzamanlı tutar. *(Küçük bir ayrıntı: bir klasörü uygulamanın **dışından** yeniden adlandırmak, o notların gözden geçirme zamanlamasını ve bağlantı ağırlığı geçmişini sıfırlar — notun metnine ise dokunulmaz. Klasörleri Constellation'ın **içinden** yeniden adlandırmak her şeyi korur.)*

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

#### Bilgi kutularını özelleştirme — renkler, simgeler ve kendi türleriniz

Bilgi kutusu renkleri ve simgeleri sizindir, dilediğiniz gibi değiştirebilirsiniz ve kendi bilgi kutusu türlerinizi de oluşturabilirsiniz. Stil Ayarlayıcı'yı (dock'taki 🎨 düğmesi) açın, Düzenleyici kategorisini seçin, ardından Bilgi Kutuları'na tıklayın. Merkez, her bilgi kutusunun rengini, simgesini ve adını gösteren tek bir satır olduğu birleşik bir Bilgi Kutuları yöneticisi açar.

- Yerleşik bir bilgi kutusunu yeniden renklendirme. Satırındaki renk örneğine tıklayın. Kaydedilmiş renklerinizi içeren küçük bir palet açılır (uygulamak için birine tıklayın), ayrıca herhangi bir yeni renk için bir "Özel…" seçici bulunur — seçtiğiniz bir renk, bir sonraki sefer için paletinize de eklenir. Yerleşik türler için renk değişiklikleri, Stil Ayarlayıcı'da Sakla/Uygula düğmesine bastığınızda kaydedilir.
- Yerleşik bir bilgi kutusunun simgesini değiştirme. Satırındaki simgeye tıklayın. Emoji ve Simge Kütüphanesi açılır — herhangi bir emoji veya vektör simge seçin. Her yerde anında, o bilgi kutusunun renginde değişir. Yalnızca o simgeyi geri almanızı sağlayan küçük bir ↺ görünür.
- Yerleşikleri sıfırlama. Yöneticinin üst kısmındaki "↺ Bu öğeyi sıfırla" düğmesi, tüm yerleşik bilgi kutusu renklerini ve simgelerini varsayılanlarına döndürür. (Özel bilgi kutularınıza dokunulmaz — onları tek tek kaldırın.)
- Kendi bilgi kutusu türünüzü oluşturma. Ayırıcının altında Ekle satırı bulunur. Bir Ad (örn. Karar veya فكرة), bir Tetikleyici sözcük (yazacağınız [!sözcük] — Arapça dahil her dil çalışır) yazın, bir renk ve bir simge seçin ve Ekle'ye tıklayın. Artık herhangi bir notta > [!decision] (veya > [!فكرة]) yazmak bilgi kutunuzu oluşturur. Tetikleyiciden sonra bir başlık yazmazsanız, bilgi kutusu başlığı bilgi kutunuzun adını kalın olarak gösterir.
- Özel bir bilgi kutusunu düzenleme veya kaldırma. Satırındaki ✎ (adı/tetikleyiciyi düzenle) ve ✕ (kaldır) işaretlerini kullanın. Bir türü kaldırmak, notlarınızdaki [!…] metnine dokunmaz — türü yeniden oluşturana kadar yalnızca düz not görünümüne döner.

Özel bilgi kutularınız, renkleriniz ve simgeleriniz bu Evren ile birlikte kaydedilir, böylece kütüphanenizle birlikte taşınır.

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

### Search Hub

Search Hub, tam ekran bir arama deneyimidir. Açmak için dock çubuğundaki büyüteç simgesine tıklayın. Maksimum alan sağlamak için her iki kenar çubuğu da kapanır. Herhangi bir terim yazın ve Constellation her yerde aynı anda arar, sonuçları 5 kategoriye gruplar: Başlıklar, İçerikler, Etiketler, Özellikler ve Wikilink'ler. Her kategorinin sayı rozeti olan daraltılabilir bir bölümü vardır. Herhangi bir sonuca tıklayarak tüm geçişlerin vurgulandığı editörde açın. "Search Hub'a Dön" düğmesi görünerek yeniden aramadan geri dönmenizi sağlar.

### Bağlantı operatörleri

Constellation 6 bağlantı topolojisi arama operatörünü destekler:

| Söz dizimi | Ne bulur |
|------------|----------|
| `links to [[X]]` | X'e bağlantı veren notlar (geri bağlantılar) |
| `links from [[X]]` | X'in bağlantı verdiği notlar (giden bağlantılar) |
| `mutual [[X]]` | X'e bağlı ve X geri bağlantı veren notlar (çift yönlü) |
| `mentions [[X]]` | [[wikilink]] olmadan X'in adını içeren notlar |
| `orphans` | Gelen veya giden bağlantısı olmayan notlar |
| `links between [[X]] and [[Y]]` | Hem X hem de Y'ye bağlantı veren notlar |

Herhangi bir bağlantı operatörü yazarken, `[[` otomatik tamamlama evrendeki tüm notları gösterir. Bir not seçtikten sonra, başlık tamamlama için `#` veya bağlantı türü tamamlama için `|type:` yazın.

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

## 8. Constellation Sight

Constellation Sight, tum bilgi sisteminizi bir kutle cekimi kuyusu grafigi olarak gorsellestirir. Su soruya cevap verir: **"Bilgim nasil gorunuyor ve ne kadar saglikli?"**

### Sight'i Acma

Sol seritte **Sight dugmesine** (goz simgesi) tiklayin. Kutle cekimi kuyusu grafigi goruntulenir. Kapatmak icin x'e tiklayin.

### Kutle Cekimi Kuyusu Grafigi

Notlar onem (merkezilik) derecesine gore es merkezli halkalar seklinde duzenlenir. En cok baglantili notlar merkezde; cevresel notlar kenarlarda yer alir. Her halka icinde notlar kutuphaneye (organizasyonunuza) gore gruplanir. Dugum rengi = kutuphane.

| Oge | Anlami |
|-----|--------|
| **Buyuk dugum** | Yuksek merkezilik — farkli bilgi alanlarini birlestiren kopru |
| **Kucuk dugum** | Cevresel — tek bir alan icinde |
| **Dugum rengi** | Kutuphane uyeligi |
| **Duz cizgi** | Iki not arasindaki baglanti |
| **Yon oklari** | Baglanti yonunu gosteren kucuk oklar |
| **Cizgi kalinligi** | Guven duzeyi (kalin = yerlesik, ince = hipotez) |

### Etkilesim

- **Tek tiklama**: Dugumun komsulugunu vurgular (tum baglantili notlar). Diger her sey soluklaşır.
- **Cift tiklama**: Notu duzenleyicide acar.
- **Bos alana tiklama**: Vurgulamayi temizler.
- **Kaydirma**: yakinlastirma/uzaklastirma. **Surukle**: kaydir. **Ekrana Sigdir**: arac cubugu dugmesi.

### Sight'ta Arama

Buyutece tiklayin. Tum operatorleri destekler: `links to [[X]]`, `links from [[X]]`, `mutual [[X]]`, `orphans`, `supports [[X]]`, `contradicts [[X]]`, `#tag`, serbest metin ve anlamsal arama. Sonuclar yonsel renkler gosterir: yesil (gelen), kirmizi (giden).

### Analiz Paneli (SightPanel)

Izgara simgesine tiklayarak yan cubugu acin. Gosterir: Evren Saglik puani (0-100), not/baglanti/yetim sayaclari, baglanti turu ve guven cubuklari, en iyi 10 kopru ve Bilgi Icgörüleri (en guclu kanit, zayif temeller, gerilimler, durgunlar, en cok baglantili, bilgi boslukları).

### Ayarlar

Disli simgesi: baglanti cizgi kalinligini, opakligini ve ok boyutunu ayarlayin. Ayarlar oturumlar arasinda kalici olarak saklanir.

### 8a. Not basina gelenek alanlari (MIG-029)

Sight'in sol ust kosesindeki gelenek cipi, 10 epistemik aile icindeki 24 akademik gelenek araciligiyla kubbeyi yeniden cerceveletmenize olanak tanir. Bu geleneklerden dokuzu icin (sektoryel / es merkezli / merdiven sekilli olanlar), her not frontmatter alani araciligiyla **acikca siniflandirilabilir**. Alan icermeyen notlar, gelenege ozgu makul bir varsayilan kovaya duser; alani OLAN notlar ise sizin adlandirdiginiz kovaya duser.

Notun YAML frontmatter'ina alani ekleyin:

```yaml
---
masadir_source: sunnah
---
```

O gelenegin cipine gecin → notunuz varsayilanin yerine adlandirilmis sektorune duser.

**Izin verilen alanlar ve degerler:**

| Gelenek | frontmatter alani | Izin verilen degerler | Yoklugunda varsayilan |
|---|---|---|---|
| **masādir (mesâdir)** (Sunni uṣūl al-fiqh) | `masadir_source` | `quran` / `sunnah` / `ijma` / `qiyas` | `quran` |
| **pramāṇa** (Hint Nyāya) | `pramana_kind` | `pratyaksha` / `anumana` / `upamana` / `shabda` | `pratyaksha` |
| **İbn Rüşd burhân** | `burhan_kind` | `burhan` / `jadal` / `khataba` / `shir` | `shir` (en dis halka) |
| **PaRDeS** (Yahudi yorumbilim) | `pardes_level` | `peshat` / `remez` / `derash` / `sod` | `peshat` |
| **Peirce** (3 faneroskopik kategori) | `peirce_category` | `firstness` / `secondness` / `thirdness` | `firstness` |
| **Habermas** (3 bilgi ilgisi) | `habermas_interest` | `technical` / `practical` / `emancipatory` | `technical` |
| **Mensiyusçu filizler** (4 ahlaki filiz) | `mencian_sprout` | `ceyin` / `xiuwu` / `cirang` / `shifei` | `ceyin` |
| **Mohist sān biǎo** (3 standart) | `mohist_zone` | `ben` / `yuan` / `yong` | hash ile 3 bolgeye dagilmis |
| **Kore Sŏngnihak** (Dort-Yedi tartismasi) | `songnihak_cell` | `li-sa` / `li-chil` / `qi-chil` / `qi-sa` | `li-sa` |

**Davranis:**
- Gelenek tarafindan taninmayan bir deger yazarsaniz (yazim hatasi veya uydurma), not varsayilan kovaya duser. Cokme yok, render hatasi yok.
- Frontmatter degisiklikleri otomatik olarak yayilir — notu kaydedin → kubbenin bir sonraki render'i degisikligi yansitir.
- Ayni alan sadece kendi adini tasiyan gelenek tarafindan okunur. Bir nota `masadir_source: sunnah` ayarlamak, PaRDeS veya Peirce'e gectiginizde hicbir etki yapmaz — her gelenek kendi alanini bagimsiz olarak okur.
- Bu, kubbenin uzamsal gramerini kontrol etmenin en acik yoludur. Bu alanlar olmadan, geometri dogrudur ama her not ayni varsayilan kovaya duser; bu alanlarla, cip analitik olarak anlamli hale gelir.

**Not basina alani olmayan gelenekler** (su anda diger araclarla tum yildizlari kovaliyor — klasor / kitaplik / hash):

- Aristotelesçi (varsayilan, yeniden eslestirme yok)
- Polanyi (gradyan sis; sektorlestirme yok)
- Husserl, Longino, Şâtıbî maqāṣid, Maimonides nübüvveti, Talmudik 13 middot, Wang Yangming, Mignolo plüriversal, Dussel transmodernite, Maldonado-Torres, Akan Wiredu, İbn Haldun ʿumrân, Ibuanyidanda

(Gelecekteki gocler, kullanici talebi belirginlestikce bu gelenekler icin de not basina frontmatter alanlari ekleyebilir.)

---

## 9. İkinci Ekran

İkinci Ekran, geçerli kenar çubuğu modunuza uyum sağlayan mod tabanlı bir eşlik penceresidir.

- **Açma**: Kenar çubuğundaki ikinci ekran simgesine tıklayın veya `Ctrl+Shift+2`
- **Otomatik kapanma**: Ana pencereyi kapattığınızda ikinci ekran otomatik olarak kapanır

### Mod Tabanlı Eşlik

İkinci ekran, ana penceredeki aktif kenar çubuğu moduna göre içeriğini değiştirir:

| Kenar Çubuğu Modu | İkinci Ekran Gösterir |
|---|---|
| **Dosya Gezgini** | Evren Paneli — istatistikler, kütüphane dağılımı, alt evrenler, etiketler, son düzenlenen/açılan notlar |
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

## 10. Özellikler ve Frontmatter

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

## 10b. Kaynak Gözden Geçirme (Constellation Epistemic Content Engine — CECE)

> *(Çeviri notu: V3-§10.F bölümünün yapay zekâ ile üretilmiş çevirisi; ana dili konuşan kişi tarafından gözden geçirme bekliyor.)*

En önemli iki frontmatter özelliği — `sources:` ve `content_type:` — bir şeyi *nasıl öğrendiğinizi* ve *ne tür bir bilgi* olduğunu tanımlar. Constellation'ın **Epistemic Content Engine** (CECE), 6 kataloglayıcıdan oluşan bir topluluk kullanarak her notu bu iki eksen boyunca otomatik olarak sınıflandırır. **Kaynak Gözden Geçirme** paneli, bu sınıflandırmaları gözden geçirip düzelttiğiniz yerdir.

### Motorun yaptığı iş

Bir notu sınıflandırdığınızda (sağ tık → «Kaynak ve içerik türü öner», ya da Ayarlar > Tarama çalıştır üzerinden, ya da otomatik olarak arka plan tarama anahtarı üzerinden), CECE altı bağımsız kataloglayıcıyı nota karşı çalıştırır. Her biri notu farklı bir mercekten okur ve iki soruda oy kullanır:

- **Kaynak** (yatay eksen) — bu bilgi *nereden* geldi? On bir olası değer: algı, çıkarım, tanıklık, kütlesel-aktarım, karşılaştırma, postülasyon, kavrayışsızlık, hafıza, doğuştan-eğilim, ilham, vahiy. Artı *sınıflandırılamaz*.
- **İçerik türü** (dikey eksen) — bu *ne tür* bir bilgi? Beş üst düzey dal: duyusal girdiler, sembolik varlıklar, anlamsal içerikler, epistemik durumlar, üst düzey yapılar.

İki eksen birbirinden bağımsızdır. «Ay'a iniş hakkında şüphem var» notu kaynak ekseninde tanıklık (birisi bunu bildirdi) + içerik türünde epistemik-durumlar/şüphe (sizin tutumunuz) olur.

Motor **cihazınızda** çalışır — hiçbir not Constellation'dan ayrılmaz.

### Altı kataloglayıcı

Her kataloglayıcı bir mercektir. Kaynak Gözden Geçirme kartı bunları her kartın sağ üst köşesinde altı küçük renkli nokta olarak gösterir:

- **Ön bilginiz (frontmatter)** (mavi) — zaten ayarladığınızı mutlak yetkiyle benimser
- **Atıflar ve yapı** (gül) — atıflar, blok alıntılar, teorem işaretleri, tanım ifadeleri
- **Kelime kökleri ve sözlük** (kehribar) — Arapça kök analizi + diller arası terim eşdeğerliği
- **Bağlı notlar** (turkuaz) — diğer sınıflandırılmış notlara tipli Living Links
- **Benzer notlar** (mor) — zaten sınıflandırdığınız notlara embedding-benzerliği
- **Yapay zekâ değerlendirmesi** (yeşil) — yerel bir LLM (Qwen3-4B; *henüz aktif değil*, gelecekteki bir sürüme ertelenmiş)

Dolu bir nokta o kataloglayıcının ses verdiği ve sentezle aynı fikirde olduğu anlamına gelir. Halkalı bir nokta ses verdiği ama muhalif kaldığı anlamına gelir. Kesik konturlu bir nokta sessiz kaldığı anlamına gelir (bu mercekte sinyal yok).

### Üç güven rejimi

Kataloglayıcılar oy kullandıktan sonra her eksen üç rejimden birine düşer:

- **Oybirliği** — ses veren her kataloglayıcı hemfikir oldu
- **Güçlü çoğunluk (bir muhalif)** — çoğunluk hemfikir oldu; bir muhalif adlandırıldı
- **Bölündü** — net bir çoğunluk yok; motor tahmin etmeyi reddediyor ve sizden seçim istiyor

Her eksen kendi rejimini bağımsız olarak alır — bir kart yatayda Oybirliği + dikeyde Bölündü olabilir vs.

### Sibling Disambiguation

Bir eksen Bölündüğünde, motor aday değerleri bir istem altında **çipler** olarak sunar: *«Nota en uygun olanı seçin.»* Bir çipe tıklayın → motor bu seçimi notun frontmatter'ına yazar ve kartı kuyruktan kaldırır. DİĞER eksen çözüldüyse (Oybirliği veya Güçlü çoğunluk), motor *aynı zamanda* o eksenin değerini de yazar — yalnızca biri Bölünmüşken bir tıklama her iki ekseni bitirir.

### Gerekçe izi

Her kartta bir *«▸ Bu sınıflandırma neden?»* anahtarı vardır. Genişlettiğinizde, ses veren her kataloglayıcı için bir satır gösterilir; gerekçe, öz raporlanmış güven ve dostane kural çipleri («Yüzey anahtar kelime eşleşmesi», «Arapça kök eşleşmesi (CAE)», «Tanım işareti» vb.) ile birlikte — bunlar her kataloglayıcının tetiklediği özel kurallardır.

**İlk 50 incelemeniz** sırasında iz, her kartta otomatik olarak genişler (bir *güven kalibrasyon dönemi*), böylece motora ne zaman güveneceğinize dair sezgi geliştirebilirsiniz. Bundan sonra Oybirliği kartlarında izler isteğe bağlıya katlanır. **Ayarlar > Zekâ > CECE > Gerekçe izi görünürlüğü** üzerinden istediğiniz zaman geçersiz kılın.

### Kuyruk bileşim filtresi

Sayım çubuğunun üzerinde, beş çip kuyruğu her kartın hangi tür karara ihtiyaç duyduğuna göre dilimler:

- **Tümü** — tam kuyruk
- **Her iki eksen kararınızı bekliyor** — her iki eksen Bölündü
- **Kaynak kararınızı bekliyor** — yatay Bölündü + dikey çözüldü
- **İçerik türü kararınızı bekliyor** — dikey Bölündü + yatay çözüldü
- **Kataloglayıcılar hemfikir** — hiçbir eksen Bölünmedi (damga adayları)

Her çip kova sayımını gösterir. Filtre bir oluşturma katmanı dilimleyicisidir — Tümünü Kabul Et matematiği, hangi filtre etkin olursa olsun, her zaman tam kuyruk üzerinde çalışır.

### Kart başına işlemler

- **Kabul Et** — motorun sentezini her iki eksen için birincil olarak yaz; kartı kaldır. Kataloglayıcı başına güvenilirliği günceller.
- **Düzenle** — her iki eksen için ağaç seçici aç; manuel olarak seç. Aynı güvenilirlik güncellemesi.
- **Reddet** — yazmadan kartı temizler.
- **Sibling Disambiguation çipi** — yalnızca Bölünmüş kartlarda.

### Kütüphane bazında kalibrasyon

**Ayarlar > Zekâ > CECE > Kütüphane bazında kalibrasyon**, etkin Kütüphanedeki her kataloglayıcının eksen başına doğruluğunu gösteren salt okunur bir tablo açar. Farklı Kütüphanelerin kataloglayıcı başına farklı doğrulukları vardır — Dilbilim Arapça ağırlıklı Kütüphanelerde başarılı olur, Grafik yoğun bağlantılı olanlarda başarılı olur. Sentez katmanı oyları ağırlıklandırmak için bu kalibrasyon verilerini kullanır.

Bir kataloglayıcının doğruluk oranının gösterilebilmesi için **20 düzeltme** gerekir. Bu eşiğin altında etiket *«(eşit)»* olarak okunur — kataloglayıcı yeterli veri birikene kadar eşit ağırlıklı oylarla katkıda bulunur.

### Arka plan sınıflandırması

Varsayılan olarak CECE notları yalnızca siz istediğinizde sınıflandırır (sağ tık veya Ayarlar tarama düğmesi). Otomatik sınıflandırmayı **Ayarlar > Zekâ > CECE > Arka plan sınıflandırması** üzerinden açabilirsiniz:

- **Not kaydında** — yazmayı bıraktıktan ~1,5 saniye sonra her notu sınıflandırır (mevcut gecikmeli kaydetmenin üzerine biner; asla tuş başına tetiklenmez; yazma anında kalır)
- **Uygulama başlangıcında** — her başlatmada bir kez sınıflandırılmamış notları tarar

### Sınıflandırıcı — tam pencere ana yer

Aynı kartlar, **sol dock'taki istiflenmiş kartlar simgesinden** açılan, **Sınıflandırıcı** adlı tam pencere bir görünümde de yaşar. Dar bir kenar çubuğu sekmesi yerine tüm pencere verilmiş aynı motor ve aynı kuyruktur — ve kenar çubuğu sekmesinin hiç sahip olmadığı iki kontrol ekler:

- **Notu sınıflandır…** — *herhangi bir* notu önce açmadan adıyla sınıflandırmanıza olanak tanıyan bir arama kutusu. Birkaç harf yazın, notu seçin ve kuyrukta yeni bir kart görünür.
- **Tüm özetleri oluştur** — bir özeti olmayan her not için not özetini (aşağıya bakın) arka planda, ilerleme durum çubuğunda olacak şekilde önceden hesaplar.

Bir **Taramayı başlat** düğmesi (Ayarlar'dakiyle aynı evren çapında tarama) ve canlı bir ilerleme şeridi başlığı tamamlar. Sınıflandırıcıyı **(×)** veya **Esc** ile kapatın. (*Notu sınıflandır…* arama kutusu açıkken, ilk **Esc** yalnızca o kutuyu kapatır.)

Adlandırmaya dair bir not: **Sınıflandırıcı** *odadır* (tam pencere görünümü); **kataloglayıcılar** ise motorun içinde her kart üzerinde oy kullanan *altı mercektir*. İkisini karıştırmayın.

### Not özetleri

Her kartın başlığının altında kısa bir **Özet** durur — notu açmadan sınıflandırabilmeniz için ne hakkında olduğunu söyleyen birkaç cümle. Constellation her zaman *sizin* yazdığınız bir özeti tercih eder ve yalnızca yazmadığınızda bir tane üretir:

1. Bir `summary:` / `description:` / `abstract:` / `excerpt:` **frontmatter alanı**, harfi harfine kullanılır.
2. Gövdedeki bir `> [!summary]` / `[!abstract]` / `[!tldr]` **çağrı kutusu**, harfi harfine kullanılır.
3. Aksi takdirde, **üretilmiş** bir özet — notun en merkezi üç cümlesi, çıkarılmış (asla icat edilmemiş) ve orijinal sırasıyla gösterilmiş.

Üretilmiş özetler **salt okunurdur** — Constellation bir tanesini notunuza asla geri yazmaz (File-Over-App) ve her şey **cihazınızda** hesaplanır. Bir özetin dosyada yaşamasını istiyorsanız, kendiniz bir tane yazın; Constellation o zaman sizinkini gösterir.

Daha derin ayrıntı için (her nokta durumu, her kural çipi, tıklama-tıklama yönergeleri) yardım sistemindeki **Kaynak Gözden Geçirme**, **Sınıflandırıcı** ve **Not Özetleri** konularına bakın.

---

## 10c. Epistemik Üst Veri

Bir notun bilgisinin nasıl elde edildiği, tutumu kimin üstlendiği, hangi disipline ait olduğu ve görüşünüzü en son ne zaman revize ettiğiniz hakkında daha zengin bilgileri kaydetmek için küçük bir isteğe bağlı frontmatter alanları kümesi. Boşluk analizine (`docs/epistemic-content-gap-analysis.md`) yanıt olarak MIG-022 §A'da eklendi.

Bu alanlar **tamamen isteğe bağlıdır**. Onlar olmadan notlar değişmeden çalışır.

### Hızlı referans

| Field | Type | Amaç |
|---|---|---|
| `held_by` | text | Bu kimin tutumu? (varsayılan `user`; `"al-Shāfiʿī"`, `"Ḥanafī"` vb. olabilir) |
| `domain` | list | Getirme için disiplin etiketleri (`[fiqh, ʿibādāt]`) |
| `function` | text | Bu not ne için (`reference` / `seed` / `actionable` / `shipped`) |
| `provenance_civilization` | text | Gelenek söz dağarcığı (`sunni-usuli` / `analytic-western` / `nyaya` vb.) |
| `updated_at` | date | Görüşünüzü en son bilinçli olarak revize ettiğiniz zaman (dosya sisteminin mtime'ından farklı) |
| `ikhtilāf` | list of objects | Yapılandırılmış akademik anlaşmazlık (`[{school, position}, ...]`) |
| `warrant` | text | Derece etiketi (Warrant Research iş akışı yayınlanana kadar ayrıştırılır ama atıl) |
| `warrant_notes` | text | Warrant derecesini destekleyen serbest metin (ayrıca atıl) |

### Properties panelinde nasıl görünürler

Her alan türe uygun editörle oluşturulur:
- Metin alanları → metin girişi
- `domain` → etiket listesi (eklemek için Enter, kaldırmak için ×)
- `updated_at` → tarih seçici
- **`ikhtilāf` → özel widget**: satır başına yan yana iki giriş (school + position) artı satır başına bir kaldırma düğmesi ve altta bir "Mezhep ekle" düğmesi. Widget, yapılandırılmış YAML'den okur ve oraya yazar, böylece gidiş-dönüşler her alanı korur.

### `supersedes` ne olacak?

`supersedes`, tek bir notun özelliği değil, *notlar arasındaki bir ilişkidir* (bu not daha önceki bir notun yerini alır). Constellation bunu YAML skaleri olarak değil, **türlü bağlantı** olarak ele alır:

```markdown
This replaces my earlier analysis: [[old-note-id|supersedes]]
```

Wikilink üzerindeki `|supersedes` soneki, onu `supersedes` türünde bir türlü bağlantı yapar — farklı bir kayrak mavi-gri hap, Backlinks + Outgoing Links panellerinde görünür, Living Link Architecture'a katılır.

### Bu NE DEĞİLDİR

Yeni alanlar bir **şemadır** — doldurabileceğiniz tanınmış bir söz dağarcığı. CECE bunları şu anda sınıflandırma için tüketmez. Gelecekteki MIG'ler (Warrant Research iş akışı, MIG-023 zamansal eksen), `warrant`, `updated_at` ve benzerlerini okuyan özellikler yayınlayacaktır.

Daha derin ayrıntı ve işlenmiş bir örnek için yardım sistemindeki **Epistemic Metadata** konusuna bakın.

---

## 11. Şablonlar

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

## 12. Tablolar

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

## 13. Görevler

Constellation, notlarda görev onay kutularını destekler:

```markdown
- [ ] Tamamlanmamış görev
- [x] Tamamlanmış görev
```

Canlı Önizleme modunda, onay kutuları tıklanabilirdir. Görevler, kütüphaneleriniz genelinde aranabilir ve filtrelenebilir.

---

## 14. İçe Aktarıcı

Diğer PKM araçlarından notları içe aktarın:

- **Obsidian** — tam wikilink uyumluluğuyla kasaları içe aktarır
- **Markdown klasörleri** — `.md` dosyaları içeren herhangi bir klasörü içe aktarın
- **Diğer biçimler** — HTML, metin dosyaları

İçe aktarmayı başlatmak için **Ayarlar > İçe Aktarıcı** bölümüne gidin.

---

## 15. Takvim

**Takvim**, **sol dock**'tan (takvim simgesi) açılan tam sayfa bir ay görünümüdür. Notu veya teslim tarihi olan görevi bulunan günler renkli **noktalar** ile işaretlenir. Başlık, ayı seçtiğiniz takvimde gösterir; bir **ikincil takvim** ayarladıysanız, altındaki alt başlık o takvimin denk gelen aralığını gösterir (örneğin bir Miladi ay, Hicri karşılığı olan "Zilhicce 1447 – Muharrem 1448 AH" aralığını gösterir).

**Bir güne tıklama.** Her gün hücresi etkileşimlidir:

- **Boş alana (veya gün numarasına) tıklayın** → o günün **günlük notunu** açar (veya oluşturur). Zaten bir günlük notu olan bir tarihe tıklamak yalnızca onu **açar** — asla bir kopya oluşturmaz.
- **Bir noktaya tıklayın** → o belirli öğeyi açar. **Altın** nokta günlük nottur; **mor** nokta o gün düzenlenen başka bir nottur; **kırmızı** nokta o gün teslim tarihi olan bir görevdir. (Renkler Style Setter → Takvim'den temalandırılabilir.) Bir günde birkaç not veya görev varsa, noktaya tıklamak seçim yapabileceğiniz küçük bir **liste** gösterir.
- **Bir görev noktasına tıklayın** → notu **o görevin satırına kaydırılmış** olarak, düzenlemeye hazır şekilde açar. Görev listesinde, doğrudan takvimden **bir görevin onay kutusunu işaretleyerek onu tamamlayabilirsiniz** — tamamlanan görevler anında listeden düşer. Takvimde yalnızca kendi `📅 YYYY-MM-DD` tarihini taşıyan görevler görünür (onları bir güne yerleştiren şey bu tarihtir).

**Kültürel takvimler (sekiz).** **Ayarlar → Takvim** bölümünde **takvim sistemini** ayarlayabilirsiniz — **Miladi, Hicri (İslami), Şemsi Hicri (İran/Farsça), İbrani, Hint (Saka), Budist, Çin veya Kore** — ve tüm ay ızgarası ona geçer; her hücrede hem seçilen takvim tarihini (büyük) hem de Miladi tarihi (küçük) ve ayrıca ay evresini gösterir. Her ay başlığı ayın **adını, parantez içinde numarasını ve yılı** gösterir — numara, ay sırası tanıdık olmayan takvimlerde yardımcı olur. **Çin ve Kore** takvimleri *ay-güneş* takvimleridir: bazen bir **artık ay** (闰六月 / 윤6월) eklerler; takvim bunu kendi sayfası olarak gösterir, böylece gezinme asla bir ayı atlamaz veya ikilemez. Hicri takvim, doğru bir astronomik motor kullanır; mübarek aylar vurgulanır ve İslami olaylar işaretlenir. Ayrıca **hafta başlangıcını** (Pazar/Pazartesi) seçebilir ve **hafta numarası sütununu** açıp kapatabilirsiniz.

**Hicri takvim seçenekleri.** **Ayarlar → Takvim → "Hicri takvim (İslami)"** altında iki ek denetim vardır:

- **Hesaplama yöntemi** — gerçek yeni ayı (hilâl kavuşumu) izleyen **Astronomik (Ay Kavuşumu)** (en doğru olan, varsayılan), ya da klasik aritmetik döngü olan **Tablo Bazlı (al-Tawfīqāt al-Ilhāmiyyah)** (İlham edilmiş tevfikler).
- **Ay düzeltmesi** — bir Hicri ayın başlangıcını **yerel hilâl gözlemine** uydurmak için ±1 veya ±2 gün kaydırın. Hicri yılı ve ayı seçin, bir kayma değeri belirleyin ve **Set** (Ayarla) düğmesine tıklayın; düzeltme o aya ve ondan sonraki her aya uygulanır. Düzeltmeleriniz listelenir (her biri kaldırılabilir) ve bir **Clear all** (Tümünü temizle) düğmesi bulunur.

Her iki ayar da (ve düzeltmeleriniz) **evreninizle birlikte** kaydedilir, böylece cihazlarınız arasında taşınırlar.

**Çin ve Kore görüntüleme seçenekleri.** Kore, Çin ay takvimini kullanır, dolayısıyla ikisi aynı tarihleri paylaşır — onları ayıran şey yazı ve yıldır. İkisinden biri ana veya ikincil takviminiz olduğunda, **Ayarlar → Takvim** iki ek denetim gösterir: bir **yıl gösterimi** (Çin: altmışlık döngü 丙午年, sade yıl veya her ikisi; Kore: **Dangi** dönemi 단기 4359, yıl veya altmışlık 병오년) ve **ay adları** — *yerel yazı* (五月 / 5월) veya *fonetik*, yani ayın telaffuzunun kendi dilinizde yazılmış hali (İngilizce "Wǔyuè / Owol"; Arapça "وُو-يوي / أوه-وُل").

**Takvimi biçimlendirme.** **Style Setter**'ı (sol dock veya **Ayarlar → Style Setter**) açın ve her parçayı yeniden biçimlendirmek için **Takvim** yüzeyini seçin — her öğenin kendi **rengi ve metin boyutu** vardır (gün numaraları, çapraz referans tarihi, ay hapı, hafta günü başlıkları, hafta numaraları, ay simgesi, Bugün vurgusu, ızgara çizgileri ve not/görev/olay noktaları), ayrıca takvim **yazı tipi**. Siz düzenledikçe canlı, tam boyutlu bir önizleme güncellenir; uygulamak için **Keep** (Koru) düğmesine tıklayın.

> **Günlük not dosya adları, görüntülenen takvimden bağımsız olarak her zaman Miladi kalır** (`YYYY-MM-DD`) — böylece dosyalarınız taşınabilir kalır ve doğru sıralanır. Kültürel tarih takvimde gösterilir (ve notun frontmatter'ına kaydedilebilir).

Takvim, günlük notlara tam olarak hizmet eder: herhangi bir güne tıklayarak onu açın, ya da bugüne atlamak için **"Günlük Not"** komutunu (komut paleti) çalıştırın.

**Bir notta kültürel tarih kaydetme.** İki isteğe bağlı araç, kültürel tarihi bir notun **özelliklerine** yazar (dosya adı her zaman Miladi `YYYY-MM-DD` kalır):

- **Günlük not Hicri damgası** — *Ayarlar → Takvim → "Hicri tarihi günlük notlara damgala."* Açık olduğunda (yalnızca Hicri takvim **ana veya ikincil** takviminizken kullanılabilir), her **yeni** günlük not bir `hijri:` satırı alır, örneğin `hijri: 1448-01-06`. Zaten sahip olduğunuz notlara asla dokunulmaz.
- **Bir notun Özellikler bölümündeki "+ Hijri"** — herhangi bir notun **Özellikler** bölümünü açın, tarihin üzerine gelin ve küçük bir **"+ Hijri"** düğmesi belirir (ayrıca "+ Jalali", "+ Hebrew" vb. — **seçtiğiniz her Miladi olmayan takvim için bir düğme**). Buna tıklayın; Constellation notun Miladi tarihini okur ve karşılığını ekler, örneğin `jalali: 1405-03-30`. Kore düğmesi **Dangi** yılını yazar; bir Çin/Kore **artık ayı** bir `L` ile işaretlenir (örneğin `chinese: 2025-06L-17`). Notun bir tarih özelliği yoksa, dosyanın oluşturulma tarihi kullanılır.

---

## 16. Lens ve Constellation Base

**Lens**, ilgilendiğiniz özelliklerle birlikte notların filtrelenmiş ve sıralanmış bir listesini gösteren kayıtlı bir sorgudur. Constellation iki yol sunar:

### Constellation Base — gömülü Lens blokları

Bir Lens'i herhangi bir Markdown notunun gövdesine doğrudan bir ` ```base ` kod bloğu kullanarak gömebilirsiniz:

````markdown
```base
schema: 1
view: list
dimensions: [note.name, note.created_at]
sort: [note.created_at, desc]
limit: 20
```
````

Notu görüntülediğinizde, kod bloğu eşleşen notları gösteren etkileşimli bir tabloyla değiştirilir. Canlı önizlemede, bloğu düzenlemek için **Lens** çipine tıklayın.

**v1'de kullanılabilen boyutlar:** `note.name`, `note.path`, `note.created_at`, `note.headline`.

**Federasyon:** varsayılan olarak Lens blokları aktif evren VE bağlı her cUniverse'den okur. Yalnızca aktif evrenle sınırlamak için YAML'da `federation: active` ayarlayın.

### Tam sekme Temel

Bir `.base` dosyasını açın; sekmeyi etkileşimli bir tablo olarak doldurur — her not için bir satır, her özellik için bir sütun. Ad sütunu her zaman ilk sıradadır; bir notu açmak için adına tıklayın. Bu tam sekme tablosunda, bir notu hızla bulmanın üç yolu vardır:

- **Bu Temel'de ara** — başlıktaki arama kutusu, siz yazdıkça satırları süzer; bir notun adıyla *ve* görünen her sütunun metniyle eşleşir. Süzme sırasında sayı rozeti `eşleşen / toplam` değerini gösterir (örneğin `4/7684`). Her yazıda arar — Arapça başlıkları bulmak için Arapça yazın. Binlerce satırda bile süzme anında gerçekleşir.
- **Harf rayı** — 50 veya daha fazla satırlı bir Temel'de, tablonun kenarında ince bir harf şeridi belirir; gerçek not başlıklarınızın ilk harflerinden oluşturulur (böylece İngilizce için A–Z, Arapça için أ ب ت… ve sahip olduğunuz başka herhangi bir yazı için doğru harfleri gösterir). O harfle başlayan ilk nota doğrudan atlamak için bir harfe tıklayın — tablo henüz Ad'a göre sıralı değilse, önce Ad'a göre sıralar, sonra atlar.
- **Bir satıra sağ tıklayın** — standart not menüsünü açar: Aç, Yeni sekmede aç, Yer İmi, Yolu / adı kopyala, Dosya ağacında göster, Varsayılan uygulamada aç, Sistem gezgininde göster, Stil… (Yeniden adlandırma, taşıma ve silme burada bilinçli olarak sunulmaz — bunları dosya ağacından yapın.)

### Beş Eylem (Five Acts) — yerleşik Lens'ler

Kenar çubuğunun **Five Acts** bölümü (Workspace Bases'ün üzerinde), `{universe}/Five Acts/*.md` altında Constellation tarafından düzenlenmiş ana notları listeler. v1 birini içerir: **Observation — Recent Captures** (en son yakalanan 20 notun federasyonel listesi). Bu notları serbestçe düzenleyebilirsiniz — Constellation düzenlemelerinizin üzerine yazmaz.

### Klasik Lens paneli

Eski Lens paneli (etiketlere, klasörlere, özelliklere göre filtreleme) hâlâ **Ayarlar → Paneller → Lens** üzerinden kullanılabilir.

### Yapı (yapısal bağlantılar)

**Yapı** paneli, açık notun daha büyük bir *yapıtın* — bir kitap, bir senaryo, bir kurs, bir İçerik Haritası — içinde nerede durduğunu gösterir. Geri Bağlantılar ve Giden Bağlantılar panellerinden farklı bir soruya yanıt verir. Onlar *"bu fikir başka bir fikirle nasıl ilişkilidir?"* sorusuna yanıt verir (düşünme bağlantıları — destekler, çelişir, neden olur…). Yapı ise *"bu not, oluşturmakta olduğum bütün yapıtın içinde nerede durur?"* sorusuna yanıt verir — Kitap → Bölüm → Fasıl → Sahne.

Bu, bir yapıtın **kompozisyon omurgasıdır**: içindekiler tablosu, sıralı taslak. Her düşünme, olgunluk ve bağlantı ölçüsünün **dışında** tutulması bilinçli bir tercihtir — bir notu "bir Kitabın altına" yerleştirmek, o notun olgunluğunu, bağlantı sayımlarını veya Sky View'daki varlığını asla değiştirmez. Bir içindekiler tablosu, yargılanacak bir iddia değil, yazarlıktır.

**İki tür yapısal bağlantı** (yalnızca tek tarafı yazarsınız — Constellation tersini sizin için çözer):

- **`parent`** — *bu notun* tek bir ebeveynin altındaki yeri (örneğin bir fasıl, ait olduğu bölümü beyan eder).
- **`contains`** — *bu notun* sıralı çocuk listesi (örneğin bir kitap, bölümlerini okuma sırasıyla listeler).

**Yapısal bir bağlantı oluşturma** — notun **Özellikler** bölümünü açın (sağ kenar çubuğundaki Özellikler sekmesi veya notun üstündeki özellikler bloğu):

1. **+ Özellik ekle**'ye tıklayın ve `parent` ya da `contains` anahtarını yazın.
2. Değer kısmına **hedef notun adını** yazın — yalnızca adı, örneğin `Part I - The Cartographer`. Constellation onu sizin için bir `[[link]]` içine sarar; köşeli parantezleri **siz yazmazsınız**. (Zaten parantezli bir ad yapıştırırsanız, yine tek bir `[[ad]]` olarak temiz biçimde saklanır — asla çift sarmalanmaz.)
3. `contains` için her çocuğu, okunmasını istediğiniz sırayla kendi çipi olarak ekleyin — o sıra, taslak sırası olur.

Yapısal bağlantılar **güvenle yeniden adlandırılır**: bir faslı yeniden adlandırın, yapıdaki yeri otomatik olarak takip eder, çünkü bağlantı dondurulmuş bir metin parçasına değil, notun kendisine işaret eder.

**Yapı panelini okuma** — sağ kenar çubuğundaki **Yapı** sekmesini açın (Geri Bağlantılar'ın hemen ardından):

- Panel, **bütün yapıtı** girintili bir taslak olarak (teal renkli madde imleriyle) gösterir; başlığında, alt öğelerin sayımıyla birlikte **TASLAK (OUTLINE)** yazar — yalnızca açık notun kendi çocukları değil.
- Şu anda görüntülemekte olduğunuz not, o taslağın içinde **vurgulanır** ("buradasınız").
- Üst kısımdaki bir **kırılma yolu (breadcrumb)** omurga boyunca yukarı giden yolu gösterir (örneğin *The Atlas of Lost Places › Part I › Chapter 1*). O nota atlamak için herhangi bir kırıntıya — veya herhangi bir taslak satırına — tıklayın.
- Bir **Bütün yapıt ⇄ Bu not** anahtarı (panelin sağ üstünde), bütün yapıt ile yalnızca açık notun kendi alt ağacı arasında geçiş yapar. Yalnızca notun gerçekten bir ebeveyni olduğunda görünür, böylece iki görünüm farklılaşır.
- Yapı yanlışlıkla kendi üzerine döngü oluşturursa (A notunun ebeveyni B, B'nin ebeveyni de A ise), taslak zinciri çizer ve ardından temiz biçimde durur, kesim noktasını küçük bir **↻** ile işaretler. Asla takılıp kalmaz.

**Bir çatışmayı çözme (İtiraz edilen / Contested).** İki not da aynı çocuğu sahiplenirse — biri çocuğun kendi `parent` özelliği üzerinden, diğeri bir `contains` listesi üzerinden — panel, o satırı sessizce bırakmak yerine **İtiraz edilen (Contested)** olarak işaretler (diğer sahiplenen tarafı adlandıran amber renkli bir ⚠ rozeti). İki tek tıklamalık düğme bunu çözer:

- **Tut (Keep)** — çocuğun kendi beyan ettiği ebeveyni koru (bu not, çocuk üzerindeki iddiasından vazgeçer).
- **Buraya taşı (Move here)** — bu notu ebeveyn olarak kabul et (çocuğun `parent` değeri bu nota geçer).

Her iki düğme de not dosyalarını doğrudan günceller ve taslağı yeniler. Sizin tıklamanız olmadan hiçbir şey asla değiştirilmez.

---

## 17. Ayarlar

Kenar çubuğundaki dişli simgesinden veya `Ctrl+,` ile ayarlara erişin.

### Genel

- Dil (15 dil)
- Tema (Açık / Koyu)
- Arayüz yazı tipi, Metin yazı tipi, Mono yazı tipi, Yazı tipi boyutu
- Yazı tipi teması — hazır yazı tipi kombinasyonları (Daktilo, Klasik, Modern vb.) hızlı geçiş için
- **Temalar** — altı yerleşik temadan seçin, özel temalar oluşturun (beş renkli düzenleyici), Obsidian topluluk kayıt defterinden temalar içe aktarın (200+ tema) veya bir `.json` tema dosyası içe aktarın. Üzerine gelindiğinde ✕ düğmesiyle herhangi bir özel temayı silin.

### Style Settings

Arayüzün her görünür öğesinin ince ayarı için özel bir sekme, aktif temaya canlı olarak uygulanır.

- **Renkler** — arka plan, yüzeyler, metin (normal/azaltılmış/soluk), vurgu, kenarlıklar, durum renkleri
- **Tipografi** — arayüz/not/kod yazı tipi boyutları, H1–H6 boyutları, başlık ağırlığı, satır yükseklikleri, paragraf aralığı
- **Düzen ve Şekil** — küçük/orta/büyük köşe yarıçapları, kenarlık genişlikleri, gölgeler, editör okunabilir satır uzunluğu, yan kenar boşlukları
- **Bileşenler** — şerit dock, yan işlem çubuğu, düzen çubuğu (panel anahtarları), üst çubuk/sekme şeridi, durum çubuğu, sağ kenar çubuğu (müfettiş), dosya gezgini (Evren notları, alt evrenler, kitaplıklar, klasörler, notlar), düğmeler, etiketler, callout'lar — her biri bağımsız boyut, yarıçap, renk ve geçerli olduğunda aktif durum stili ile
- **Editör** — bağlantı renk/üzerine gelme/dekorasyon, satır içi kod renk/arka plan/yarıçap, alıntı çubuğu genişlik/renk, imleç rengi, seçim arka planı

**İçe Aktar / Dışa Aktar** — sekmenin üstündeki araç çubuğu:
- Panodan yapıştır (tek tıklama)
- İçe aktar / Yapıştır (Birleştir veya Değiştir ile metin alanı)
- Dosyadan (.json)
- Kopyala (mevcut değerleri panoya)
- Dışa aktar (.json)

Biçim Obsidian'ın Style Settings eklentisiyle tam olarak eşleşir, böylece Obsidian ve Constellation arasında ayarları paylaşabilirsiniz.

Değişiklikler aktif temaya otomatik kaydedilir; yerleşik bir temayı düzenlerseniz, değişiklikler orijinali değiştirmeden kalıcı olacak şekilde özel temalarınıza otomatik klonlanır.

### Stil Ayarlayıcı

**Stil Ayarlayıcı**, tam ekran bir tasarım stüdyosudur — **Ayarlar → Görünüm → "✦ Open Style Setter"** üzerinden açın. Gerçek arayüzünüzü ortada gösterir; herhangi bir bölüme (kenar çubuğu, not başlığı, başlık, bağlantı, not sayfası) tıklayın, o öğenin denetimleri sağda belirir ve önizleme anında güncellenir. Tema kartları (Midnight / Daylight / Chocolate / Nord) bütün bir görünümü tohumlar — stüdyonun kendisi siz tasarlarken onu giyer — ve *Yüzeyler* listesi görünümü yalnızca editörde değil, uygulama genelinde önizler. **"Apply to app"** vurgunuzu, arka planlarınızı, metin renginizi ve yazı tiplerinizi gerçek Constellation'a aktarır; **"Esc"** veya **"✕"** yalnızca Ayarlayıcı'yı kapatır, Ayarlar'ı değil. Uygulama şimdilik oturum için canlı bir önizlemedir; bir görünümü kalıcı, adlandırılmış bir Stil olarak kaydetmek (yeniden kullanılabilir, yeniden adlandırılabilir renk örnekleri ve dışa / içe aktarma ile) bir sonraki sürümde gelecek.

### Arapça Geçersiz Kılmalar

Arapça motorun belirli yüzey biçimlerini nasıl çözümleyeceğini sabitlediğiniz, Evren başına bir panel — kendi türettiğiniz sözcükler, yerel adlar, alana özgü ödünç sözcükler veya motorun otomatik okumasıyla aynı fikirde olmadığınız durumlar için. Her geçersiz kılma üretici FST, kaskad ve sezgisel yedeği geçer. Bir geçersiz kılma eklemek veya kaldırmak, yalnızca etkilenen yüzey biçimini içeren notların hedefli bir şekilde yeniden dizinlenmesini tetikler — tam yeniden oluşturma yoktur. Adım adım kılavuz için §19 ("RTL ve Arapça Desteği") bölümüne bakın.

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

## 18. Klavye Kısayolları

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

## 19. RTL ve Arapça Desteği

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

### Arapça Motor Geçersiz Kılmaları

Constellation'ın Arapça motoru, her aramanın, her bağlantının ve her dizin girdisinin altında çalışan beş katmanlı bir biçimbilim çözümleyicisidir. Kökleri, örüntüleri, özel adları, ödünç sözcükleri ve fonolojik onarımları anlar — böylece كاتب sorgusu كتبنا ve كتاب sözcüklerini bulur, ancak وائل bir ad olarak olduğu gibi kalır, ائل olarak bozulmaz.

Ayarlar'daki **Arapça Geçersiz Kılmalar** paneli, motora kendi terminolojinizi öğrettiğiniz yerdir. Her geçersiz kılma egemen yanıttır — üretici FST, kaskad ve sezgisel yedeği geçer.

**Geçersiz kılmaları ne zaman kullanmalı:**
- Motorun bilmediği kişi adları, yerel yer adları veya alana özgü terimler
- Evreninize özgü türetilmiş sözcükler veya kısaltmalar
- Belirli bir yazımı korumak istediğiniz ödünç sözcükler
- Motorun otomatik çözümlemesinin sözcüğü nasıl okuduğunuzla çeliştiği her durum

**Adım adım:**

1. **Ayarlar** bölümünü açın (dişli simgesi veya `Ctrl + ,` / `Cmd + ,`) ve kenar çubuğundan **Arapça Geçersiz Kılmalar** öğesini seçin.
2. **Geçersiz kılma ekle** düğmesine tıklayın.
3. Şunları doldurun:
   - **Yüzey biçimi** — Arapça sözcüğü yazdığınız şekliyle
   - **Lemma** — motorun döndürmesi gereken kanonik biçim
   - **Kök** (isteğe bağlı) — sözcüğün klasik bir kökü varsa 3 veya 4 ünsüz
   - **Örüntü** (isteğe bağlı) — örn. `فاعل`
   - **Söz türü** — Özel ad / Ad / Sıfat / Zarf / Fiil / İlgeç / Yabancı / Bilinmiyor
   - **Not** (isteğe bağlı) — kendiniz için bir bağlam satırı
4. **Kaydet** düğmesine tıklayın. Panel, yüzey biçimini içeren her not yeniden belirteçlenirken **Yeniden dizinleniyor…** gösterir ve tamamlandığında **N not yeniden dizinlendi** gösterir.
5. Bir geçersiz kılmayı kaldırmak için satırındaki **×** düğmesine tıklayın — aynı yeniden dizinleme taraması tersine çalışır.

Geçersiz kılmalar Evren başına `<universe>/.constellation/arabic-overrides.json` konumunda saklanır — düz metin, alfabetik sıralı, atomik yazılmış. Dosyayı sürüm kontrolüne alabilir veya cihazlar arasında paylaşabilirsiniz.

---

## 20. Güvenlik ve Gizlilik

- **Tüm veriler yerelde kalır** — bulut senkronizasyonu yok, telemetri yok, izleme yok
- **Markdown dosyaları** — notlarınız tamamen size ait düz metin dosyalarıdır
- **Hesap gerekmez** — Constellation tamamen çevrimdışı çalışır
- **İsteğe bağlı güncellemeler** — Ayarlar üzerinden güncellemeleri elle kontrol edin
- **Açık kaynak** — kodu [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation) adresinden inceleyin

---

## 21. Bilgi Haritası

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

## 22. Bilişsel Motor

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

*Constellation Kullanım Kılavuzu — Sürüm 0.1.0 — Mart 2026*
*uconstellation.world*

---

## 23. Önerilen Bağlantılar

Constellation bilgiyi *formüle etmek* içindir ve bilgi bağlantıdır. **Önerilen Bağlantılar**, kütüphanenizde halihazırda bulunan ve baktığınız notla en çok ilişkili olan notları bulur — bağlanması gereken ama henüz bağlanmamış akrabalarını — ve bunların herhangi birini tek tıkla bir **türlü bağlantıya** dönüştürür. Bu, "buna benzer daha fazlası" ama düşünmek içindir.

**Her öneri türlüdür.** Bir öneriyi kabul ettiğinizde, Constellation iki notun *nasıl* ilişkili olduğunu sorar — destekler, çelişir, örnekler, türetilir, vb. ya da yalnızca **ilişkili**. Türlü bir bağlantı, daha sonra okuyabileceğiniz, arayabileceğiniz ve sorgulayabileceğiniz bir muhakeme parçasıdır; özellik asla toplu olarak bağlantı eklemez ve asla sessizce türsüz bir bağlantı eklemez. (Bkz. **Bilgi Formülasyonu** ve **Özellikler**.)

**Onları nasıl bulur.** Adaylar **yalnızca kendi kütüphanenizden** gelir ve en *ayırt edici* ortak söz dağarcığına göre — yaygın olanlara değil, nadir ve belirleyici sözcüklere göre — Constellation'ın canlı arama dizinine karşı sıralanır. Her öneri, neden ortaya çıktığını açıklayan **ortak terimleri** gösterir, böylece asla bir kara kutu tahminini kabul etmezsiniz.

**Beş yer, tek liste.** Aynı öneri listesi şu yerlerde görünür: **Gözden Geçirme** (🕐, *öksüz* veya *kırılgan* olarak işaretlediği notlar için), **Geri Bağlantılar sekmesi** (sağ kenar çubuğu), **360° Denetleyici**, **Sağlık sekmesi** ve **Sky View** (🌌 — herhangi bir yıldıza sağ tıklayın → **Bağlantıları öner…**).

**Gelen mi giden mi — ve neden siz seçmiyorsunuz.** Tanı yüzeyleri (**360° Denetleyici** ve **Sağlık sekmesi**) **gelen** bağlantılar önerir — *hangi notlar **buraya** işaret etmeli*. Genel yüzeyler (**Geri Bağlantılar sekmesi** ve **Sky View**) **giden** bağlantılar önerir — *bu not neye **işaret etmeli***. Yönü, işine uyan biçimde yüzey seçer; notu ve türü siz seçersiniz. (Gelecekteki bir güncelleme, yönü kendiniz değiştirmenize olanak tanıyacaktır.)

**Kullanımı.** **Önerilen Bağlantılar** başlığı altında, en yakın önce sıralanmış ilgili notları, her birini ortak terimleriyle birlikte göreceksiniz. Bir adayın **Bağla** düğmesine tıklayın → küçük **"Nasıl ilişkililer?"** menüsünde ilişki türünü seçin → türlü bağlantı **anında** oluşturulur ve öneri listeden düşer. Ardından notun **özelliklerinde** yaşar, geri bağlantılarında/giden bağlantılarında ve grafiğin her yerinde görünür. Hiçbiri gerçekten uymuyorsa onları olduğu gibi bırakın — ya da Gözden Geçirme'de notu bilinçli bir **bağımsız** olarak işaretleyin. Önerilen Bağlantılar önerir; kararı siz verirsiniz.

**Yerel, gizli, engellemeyen.** Öneriler yalnızca kütüphanenizden talep üzerine hesaplanır — hiçbir şey cihazınızdan ayrılmaz — ve onların toplanması yazmanızı asla engellemez (çalışırken kısa bir "İlgili notlar bulunuyor…" göreceksiniz). Öneriler, ortak terim ipuçları ve ilişki türleri seçtiğiniz dilde görünür ve sağdan sola yazılar için doğru biçimde yansıtılır.

---

## 24. Bilişsel Renkler ve Sağ Tık Menüleri

### Özellikler stillemesi (Stil Tasarımcısı)

**Stil Tasarımcısı**'nı açın (Ayarlar → Görünüm → ✦ Stil Tasarımcısı'nı Aç ya da kendi sekmesi) ve bir notun frontmatter'ındaki küçük etiketleri yeniden stillemek için **Özellikler** kategorisini seçin. İki öğe: **Özellik etiketleri** (sıradan `tags` tarzı çipler — Etiket arka planı, Etiket metni, Etiket yarıçapı 0–20 px, Yükseklik 14–32 px) ve **Taksonomi rozetleri** (Arka plan, Metin, Yarıçap 0–20 px). Ortadaki canlı bir önizleme siz düzenledikçe güncellenir; her değer tam olarak bugünkü görünümden başlar, böylece bir kontrole dokununcaya kadar hiçbir şey değişmez. Bu Evren için kaydetmek üzere **Keep** tıklayın.

### Bilişsel renkler (Stil Tasarımcısı)

**Bilişsel renkler** kategorisi size **her bilişsel durum için tek bir paylaşılan renk** verir, böylece o durumu gösteren her yüzey uyum sağlar. Beş set:

- **Olgunluk** — Tohum, Fidan, Her dem yeşil, Kanonik, Solan.
- **Güven** — Hipotez, Kanıt, Yerleşik, Tartışmalı.
- **Köken** — Alınan, Keşfedilen, Karma, Hiçbiri.
- **Aşama** — Kıvılcım, Doğuş, Büyüme, Olgunluk, Uyku, Arşivleme.
- **Eşleşme kategorisi** (bir arama sonucunun neden eşleştiği) — Başlık, İçerik, Etiket, Vikibağlantı, Özellik, Anlamsal, Yapılandırılmış.

Davranış **istek üzerine birleştir**'dir: bir renk seçene kadar hiçbir şey değişmez. Her yüzey mevcut rengini yedek olarak korur ve bir durumun rengini burada belirlediğiniz an, o durumu gösteren **her** yüzey — dosya ağacı, sekmeler, not denetleyicisi, editör içi arama vurgusu, eşleşme rozeti ve arama sonucu vurgusu — aynı anda sizin renginize geçer. Bir durumu dokunulmadan bırakın, tam olarak eskisi gibi görünür. Kaydetmek için **Keep** tıklayın.

### Sağ tık menüleri

Constellation üç yerde bir bağlam menüsü sunar; her biri yalnızca tıkladığınız yere uyan eylemleri sunar:

- **Not gövdesine sağ tıklayın** — Bağlantı ekle / Harici bağlantı; **Biçim ▸** (Kalın, İtalik, Altı Çizili, Üstü çizili, Vurgula, Satır içi kod, Satır içi matematik, Yorumu değiştir, Üst simge, Alt simge, Biçimlendirmeyi temizle); **Paragraf ▸** (Madde işaretli/Numaralı/Görev listesi, H1–H6, Gövde, Alıntı); **Ekle ▸** (Dipnot, Tablo, Uyarı kutusu, Yatay çizgi, Kod bloğu, Matematik bloğu, Resim); Kes / Kopyala / Yapıştır / Düz metin olarak yapıştır / Tümünü seç; ve **Stil…** (Stil Tasarımcısı'nı **Editör** kategorisinde açar).
- **Bir frontmatter özellik satırına sağ tıklayın** — Değeri kopyala, Adı kopyala, Özelliği kaldır, Özellik ekle; ardından gövdedeki ile aynı düzenleme menüsü; ve Stil Tasarımcısı'nı **Özellikler** kategorisinde açan **Stil…**.
- **Bir arama sonucuna sağ tıklayın** — **güvenli** bir alt küme: Aç, Yeni sekmede aç, Dosya ağacında göster, Bağlantıyı kopyala, Yolu kopyala, Yer imini ekle, Sistem gezgininde göster, Varsayılan uygulamada aç ve **Stil…** (**Bilişsel renkler** kategorisi). Tasarım gereği burada **Yeniden adlandır, Taşı veya Sil yoktur** — arama paneli dosya ağacının saniye saniye güncel bir kopyasını tutmaz, bu yüzden yıkıcı eylemler görünümün her zaman güncel olduğu dosya ağacında kalır.

Her **Stil…** girişi, sağ tıkladığınız şeyin kategorisine iner, böylece doğru kontrolleri asla aramak zorunda kalmazsınız. Her menü öğesi, kategori adı ve durum etiketi seçtiğiniz arayüz dilinde görünür ve sağdan sola düzenler için yansır.
