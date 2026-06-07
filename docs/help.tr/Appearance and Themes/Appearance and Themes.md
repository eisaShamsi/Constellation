---
aliases:
  - Temalar
  - Stil Tasarımcısı
  - Style Setter
  - Style Settings
  - Özel tema
  - Obsidian teması içe aktar
  - Tema sil
  - Kayıtlı stili dışa aktar
description: Constellation'ın her görünür parçasını kişiselleştirin — temalar Görünüm sekmesinden, tüm stilleme (renkler, tipografi, bileşenler, bağlantı türü renkleri ve kayıtlı stiller) Stil Tasarımcısı'ndan.
---

# Görünüm ve Temalar

Constellation'ın görünümü **Ayarlar**'dan kontrol edilir:

1. **Görünüm** — bir tema seçin veya oluşturun, Obsidian'ın topluluk kayıt defterinden temalar içe aktarın ve başlık hizalaması ile Canlı Bağlantı yaşam döngüsünü ayarlayın.
2. **Stil Tasarımcısı (Style Setter)** — Ayarlar kenar çubuğunda kendi sekmesi olan ve artık **tüm stillemenin tek evi** olan bölüm: arayüz kabuğunun ve editörün her rengi, boyutu, yazı tipi ve öğesi, bağlantı türü renkleri ve kayıtlı stiller. (Eski **Style Settings** sekmesi emekliye ayrıldı ve tamamen buraya katıldı.)

Birlikte, uygulamayı iş akışınıza, ekran boyutunuza ve kişisel zevkinize göre yeniden şekillendirmenize izin verirler — tek bir satır CSS düzenlemeden.

## Temalar

**Tema**, Constellation'ın nasıl göründüğünü tanımlayan adlandırılmış bir renk, ayar ve CSS paketidir. Constellation altı yerleşik tema ile gelir (Constellation Light/Dark, Nord Light/Dark, Solarized Light/Dark), tümü açık ve koyu sistem modları arasında eşlenmiştir.

### Tema seçme

1. **Ayarlar → Görünüm** açın.
2. **Temalar** ızgarasındaki herhangi bir karta tıklayın. Tema hemen uygulanır.
3. Aktif kart bir vurgu kenarlığıyla vurgulanır.

### Özel tema oluşturma

1. Tema ızgarasında kesikli **+ Yeni Tema** kartına tıklayın.
2. Ad verin, açık veya koyu seçin ve beş renk seçin (arka plan, yüzey, metin, vurgu, kenarlık).
3. **Kaydet** tıklayın. Temanız artık ızgarada görünür.

Diğer tüm değişkenler (üzerine gelme durumları, gölgeler, azaltılmış metin) HSL matematiği kullanılarak beş renginizden otomatik olarak türetilir, böylece yalnızca önemli olanı kontrol edersiniz.

### Özel temayı düzenleme veya silme

Herhangi bir özel tema kartının üzerine gelin:
- **✏️ (kalem)** — adını, türünü veya beş ana rengini değiştirmek için düzenleyiciyi açar.
- **✕ (kırmızı X)** — onaydan sonra temayı siler. Yerleşik temalar silinemez. Aktif temayı silerseniz, Constellation varsayılana döner.

### Obsidian topluluk teması içe aktarma

200'den fazla topluluk temasına göz atmak için **🟣 Obsidian Temaları** tıklayın:
1. Ada veya yazara göre arayın.
2. Düzenin maketini ve beş renk paletini görmek için **Önizleme** tıklayın.
3. **İçe aktar** tıklayın — temanın CSS'i indirilir, Constellation için uyarlanır (seçici shim'i + değişken çıkarma + CodeMirror sözdizimi renkleri) ve özel temalarınıza eklenir.
4. Tema **Style Settings** destekliyorsa, sayı kartında gösterilir; bu seçenekler içe aktarma sonrası Style Settings sekmesinde görünür.

## Style Settings → artık Stil Tasarımcısı'nın içinde

> **Not:** Bağımsız **Style Settings** sekmesi emekliye ayrıldı. İçindeki her kontrol artık **Stil Tasarımcısı**'nın içinde (Ayarlar kenar çubuğundaki kendi sekmesi) — hepsini ve daha fazlasını kapsar (gezinti yolu, not ön izleme özeti, Evren paneli, yazı sistemi başına yazı tipleri). Aşağıdaki açıklama bu stilleme yüzeyini anlatır; artık Stil Tasarımcısı üzerinden açılır.

