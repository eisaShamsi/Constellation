# Arapça Motor

Constellation, Arapça metni bu uygulama için sıfırdan inşa edilmiş beş katmanlı bir biçimbilim motoruyla çözümler. Mevcut bir kök bulucunun (stemmer) uyarlaması değildir — Arapça köklerini, örüntülerini, özel adlarını, ödünç sözcüklerini ve kendi terminolojinizi anlayan yerel bir enstrümandır. Motorun kendisini hiçbir zaman yapılandırmazsınız; her aramanın, her bağlantının, her dizin girdisinin altında sessizce çalışır. Yapılandırabildiğiniz — ve bu yardım konusunun ele aldığı — tek şey, motorun sizin muhakemenizi davet ettiği tek yerdir: Ayarlar'daki **Arapça Motor Geçersiz Kılmaları** paneli.

---

## Motor neden var

Arapça örüntüsel (templatic) bir dildir. ك‑ت‑ب ("yazmak") gibi tek bir kök onlarca yüzey biçimi üretir — كاتب (yazar), مكتوب (yazılmış), كتاب (kitap), يكتب (yazıyor), كتبنا (yazdık) — hepsi de arama sırasında aynı anlamsal çekirdeğe inmelidir. Saf bir kök bulucu ya bu biçimleri bozar (örneğin وائل sözcüğünü aşırı keserek ائل'e çevirir) ya da aralarındaki bağı hiç yakalayamaz. Constellation'ın motoru bu iki başarısızlığı da, her Arapça sözcüğü kesin bir öncelik sırasına göre beş katmandan geçirerek önler:

1. **Katman 0 — Kullanıcı Geçersiz Kılmaları** (kontrolünüz altındaki katman)
2. **Katman 2 — Korumalı Liste** (asla dokunulmaması gereken ~1.200 elle derlenmiş özel ad, yer adı, ödünç sözcük ve işlev sözcüğü)
3. **Katman 3 — Üretici FST** (~7.000 kökü × 158 örüntüyü tam yüzey sözvarlığına eşleyen derlenmiş bir sonlu-durum dönüştürücü)
4. **Katman 3b — Kaskad** (fonolojik onarımlar: benzeşme, zayıf kökler, hemze yerleşimi)
5. **Katman 5 — Sezgisel** (zarif yedek — yalnızca diğer tüm katmanlar yanıt vermeyi reddettiğinde devreye giren tutucu bir ek sökücü)

Bir sıralama adımı (Katman 4), birden fazla katman okuma ürettiğinde tek bir en iyi çözümlemeyi seçer. Sıralama, sizin geçersiz kılmalarınızı her şeyin üstüne koyar.

---

## Özellik: Arapça Motor Geçersiz Kılmaları

### Nedir

Geçersiz kılma paneli, Ayarlar'da motora belirli Arapça yüzey biçimlerini kendi sözcüklerinizle nasıl çözümleyeceğini söylediğiniz küçük bir tablodur. Her geçersiz kılmada şunlar vardır:

- **Yüzey biçimi** — Arapça sözcüğü yazdığınız şekliyle (örn. وائل).
- **Lemma** — motorun döndürmesi gereken kanonik biçim (örn. وائل).
- **Kök** — isteğe bağlı. Sözcüğün klasik bir kökü varsa üç veya dört ünsüz.
- **Örüntü** — isteğe bağlı. Biçimbilim şablonunu kaydetmek isterseniz serbest metin etiketi (örn. `فاعل`).
- **Söz türü** — Özel ad / Ad / Sıfat / Zarf / Fiil / İlgeç / Yabancı / Bilinmiyor.
- **Not** — isteğe bağlı. Geleceğiniz için bir bağlam satırı.

### Neden önemlidir

Her bilgi ağında motorun bir sözlükten bilemeyeceği terimler vardır: kendi türettiğiniz sözcükler, yerel kasabanızdan isimler, alanınızdaki kısaltmalar, meslektaşlarınızın belirli bir şekilde yazılmasını tercih ettiği ödünç sözcükler. Geçersiz kılmalar olmadan motor bu yüzey biçimlerine genel çözümlemesini uygular ve arama sonuçlarınız küçük varyasyonlar etrafında parçalanır. Geçersiz kılma egemen yanıttır — üretici FST, kaskad ve sezgisel yedeği geçer. Katman 4'ün sıralaması geçersiz kılmalara en üst köken ve 1.0 güven değeri verir, bu nedenle başka bir çözümleme lehine asla atılmazlar.

Geçersiz kılmalar, `<Evreniniz>/.constellation/arabic-overrides.json` yolunda tek bir JSON dosyasında yaşar. Dosya düz metindir, alfabetik sıralıdır ve atomik olarak yazılır (bir `.tmp` + yeniden adlandırma çiftiyle), böylece düzenleme sırasında bir güç kaybı onu bozamaz. O sizindir — sürüm kontrolüne alabilir, diff'ini görebilir veya cihazlar arasında paylaşabilirsiniz.

### Nasıl kullanılır

**Adım 1: Paneli açın**

Ayarları açmak için sağ üst araç çubuğundaki dişli simgesine tıklayın (veya `Ctrl + ,` / `Cmd + ,` tuşlarına basın). Sol kenar çubuğunda **Arapça Geçersiz Kılmalar** öğesini seçin — **Dil** öğesinin yanında bulunur. Görünmüyorsa kenar çubuğunu kaydırın.

**Adım 2: İlk geçersiz kılmanızı ekleyin**

**Geçersiz kılma ekle** düğmesine tıklayın. Altı alanlı bir form açılır (yüzey biçimi, lemma, kök, örüntü, söz türü, not). Yüzey biçimini notlarınızda yazdığınız şekilde girin — motor diyakritikleri ve elif varyantlarını dahili olarak normalleştirir, bu yüzden bunları tam olarak eşleştirme konusunda endişelenmenize gerek yoktur. İstediğiniz lemmayı doldurun. Kökü ve örüntüyü bilmiyorsanız boş bırakın; motor geçersiz kılmayı yine de kullanır. Açılır listeden bir söz türü seçin veya **Bilinmiyor** olarak bırakın. **Kaydet** düğmesine tıklayın.

**Adım 3: Yeniden dizinleme bildirimini izleyin**

Kaydettiğiniz anda panel **Yeniden dizinleniyor…** gösterir ve motor, aktif Evrendeki metni bu yüzey biçimini içeren her notu tarar. Eşleşen her not, yeni geçersiz kılma kararı altında yeniden belirteçlenir. Tarama bittiğinde — tipik bir Evrende genellikle bir saniye içinde — bildirim **N not yeniden dizinlendi** olarak değişir ve üç saniye sonra otomatik temizlenir. Uygulamayı yeniden başlatmanıza gerek yoktur ve hiçbir dizini yeniden oluşturmanıza gerek yoktur.

**Adım 4: Aramada doğrulayın**

Arama merkezini açın (`Ctrl + K` / `Cmd + K`) ve yüzey biçimini yazın. Eşleşmeler artık belirttiğiniz lemmayı yansıtmalıdır: lemma için yapılan sorgular yüzey biçimini bulur ve yüzey biçimi için yapılan sorgular lemmanın diğer çekimli biçimlerini bulur.

**Adım 5: Bir geçersiz kılmayı kaldırın**

Geçersiz kılmanın satırındaki **×** düğmesine tıklayın. Girdi diskten hemen kaldırılır ve aynı yeniden dizinleme taraması tersine çalışır — yüzey biçimini içeren notlar motorun genel çözümlemesi altında yeniden belirteçlenir. Bildirim kaç notun etkilendiğini bildirir.

### Korumalı Liste ile etkileşim

Korumalı Liste (Katman 2) zaten asla sökülmemesi gereken ~1.200 yaygın yüzey biçimi içerir — وائل gibi isimler, فلسطين gibi yerler, إنترنت gibi ödünç sözcükler. Bunları kendiniz eklemenize gerek yoktur; motor bunlarla birlikte gelir. Geçersiz Kılmalar panelini Evreninize *kişisel* olan yüzey biçimleri için kullanın — kendi terminolojiniz, yerel adlar, alana özgü ödünç sözcükler veya motorun otomatik okumasıyla aynı fikirde olmadığınız durumlar.

### Evrenler arası etkileşim

Her Evrenin kendi geçersiz kılma dosyası vardır. Evrenleri değiştirmek bellekteki aktif geçersiz kılma kümesini değiştirir — motor, yeni Evrenin `.constellation/` klasöründen JSON'u yeniden yükler. Dosya eksikse (yeni bir Evren), motor geçersiz kılma kümesini boş kabul eder. Dosya bozuksa, motor bir uyarı kaydeder ve yüklemeyi reddetmek yerine boş bir kümeye geri döner.

### Dosyayı elle düzenlerseniz ne olur

Düzenleyebilirsiniz. Dosya biçimi şudur:

```json
[
  {
    "surface": "وائل",
    "lemma": "وائل",
    "root": null,
    "pattern": null,
    "pos": "ProperNoun",
    "note": "Personal name — never strip"
  }
]
```

Git dostu diff'ler için girdileri yüzey biçimine göre alfabetik olarak sıralı tutun. Motor her kaydetmede yeniden sıralar, bu nedenle elle yapılan yeniden sıralamalar arayüz üzerinden yapılan bir düzenlemeden sonra korunmaz.

---

## Sözlük

- **Yüzey biçimi** — yazıldığı haliyle Arapça sözcük; eklenmiş her türlü klitik dahil (örn. الكتاب, بالكتاب, كتبنا).
- **Lemma** — çekimden arındırılmış alıntı biçimi (örn. كتاب).
- **Kök** — bir sözcük ailesinin paylaştığı 3 veya 4 ünsüzlü anlamsal çekirdek (örn. ك‑ت‑ب).
- **Örüntü** — bir kökle birleşerek bir yüzey biçimi üreten sesli-ek şablonu (örn. فاعل → كاتب).
- **FST** — sonlu-durum dönüştürücüsü. Motor, kökleri × örüntüleri tam yüzey sözvarlığına verimli biçimde eşlemek için bir tane kullanır.
- **Kaskad** — benzeşme, zayıf ünsüzler ve hemze yerleşimini ele alan fonolojik onarım katmanı.
- **Geçersiz kılma** — belirli bir yüzey biçiminin nasıl çözümlenmesi gerektiğine dair kendi kararınız; diğer tüm katmanları geçer.
