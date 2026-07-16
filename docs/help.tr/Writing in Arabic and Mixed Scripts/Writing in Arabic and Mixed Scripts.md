# Arapça ve Karma Yazı Sistemleriyle Yazma

Constellation'ın düzenleyicisi dil-öncelikli olarak inşa edilmiştir: Arapça, İbranice, Farsça, Urduca ve iki dilli notlar sonradan eklenmiş bir özellik değildir — imleç, seçim ve her paragrafın yönü, Microsoft Word'ün Windows'ta kullandığı kuralların aynısını izler; böylece kas hafızanız olduğu gibi taşınır. Bu konu, sağdan sola ve karma metinlerde *yazmayla* ilgili her şeyi kapsar: imlecin nasıl hareket ettiği; sözcük, cümle, satır, paragraf veya ekran bazında nasıl seçim yapılacağı; ve otomatik algılama istediğiniz gibi olmadığında bir paragrafın yönünü nasıl zorlayacağınız.

(Constellation'ın Arapçayı nasıl *anladığı* — kökler, arama ve biçimbilim motoru — için **Arapça Motor** konusuna bakın.)

---

## İmleç nasıl hareket eder

- **Ok tuşları, ekrandaki konuma göre değil, metnin okunuş sırasına göre birer karakter ilerler.** Salt Arapça veya salt İngilizce metinde bu, tam olarak bastığınız ok gibi görünür. Arapça ile İngilizcenin buluştuğu bir ek yerinde (örneğin içinde İngilizce bir sözcük geçen Arapça bir cümlede) imleç, karakterleri yazılış sırasıyla tek tek adımlar ve ek yerinin üzerinden gözle görülür biçimde "atlar" — bu atlama doğrudur; imlecin sınırda takılıp kalmış gibi hissetmesini önleyen şey tam da budur.
- **Home**, satırın okuma açısından **başına** gider — Arapça bir satırın *sağ* kenarına. **End** ise okuma açısından **sonuna** gider — *sol* kenara. O kenara kadar seçim yapmak için ikisinden birine **Shift** ile birlikte basın.
- Arapça bir satırda **Enter**, yeni satırın imlecini **sağa** yerleştirir — doğal yazma konumuna.
- **Arapça bir satırın sonundaki Latin harfli bir sözcük**, yönünü kaybetmek yerine net ve kararlı bir imleç konumu korur.

Yukarıdaki kuralların tümü standart düzenleyicide, Odak modunda ve çakışma birleştirme görünümünde birebir aynı çalışır.

---

## Birim bazında seçim

Her metin biriminin, her dilde ve her karışımda hızlı bir seçicisi vardır:

| Birim | Nasıl |
|---|---|
| **Sözcük** | Üzerine çift tıklayın |
| **Cümle** | Herhangi bir yerine **Ctrl+tıklama** — veya imleç içindeyken **Ctrl+Shift+S** |
| **Satır** | **Ctrl+L** |
| **Paragraf** | **Ctrl+Shift+L** — veya üçlü tıklama |
| **Bir ekran dolusu** | **Shift+Page Down** / **Shift+Page Up** |
| **Tümü** | **Ctrl+A** |

Bilinmeye değer ayrıntılar:

- **Cümle seçimi Arapça noktalama işaretlerini anlar.** Cümleyi **؟ ۔ !** işaretlerinde ve noktada bitirir — ama Arapça noktalı virgül **؛** cümlenin *içindeki* bir duraklamadır, dolayısıyla seçim onu doğru biçimde aşıp devam eder. 3.14 gibi ondalık sayılar hiçbir zaman cümleyi bölmez.
- **Paragraf**, üstünde ve altında boş satır bulunan bir metin bloğudur — tıpkı Word'deki gibi. Satır ve paragraf seçimleri metne yapışır: Arapça bir satırda vurgu, boş sol tarafa uzanmak yerine sözcüklerde durur.
- Ctrl+tıklama, o tuştaki eski "bir imleç daha ekle" hareketinin *yerini alır* — artık bu tıklamanın yaptığı şey cümle seçimidir.

## Paragraf bazında hareket

- **Ctrl+↓**, **sonraki** paragrafın başına atlar; **Ctrl+↑** ise **geçerli** paragrafın başına (bir öncekine gitmek için yeniden basın). Atlarken paragraf paragraf seçmek için **Shift** ekleyin. Bu, Word'ün alışılmış davranışıdır; "sonraki" de yalnızca sayfada daha aşağısı demektir — Arapça, İngilizce ve karma notlarda birebir aynı çalışır.

---

## Paragraf yönünü zorlama

Constellation her satırın yönünü ilk harflerinden otomatik olarak algılar. Bu genellikle tam isabetlidir — ama bazen kararı kendiniz vermek istersiniz: İngilizce bir marka adıyla açılan Arapça bir paragraf ya da sağdan sola okunmasını istediğiniz, çoğunlukla İngilizce bir paragraf.

**Klavyenizin SAĞ tarafındaki Ctrl+Shift tuşlarına basıp bırakın** → imlecin bulunduğu paragraf **%100 sağdan sola** olur.
**SOL taraftaki Ctrl+Shift tuşlarına basıp bırakın** → **%100 soldan sağa**.

Bu, Microsoft Word'ün alışılmış davranışıdır. Bilinmesi gerekenler:

- **Tuşları bıraktığınız anda devreye girer** — iki tuşa birlikte basın, bırakın ve arada başka hiçbir tuşa basmayın. Ctrl+Shift+S, Ctrl+Shift+L ve diğer tüm kısayolların normal çalışmaya devam etmesinin nedeni budur: araya üçüncü bir tuş girdiği anda yön değiştirme geri çekilir.
- **Kesin bir geçersiz kılmadır** — otomatik algılamayı yener ve paragrafın tamamına (veya bir seçimin dokunduğu her paragrafa) uygulanır.
- **Metnin kendi içine kaydedilir** — görünmez bir yön karakteri olarak. Bu sayede notu kapatmaya, uygulamayı yeniden başlatmaya ve eşitlemeye dayanır — hatta metni Word'e veya Obsidian'a yapıştırdığınızda metinle birlikte taşınır.
- **Tek bir Ctrl+Z geri alır.** Aynı tarafa iki kez basmak fazladan bir şey yapmaz.
- **Markdown güvende kalır.** Listeler liste, başlıklar başlık, alıntılar alıntı olarak kalır. Kod bloklarına, tablolara ve yatay çizgilere bilerek dokunulmaz. Bir #etiketle *başlayan* satır otomatik yönünü korur (oraya zorlanmış bir işaret etiketi bozardı) — paragrafın geri kalanı yine de döner.

---

## Yazı tipleri ve arayüz

- **Yazı tipi betikleri**: Arapça, İbranice ve CJK yazı tiplerini **Ayarlar → Dil** bölümünden bağımsız olarak yapılandırın.
- **Betik araç çubukları**: dile özgü simge ve noktalama düğmeleri.
- **Hareke vurgulama**: Arapça harekelerin vurgulanmasını düzenleyici araç çubuğundan açıp kapatın.
- Arayüz dili olarak Arapça veya İbranice seçildiğinde uygulamanın tamamı sağdan sola döner.

---

## Sözlükçe

- **Okunuş sırası** — karakterlerin, ekranda nerede durduklarından bağımsız olarak, yazıldıkları ve okundukları sıra.
- **Ek yeri** — aynı satırdaki sağdan sola bir metin parçası ile soldan sağa bir metin parçası arasındaki sınır.
- **Kesin geçersiz kılma** — sizin belirlediğiniz ve otomatik ilk-harf algılamasını yenen açık yön.
- **Yön işareti** — geçersiz kılmanızı metnin kendi içinde saklayan görünmez karakter (RLM/LRM).