Bu stilleme yüzeyi, Constellation'ın yerel, tema-bağımsız kontrol panelidir. Çerçevenin her görünür parçasını artı editörü kapsar ve herhangi bir tema (yerleşik, özel veya içe aktarılmış) ile çalışır.

### Nasıl düzenlenmiş

Bölümler varsayılan olarak daraltılmıştır. Genişletmek için chevron'a tıklayın:

- **Constellation — Renkler** — arka plan ve yüzeyler, metin, vurgu
- **Constellation — Tipografi** — arayüz/not/kod yazı tipi boyutları, H1–H6 boyutları, başlık ağırlığı, satır yükseklikleri, paragraf aralığı
- **Constellation — Düzen ve Şekil** — köşeler (küçük/orta/büyük yarıçaplar), kenarlık genişlikleri, gölgeler, editör satır uzunluğu, yan kenar boşlukları
- **Constellation — Bileşenler** — şerit dock, yan işlem çubuğu, düzen çubuğu (panel anahtarları), üst çubuk/sekme şeridi, durum çubuğu, dosya gezgini, sağ kenar çubuğu, düğmeler, etiketler, callout'lar
- **Constellation — Editör** — bağlantılar, kod ve bloklar, blok alıntı, imleç ve seçim

### Bir değeri değiştirme

- **Renk seçiciler** — örneğe tıklayın, bir renk seçin. Hex yanında gösterilir.
- **Kaydırıcılar** — ayarlamak için sürükleyin. Sayısal değer birimde (px, %, vb.) görünür.
- **Anahtarlar** — sınıfları açıp kapatmak için tıklayın (çoğunlukla içe aktarılan temalar için).
- **Açılır menüler** — bir seçenek seçin (bağlantı dekorasyon stili, vb.).
- **Sıfırlama oku (↺)** — her satırın sonunda üzerine gelindiğinde görünür. Tıklamak geçersiz kılmanızı temizler ve temanın varsayılanını geri yükler.

### Kaydetme nasıl çalışır

- Değişiklikler otomatik olarak aktif temanın **styleSettingsValues**'a kaydedilir.
- Yerleşik bir tema aktifken bir Style Setting değiştirirseniz, Constellation yerleşiği özel temalarınıza (`{Ad} (custom)` olarak) **otomatik klonlar**, ardından değişikliklerinizi oraya kaydeder. Yerleşik dokunulmaz kalır.
- Sekmenin altındaki **Kaydedildi:** etiketi, geçersiz kılmalarınızı şu anda hangi temanın içerdiğini gösterir.
- Aktif temadaki tüm geçersiz kılmaları silmek için **Tümünü varsayılana sıfırla** tıklayın.

### Style Settings'i içe aktarma / dışa aktarma

Style Settings sekmesinin üstündeki araç çubuğu:

- **📋 Panodan yapıştır** — tek tıklama: panoyu okur ve geçerli JSON'u aktif temaya birleştirir.
- **⬆️ İçe aktar / Yapıştır** — bir metin alanı açar; JSON'u elle yapıştırın. **Birleştir** (ekler/geçersiz kılar) veya **Tümünü değiştir** (siler, yalnızca yapıştırılanı kullanır) seçin.
- **📄 Dosyadan** — Obsidian'ın Style Settings eklentisinden veya başka bir Constellation kurulumundan dışa aktarılmış bir `.json` dosyasını açın.
- **📋 Kopyala** — mevcut değerleri güzel biçimlendirilmiş JSON olarak panoya kopyalar.
- **⬇️ Dışa aktar** — değerleri `{theme-name}-style-settings.json` olarak kaydeder.

JSON biçimi Obsidian'ın Style Settings eklentisiyle tam olarak eşleşir — ayar ID'lerini dize değerlerine eşleyen düz bir nesne:

```json
{
  "h1-size": "36",
  "interactive-accent": "#7c3aed",
  "my-themed-color@@light": "#ffffff",
  "my-themed-color@@dark": "#1e1e2e"
}
```

Bu, Style Settings'inizi Obsidian'dan kopyalayıp doğrudan Constellation'a yapıştırabileceğiniz veya tersi anlamına gelir.

## Neleri kontrol edebilirsiniz

Her ayar yukarıdaki beş bloktan birinin altındadır. Öne çıkanlar:

### Tipografi

- **Arayüz yazı tipi boyutu** — kenar çubuğu, araç çubukları, menüler
- **Not yazı tipi boyutu** — editörde gövde metni
- **Kod yazı tipi boyutu** — satır içi kod ve çevrelenmiş kod blokları
- **H1 – H6 boyutları** — her başlık düzeyi bireysel olarak
- **Başlık ağırlığı** — tüm başlıkların hafifliği veya kalınlığı
- **Satır yükseklikleri** — normal (gövde) ve sıkı (başlıklar ve yoğun UI)
- **Paragraf aralığı** — paragraflar arası boşluk

