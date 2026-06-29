# Yapı

*(Kompozisyon omurgası — bu notun bütün yapıtın içinde durduğu yer)*

Constellation size zaten sekiz **düşünme bağlantısı** sunar — *destekler, çelişir, neden olur, örnekler, genelleştirir, türetilir, parçasıdır, yerini alır* — bir fikri başka bir fikirle ilişkilendirmek için kullandığınız sözcük dağarcığı. **Yapısal bağlantılar** bilinçli olarak farklı bir türdür. Onlar fikri fikre ilişkilendirmez; notlarınızdan inşa ettiğiniz bir **yapıtın sıralı biçimini** ortaya koyar: Kitap → Bölüm → Fasıl → Sahne, ya da herhangi bir İçerik Haritası taslağı. **Yapı** paneli, bu biçimi okuduğunuz yerdir.

Yapı'nın yanıtladığı tek soru şudur: **"Bu not, bütün yapıtın içinde nerede durur?"** — *"bu fikir şununla nasıl ilişkilidir"* değil. O ikinci soru Geri Bağlantılar ve Giden Bağlantılar panellerine aittir ve Yapı onların yoluna çıkmaz.

---

## Yapısal bağlantılar neden düşünmenizden ayrı tutulur

Yapısal bir yerleştirme **yazarlıktır, yargılanacak bir iddia değildir**. Bir sahneyi bir faslın altına ya da bir faslı bir kitabın altına koymak, *taslağınızın biçimi* hakkında bir karardır — kanıt değildir, argüman değildir, çelişilebilecek ya da zamanla daha kesin hâle gelebilecek bir şey değildir.

Bu yüzden yapısal bağlantılar bilinçli olarak her düşünme, olgunluk ve bağlantı ölçüsüne görünmezdir:

- Bir notun geri bağlantılarında veya giden bağlantılarında bağlantı olarak **sayılmazlar**.
- Bir notun olgunluğunu **yükseltmezler**.
- Sky View'da veya grafikte **görünmezler**.

Bir içindekiler tablosu, bir notu olduğundan daha "bağlantılı" göstermemelidir. Düşünme bağlantılarınız ve taslağınızın yapısı iki ayrı şeydir ve Constellation onları öyle tutar.

---

## İki tür — yalnızca tek tarafı yazarsınız

Yapıyı, hangi uçtan uygunsa o uçtan beyan edersiniz, Constellation tersini sizin için çözer. İki ucu birden korumak zorunda kalmazsınız.

| Özellik | Anlamı |
|---|---|
| **`parent`** | *Bu notun* tek bir ebeveynin altındaki yeri. (Bir fasıl, ait olduğu bölümü söyler.) |
| **`contains`** | *Bu notun* sıralı çocuk listesi. (Bir kitap, bölümlerini okuma sırasıyla listeler.) |

Bir çocuğun `parent` değerini beyan etmek ile onu bir `contains` listesinde sıralamak, aynı şeyi söylemenin iki yoludur. Düşünme biçiminize uyanı kullanın — yukarıdan aşağıya (bölümlerini *içeren (contains)* bir kitap) ya da aşağıdan yukarıya (*ebeveynini (parent)* adlandıran bir fasıl).

---

## Yapısal bir bağlantı oluşturma — adım adım

Yapıyı bir notun **Özellikler** bölümünde oluşturursunuz — sağ kenar çubuğundaki Özellikler sekmesi veya notun üstündeki özellikler bloğu.

1. **+ Özellik ekle**'ye tıklayın.
2. Anahtar için **`parent`** ya da **`contains`** yazın.
3. Değer kısmına **hedef notun adını** yazın — yalnızca adı, örneğin `Part I - The Cartographer`. **Köşeli parantezleri siz yazmazsınız.** Constellation adı sizin için otomatik olarak bir `[[link]]` içine sarar. (Zaten parantezli bir ad yapıştırırsanız, tek bir `[[ad]]` olacak şekilde temizlenir — asla çift `[[[ ]]]` olmaz.)
4. **`contains`** için her çocuğu kendi çipi olarak ekleyin — bir ad yazın, Enter'a basın, sonrakini yazın. **Onları eklediğiniz sıra, taslağın okuma sırasıdır.**

