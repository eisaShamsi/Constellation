---
aliases:
  - Takvim Paneli
  - Günlük Not Takvimi
  - Kültürel Takvimler
description: Tıklanabilir günler, günlük not oluşturma, görev teslim tarihleri ve kültürel tarih kaydı ile sekiz takvim arasında tam sayfa bir ay görünümü.
---

# Takvim

**Takvim**, **sol dock**'tan (takvim simgesi) açılan tam sayfa bir ay görünümüdür. Notu veya teslim tarihi olan görevi bulunan günler renkli **noktalar** ile işaretlenir. Başlık, ayı seçtiğiniz takvimde gösterir; bir **ikincil takvim** ayarlarsanız, altındaki alt başlık o takvimin denk gelen aralığını gösterir (örneğin bir Miladi ay, Hicri karşılığı olan "Zilhicce 1447 – Muharrem 1448 AH" aralığını gösterir).

## Bir Güne Tıklama

Her gün hücresi etkileşimlidir:

| Eylem | Sonuç |
|--------|--------|
| Boş alana (veya gün numarasına) tıklayın | O günün **günlük notunu** açar — veya oluşturur. Zaten bir günlük notu olan bir tarihe tıklamak yalnızca onu **açar**; asla bir kopya oluşturmaz. |
| Bir noktaya tıklayın | O belirli öğeyi açar. Bir günde birkaç not veya görev varsa, noktaya tıklamak seçim yapabileceğiniz küçük bir **liste** gösterir. |
| Bir görev noktasına tıklayın | Notu **o görevin satırına kaydırılmış** olarak, düzenlemeye hazır şekilde açar. |

### Nokta Renkleri

| Nokta Rengi | Anlamı |
|-----------|---------|
| Altın | O güne ait **günlük not** |
| Mor | O gün düzenlenen (veya o tarihli) başka bir **not** |
| Kırmızı | O gün teslim tarihi olan bir **görev** |

Tüm nokta renkleri — ve takvimin diğer her parçası — **Style Setter → Takvim** yüzeyinden temalandırılabilir.

> [!tip]
> Görev listesinde, doğrudan takvimden **bir görevin onay kutusunu işaretleyerek onu tamamlayabilirsiniz** — tamamlanan görevler anında listeden düşer. Takvimde yalnızca kendi `📅 YYYY-MM-DD` teslim tarihini taşıyan görevler görünür (onları bir güne yerleştiren şey bu tarihtir).

## Kültürel Takvimler (Sekiz)

**Ayarlar → Takvim** bölümünde **takvim sistemini** ayarlayabilirsiniz ve tüm ay ızgarası ona geçer:

- **Miladi**
- **Hicri (İslami)** — doğru bir astronomik motor; mübarek aylar vurgulanır ve İslami olaylar işaretlenir.
- **Şemsi Hicri (İran/Farsça)**
- **İbrani**
- **Hint (Saka)**
- **Budist**
- **Çin** — *ay-güneş*
- **Kore** — *ay-güneş*

Her hücre, hem seçilen takvim tarihini (büyük) hem de Miladi tarihi (küçük) ve ayrıca ay evresini gösterir. Her ay başlığı ayın **adını, parantez içinde numarasını ve yılı** gösterir — numara, ay sırası tanıdık olmayan takvimlerde yardımcı olur.

**Çin ve Kore** takvimleri *ay-güneş* takvimleridir: bazen bir **artık ay** (闰六月 / 윤6월) eklerler; takvim bunu kendi sayfası olarak gösterir, böylece gezinme asla bir ayı atlamaz veya ikilemez.

Ayrıca **hafta başlangıcını** (Pazar/Pazartesi) seçebilir ve **hafta numarası sütununu** açıp kapatabilirsiniz.

### Hicri Takvim Seçenekleri

**Ayarlar → Takvim → "Hicri takvim (İslami)"** altında iki ek denetim vardır:

- **Hesaplama yöntemi** — gerçek yeni ayı (hilâl kavuşumu) izleyen **Astronomik (Ay Kavuşumu)** (en doğru olan, varsayılan), ya da klasik aritmetik döngü olan **Tablo Bazlı (al-Tawfīqāt al-Ilhāmiyyah)** (İlham edilmiş tevfikler).
- **Ay düzeltmesi** — bir Hicri ayın başlangıcını **yerel hilâl gözlemine** uydurmak için ±1 veya ±2 gün kaydırın. Hicri yılı ve ayı seçin, bir kayma değeri belirleyin ve **Set** (Ayarla) düğmesine tıklayın; düzeltme o aya ve ondan sonraki her aya uygulanır. Düzeltmeleriniz listelenir (her biri kaldırılabilir) ve bir **Clear all** (Tümünü temizle) düğmesi bulunur.

Her iki ayar da (ve düzeltmeleriniz) **evreninizle birlikte** kaydedilir, böylece cihazlarınız arasında taşınırlar.

### Çin ve Kore Görüntüleme Seçenekleri

Kore, Çin ay takvimini kullanır, dolayısıyla ikisi aynı tarihleri paylaşır — onları ayıran şey **yazı** ve **yıldır**. İkisinden biri ana veya ikincil takviminiz olduğunda, **Ayarlar → Takvim** iki ek denetim gösterir:

- **Yıl gösterimi** — Çin: altmışlık döngü 丙午年, sade yıl veya her ikisi; Kore: **Dangi** dönemi 단기 4359, yıl veya altmışlık 병오년.
- **Ay adları** — *yerel yazı* (五月 / 5월) veya *fonetik* — yani ayın telaffuzunun kendi dilinizde yazılmış hali (İngilizce "Wǔyuè / Owol"; Arapça "وُو-يوي / أوه-وُل").

## Takvimi Biçimlendirme

**Style Setter**'ı (sol dock veya **Ayarlar → Style Setter**) açın ve her parçayı yeniden biçimlendirmek için **Takvim** yüzeyini seçin — her öğenin kendi **rengi ve metin boyutu** vardır (gün numaraları, çapraz referans tarihi, ay hapı, hafta günü başlıkları, hafta numaraları, ay simgesi, Bugün vurgusu, ızgara çizgileri ve not/görev/olay noktaları), ayrıca takvim **yazı tipi**. Siz düzenledikçe canlı, tam boyutlu bir önizleme güncellenir; uygulamak için **Keep** (Koru) düğmesine tıklayın.

## Günlük Notlar

Takvim, günlük notlara tam olarak hizmet eder: herhangi bir güne tıklayarak onu açın, ya da bugüne atlamak için **"Günlük Not"** komutunu (komut paleti) çalıştırın.

> [!tip]
> **Günlük not dosya adları, görüntülenen takvimden bağımsız olarak her zaman Miladi kalır** (`YYYY-MM-DD`) — böylece dosyalarınız taşınabilir kalır ve doğru sıralanır. Kültürel tarih takvimde gösterilir ve notun frontmatter'ına kaydedilebilir (aşağıya bakın).

## Bir Notta Kültürel Tarih Kaydetme

İki isteğe bağlı araç, kültürel tarihi bir notun **özelliklerine** yazar (dosya adı her zaman Miladi `YYYY-MM-DD` kalır):

- **Günlük not Hicri damgası** — *Ayarlar → Takvim → "Hicri tarihi günlük notlara damgala."* Açık olduğunda (yalnızca Hicri takvim **ana veya ikincil** takviminizken kullanılabilir), her **yeni** günlük not bir `hijri:` satırı alır, örneğin `hijri: 1448-01-06`. Zaten sahip olduğunuz notlara asla dokunulmaz.
- **Bir notun Özellikler bölümündeki "+ Hijri"** — herhangi bir notun **Özellikler** bölümünü açın, tarihin üzerine gelin ve küçük bir **"+ Hijri"** düğmesi belirir (ayrıca "+ Jalali", "+ Hebrew" vb. — **seçtiğiniz her Miladi olmayan takvim için bir düğme**). Buna tıklayın; Constellation notun Miladi tarihini okur ve karşılığını ekler, örneğin `jalali: 1405-03-30`. Kore düğmesi **Dangi** yılını yazar; bir Çin/Kore **artık ayı** bir `L` ile işaretlenir (örneğin `chinese: 2025-06L-17`). Notun bir tarih özelliği yoksa, dosyanın oluşturulma tarihi kullanılır.

> [!tip] RTL Support
> Takvim ızgarası geçerli metin yönüne uyar. RTL dillerinde (Arapça, İbranice, Farsça, Urduca), takvim düzeni buna göre ayarlanır.