### Kabuk bileşenleri

- **Şerit dock (sol simgeler)** — genişlik, düğme boyutu, simge boyutu, yarıçap, renkler
- **Yan işlem çubuğu** — yeni not/tablo/klasör simgeleri — boyut, renk, yükseklik, arka plan
- **Düzen çubuğu (panel anahtarları)** — sol/bölme/sağ kenar çubuğu anahtarları — düğme boyutu, simge boyutu, renkler, aktif durum rengi
- **Üst çubuk / sekme şeridi** — yalnızca notlar sekmelerde açıkken görünür; şerit yüksekliği, arka plan, sekme yüksekliği/yazı tipi/yarıçapı, aktif ve etkin olmayan sekme renklerini kontrol eder
- **Durum çubuğu** — yükseklik, yazı tipi boyutu, arka plan, metin rengi
- **Sağ kenar çubuğu (müfettiş)** — arka plan, sekme satırı yüksekliği, sekme simgesi boyutu, renkler
- **Dosya gezgini (sol kenar çubuğu)** — Evren notları satırı, alt evren (cUniverse) satırları, kitaplık adları, klasörler, notlar — her biri bağımsız boyut, ağırlık ve renkle; artı dikey satır aralığı

### Editör

- **Başlık boyutları** (H1–H6) ve ağırlık
- **Satır yüksekliği** not gövdesinde
- **Satır içi kod** arka plan, metin rengi, yarıçap, yazı tipi boyutu
- **Bağlantı rengi** (varsayılan + üzerine gelme) ve dekorasyon stili (yok/altı çizili/noktalı)
- **Callout çubuk genişliği** ve **callout yarıçapı**
- **İmleç rengi** ve **seçim arka planı**

### Renkler (uygulamadaki her renk)

- Arka plan (birincil/alternatif), yüzeyler, üzerine gelme arka planı, kenarlıklar, giriş arka planı
- Metin (normal/azaltılmış/soluk/vurgu üzerinde), hata/uyarı/başarı durumları
- Vurgu (etkileşimli vurgu + üzerine gelme), vurgu üzerinde metin

## Stil Tasarımcısı (Style Setter)

**Stil Tasarımcısı**, **tüm stillemenin tek evi** — arayüzünüzün tamamı için tam sayfa bir tasarım stüdyosudur. Ayarları tek tek değiştirip sonucu hayal etmek yerine, gerçek arayüzünüzün anında güncellendiğini siz tasarladıkça görürsünüz.

**Açmak için:** Ayarlar kenar çubuğundaki **Stil Tasarımcısı** (✦) sekmesine tıklayın; ya da simge yuvasında Ayarlar dişlisinin üstündeki **artı işareti (✛)** simgesine tıklayarak doğrudan inceleme moduna girin (uygulamanın herhangi bir parçasının üzerine gelin ve tıklayın, o öğenin denetimlerine atlarsınız). Panel yeniden boyutlandırılabilir.

Denetimler solda **kategorilere** ayrılmıştır: **Arayüz** (dosya ağacı, durum çubuğu, Evren paneli), **Bileşenler** (simge yuvası, araç çubukları, sekmeler, düğmeler, etiketler), **Editör** (not — gezinti yolu, başlıklar, bağlantılar, kod, alıntı ve notun rengi, yazı tipi ve boyutuyla ön izleme özeti), **Genel** (tonlar, vurgu rengi, köşeler, yazı sistemi başına yazı tipleri) ve **Bağlantılar** (bağlantı türü renkleri ve genişlikleri). Altta, tek tıklamayla uygulayabileceğiniz **kayıtlı stilleriniz** görünür.

**Değişikliklerinizi görmenin iki yolu.** **Editör** kategorisi **ortada bir not ön izlemesi** gösterir — bir başlığa, bağlantıya veya sayfaya tıklayın, denetimleri sağda görünür ve anında güncellenir. Diğer **tüm kategoriler** ise paneli bir kenara yaslayıp şeffaflaştırır, böylece değişiklikleriniz **doğrudan gerçek uygulamada** ön izlenir (bunu yeşil bir **● live** etiketi belirtir): durum çubuğunun rengini ya da simge yuvasının genişliğini değiştirin, siz sürükledikçe gerçek kabuğun stillendiğini görürsünüz.