> **Güvenle yeniden adlandırılırlar.** Bir faslı yeniden adlandırın, yapıdaki yeri otomatik olarak takip eder — bağlantı dondurulmuş bir metin parçasına değil, notun kendisine işaret eder. Yeniden adlandırmadan sonra bir taslağı asla bulup düzeltmek zorunda kalmazsınız.

---

## Yapı panelini okuma

Sağ kenar çubuğundaki **Yapı** sekmesini açın — Geri Bağlantılar sekmesinin hemen ardından.

- **Taslak.** Bir sayımla birlikte **TASLAK (OUTLINE)** başlığı altında panel, **bütün yapıtı** teal renkli madde imli, girintili bir ağaç olarak gösterir — yapıtın her alt öğesi, sırayla — yalnızca açık notun kendi çocukları değil. Böylece tek bir sahnenin üzerinde dururken bile, çevresindeki bütün kitabı görürsünüz.
- **"Buradasınız."** Şu anda görüntülemekte olduğunuz not, taslağın içinde **vurgulanır**, böylece nerede durduğunuzu her zaman bilirsiniz.
- **Kırılma yolu (breadcrumb).** Üst kısım boyunca, teal renkli bir kırılma yolu omurga boyunca yukarı giden yolu gösterir — örneğin *The Atlas of Lost Places › Part I › Chapter 1*. Doğrudan o nota atlamak için herhangi bir kırıntıya (veya taslaktaki herhangi bir satıra) tıklayın.
- **Bütün yapıt ⇄ Bu not.** Sağ üstteki bir anahtar, bütün yapıt ile yalnızca açık notun kendi dalı arasında geçiş yapar. Yalnızca notun bir ebeveyni olduğunda görünür (aksi hâlde iki görünüm aynı olurdu).

> **Bir döngü onu asla takmaz.** Yapı yanlışlıkla kendi üzerine dönerse — A notunun ebeveyni B ve B'nin ebeveyni A ise — taslak zinciri çizer ve sonra temiz biçimde durur, kesim noktasını küçük bir **↻** ile işaretler. Tek satırlık bir açıklama için üzerine gelin.

---

## İki not aynı çocuğu sahiplendiğinde — "İtiraz edilen"

Yapı'nın temiz bir ağaç olması amaçlanır, bu yüzden bir çocuğun tam olarak bir ebeveyni olmalıdır. İki not da aynı çocuğu sahiplenirse — biri çocuğun kendi **`parent`** özelliği üzerinden, diğeri onun **`contains`** listesi üzerinden — Constellation sessizce birini seçip diğerini **bırakmaz**. Bunun yerine o satır, diğer sahiplenen tarafı adlandıran amber renkli bir **⚠** rozetiyle **İtiraz edilen (Contested)** olarak işaretlenir, böylece çatışmayı görüp karar verebilirsiniz.

İki tek tıklamalık düğme bunu çözer:

- **Tut (Keep)** — çocuğun kendi beyan ettiği ebeveyni koru. (Bu not, çocuk üzerindeki iddiasından vazgeçer.)
- **Buraya taşı (Move here)** — bu notu ebeveyn olarak kabul et. (Çocuğun `parent` değeri bu nota geçer.)

Her iki tercih de not dosyalarını doğrudan günceller ve taslağı yeniler. **Sizin tıklamanız olmadan hiçbir şey asla değiştirilmez** — Constellation çatışmayı işaretler ve kararınızı bekler.

---

## Bilmekte fayda var

- **Yerel ve özel.** Taslak, kendi notlarınızdan istek üzerine okunur; hiçbir yere hiçbir şey gönderilmez.
- **Büyük yapıtlarda hızlı.** Uzun taslaklar (yaklaşık 50 satırı geçenler) kendi kaydırma çubuğunu alır ve yalnızca ekrandaki satırları işler, böylece büyük bir taslak sorunsuz açılır ve kaydırılır.
- **Dilinizi konuşur.** Panelin etiketleri, kırılma yolu ve çözme düğmelerinin tümü seçtiğiniz arayüz dilinde görünür ve sağdan sola diller için doğru biçimde yansıtılır. `parent` / `contains` özellik *anahtarları* dosyada standart İngilizce kalır (böylece yapı her dilde aynı okunur), ekrandaki hap (pill) etiketleri ise yerelleştirilir.