**Bağlantılar.** Bağlantılar kategorisi her türü gerçek rengiyle bir **hap (pill)** olarak gösterir — yeniden renklendirmek için bir hapa tıklayın (her yerde anında uygulanır) — ayrıca **bağlantıları renklendir** ve **etiketleri göster** anahtarları, **hap şekli** denetimi ve yeniden kullanılabilir bir **kayıtlı renkler** paleti içerir.

**Görünümünüzü kaydedin.** Görünümü **bu Evren için** kaydetmek için **Keep** düğmesine basın (yeniden başlatmadan sonra da kalır); kaydedilmemiş değişiklikleri atmak ve uygulamayı eski haline döndürmek için **Discard** (ya da **✕** / **Esc**); sade temaya geri dönmek için **Reset**. Keep'e basana kadar hiçbir şey diske kaydedilmez.

**Bir görünümü yeniden kullanın — stil olarak kaydedin.** Listenin üstündeki "draft:" kutusuna bir ad yazıp **+ Save current as a style** düğmesine basın — görünüm **Kayıtlı Stiller** listesinde (sol altta) belirir, tüm Evrenler arasında geneldir ve yalnızca bir temayı değil, Tasarımcı'da oluşturduğunuz görünümü yakalar. Uygulamak için kayıtlı bir stile tıklayın; satırının üzerine gelerek **↻ güncelle**, **⤓ dışa aktar**, **✎ yeniden adlandır** ve **✕ sil** işlemlerini yapın. (Yerleşik temalar **Ayarlar → Görünüm**'de kalır; Tasarımcı ise kayıtlı stillerinizi ve canlı görünümü taşır.)

## Sık Sorulan Sorular

### Windows başlık çubuğunu ("Constellation v0.3.4 — …") stillendirebilir miyim?

Hayır — o çubuğu işletim sistemi (Windows/macOS/Linux) çizer. Constellation'ın ona CSS erişimi yoktur. Altındaki her şey tamamen stillendirilebilir.

### Kenar çubuğu genişliğini nasıl değiştiririm?

Kenar çubuğu genişliği, kenar çubuğunun kenarındaki **sürükleme tutamağı** ile kontrol edilir (yeniden boyutlandırmak için sürükleyin) — bir kaydırıcıyla değil. Genişlik kaydırıcısını Stil Tasarımcısı'ndan kasıtlı olarak kaldırdık, çünkü sürükleme tutamağını tekrarlıyordu (çakışan gerçek kaynaklarından kaçınmak için).

### Stilim nerede yaşıyor?

Stil Tasarımcısı'nda **Keep** ile kaydettikleriniz, Evren ayarları içinde **Evren başına** (Evren düzeyinde bir stil geçersiz kılması) saklanır, böylece Evren dizininizle birlikte yolculuk eder — cihazlar arasında senkronize ederseniz, stiliniz de birlikte gelir. **Kayıtlı Stiller** ise tüm Evrenler arasında geneldir.

### Bir temayı biriyle paylaşabilir miyim?

Evet:
- **Tam tema** — tema düzenleyicide **Dışa aktar** tıklayın. `.json` dosyasını paylaşın. Alıcı tema ızgarasında **↓ İçe aktar** tıklayıp onu seçer.
- **Kayıtlı stil** — **Stil Tasarımcısı**'nda kayıtlı bir stil satırının üzerine gelin ve **⤓ dışa aktar** tıklayıp bir `.constellation-style.json` dosyası olarak paylaşın (yalnızca görünüm — sır veya yol içermez). Alıcı, stil listesinde **Import…** tıklayarak ekler.

### İçe aktarılan bir Obsidian teması bozuk görünüyor. Şimdi ne olacak?

Obsidian temaları karmaşık olabilir. Bilinen durumlar:
- **HSL bölünmüş renkler** kullanan temalar (Minimal gibi) — bu sürümden itibaren Constellation'da desteklenir.
- Obsidian'ın belirli DOM yapısına bağımlı temalar kısmen görüntülenebilir. Constellation en yaygın seçicileri eşleyen bir sınıf shim'i içerir, ancak çok yapısal temalar beş ana rengi ayarlamayı veya telafi için Style Settings değerlerini elle ayarlamayı gerektirebilir.

## İlgili

- [[Universe]] — temaların ve Style Settings değerlerinin saklandığı yer
- [[Libraries]] — kitaplık başına renk vurguları (kitaplık ayarlarında ayarlanır, temalardan bağımsız)
- [[Importer]] — notları içe aktarmak için, tema değil (tema içe aktarma Görünüm altında)
