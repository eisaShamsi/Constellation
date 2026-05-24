---
translation_status: AI-generated 2026-05-21 — native-speaker review recommended
language: tr
source: docs/help.uConstellation.World/Note Summaries/Note Summaries.md
aliases:
  - Note Summaries
  - Note Summary
  - Summary
  - NSC
  - Note Summary Creator
  - Build all summaries
  - Not Özetleri
  - Not Özeti
  - Özet
description: Not Özetleri, bir notu açmadan hakkında yargıya varabilmeniz için size kısa, sade dilde bir özet verir. Constellation, kendiniz yazdığınız bir özeti — frontmatter'da ya da bir özet çağrı kutusunda — her zaman onurlandırır ve yalnızca yazmadığınızda bir tane üretir. Üretilmiş özetler çıkarımsaldır (notun kendi en merkezi cümleleri), salt okunurdur (dosyanıza asla geri yazılmaz) ve tamamen cihazınızda hesaplanır. Özetler, bir notun göründüğü her yerde uygulama boyunca görünür — **Sınıflandırıcı**, **Arama sonuçları**, **Düzenleyici** şeridi, **Geri Bağlantılar** paneli, **Giden Bağlantılar** paneli, **Dizin**, **Sky View** hover ipucu ve **Evren Özeti**.
---

# Not Özetleri

> *Bir özet yazdıysanız, Constellation sizinkini kullanır. Yalnızca yazmadığınızda bir tane yazar — ve o zaman bile, asla dosyanıza değil.*

Bir **Not Özeti**, bir notun kısa bir özetidir — notun bir bakışta ne hakkında olduğunu söyleyen birkaç cümle. Özetler, **Note Summary Creator (NSC)** tarafından üretilir. Onları **bir notun göründüğü her yerde, uygulamanın tamamında** göreceksiniz: **Sınıflandırıcı** / **Kaynak Gözden Geçirme** kuyruğundaki her kartın başlığının altında (arkasındaki notu açmadan bir kartı nasıl sınıflandıracağınıza karar verdiğiniz yer), **Arama sonuçlarındaki** her sonucun altında soluk italik bir satır olarak (size bir notun ne *hakkında* olduğunu, neden eşleştiğini gösteren parçacıkla birlikte söyler), notu **Düzenleyici**'de açtığınızda notun üstünde ince bir şerit olarak (okuduğunuz ya da yazdığınız sırada notun özü bağlam içinde kalsın diye), **Geri Bağlantılar** panelinde bağlanan her kaynağın altında ve **Giden Bağlantılar** panelinde bağlanan her hedefin altında (ilgili notların uzun bir listesi, yalnızca başlıklar yerine fikirler olarak taransın diye), **Dizin**'de bir terimi genişlettiğinizde her not bahsinin altında (bir terimin notları bir bakışta tanınabilir olsun diye), **Sky View** hover ipucunun içinde bir baloncuğa işaret ettiğinizde (yoğun bir grafiğin tıklamaya gerek kalmadan okunabilir kalması için) ve yeni **Evren Özeti** panelinin her satırının *birincil içeriği* olarak (tüm bilgi tabanının özetlerin kaydırılabilir bir listesi olduğu yer — *Özet*'e bakın).

Bu konu, özetlerin nereden geldiğini, *sizin* sözlerinizi her zaman makineninkine tercih eden katı öncelik sırasını, üretilmiş özetlerin nasıl inşa edildiğini ve bunları bir Kütüphanenin tamamı için tek seferde nasıl önceden hesaplayacağınızı açıklar.

---

## Özetler neden var

Yüzlerce kartlık bir inceleme kuyruğunda çalışırken, başlık tek başına çoğu zaman bir notun gerçekte ne dediğini hatırlamaya yetmez. Belleğinizi tazelemek için her notu açmak akışınızı bozar. Başlığın altındaki tek bakışlık bir özet bunu düzeltir: üç cümle okursunuz, notu hatırlarsınız, kararı verirsiniz, devam edersiniz.

Ama bir özet aynı zamanda küçük bir yazarlık eylemidir. Bir notu zaten kendi sözlerinizle damıttıysanız — bir `summary:` alanında ya da bir `> [!summary]` çağrı kutusunda — o zaman gösterilmesi gereken özet *odur*, makinenin tahmini değil. Bu yüzden Constellation'ın özetler için ilk kuralı, yazınıza saygı hakkında bir kuraldır: **sizinki kazanır.**

---

## Bir özet nereden gelir — öncelik sırası

Constellation, herhangi bir not için özeti bu listeyi inerek ve var olan ilkinde durarak seçer:

1. **Frontmatter özetiniz.** Notun özellikleri bir `summary:`, `description:`, `abstract:` ya da `excerpt:` alanı içeriyorsa (bu sırayla denetlenir), metni **tam olarak yazdığınız gibi** kullanılır.
2. **Özet çağrı kutunuz.** Not gövdesi bir `> [!summary]`, `> [!abstract]` ya da `> [!tldr]` çağrı kutusu içeriyorsa, metni **tam olarak yazdığınız gibi** — aksan işaretleri ve noktalama dâhil, harfi harfine korunarak — kullanılır.
3. **Üretilmiş bir özet.** Yalnızca yukarıdakilerin hiçbirini yazmadıysanız Constellation bir tane üretir — notu okuyarak ve en merkezi cümlelerini çıkararak (aşağıya bakın).
4. **Açılış metni yedeği.** Motorun cümlelere bölemediği bir not için (örneğin, net cümle noktalaması olmayan bir yazıdaki metin), sıralanmış bir özet yerine notun açılış satırlarını gösterir.

> **En çok önemli olan tek kural:** 1. ve 2. adımlar, yazdığınız bir özetin *asla* üzerine yazılmadığı anlamına gelir. Özetlediğinizi düşündüğünüz bir notta üretilmiş bir özet görürseniz, bu motorun aradığı yerde özetinizi bulamadığı anlamına gelir — frontmatter alanınızın yukarıdaki dört addan biri olduğunu ya da çağrı kutunuzun yukarıdaki üç türden biri olduğunu kontrol edin.

---

## Üretilmiş bir özet nasıl inşa edilir

Constellation bir özet üretmek zorunda kaldığında (siz bir tane yazmadığınız için), **çıkarımsal** özetleme yapar — yeni düzyazı icat etmek yerine notunuzda zaten var olan cümleleri seçer. Yöntem köklü bir yöntemdir (TextRank, Mihalcea & Tarau 2004):

1. **Cümlelere böl.** Not gövdesi, cümle sınırları için Unicode standardı kullanılarak cümlelere bölümlenir, böylece diller ve yazılar arasında çalışır.
2. **Her cümlenin anlamını oku.** Her cümle, kompakt bir cihaz-üstü model kullanılarak küçük sayısal bir "anlam parmak izine" (bir gömme) dönüştürülür.
3. **Merkeziyete göre sırala.** Anlamca *diğer* cümlelerin en çoğuna en benzer olan cümleler en yüksek puanı alır — bunlar notu bütün olarak en iyi temsil eden cümlelerdir.
4. **İlk üçünü sırasıyla al.** En yüksek sıralanan üç cümle, **notta göründükleri sırayla** gösterilir, böylece özet sırasız değil, doğal olarak okunur.

Çok uzun notlar nazikçe işlenir — motor gövdenin ne kadarını taradığını ve kaç cümleyi sıraladığını sınırlar, böylece kocaman bir notu özetlemek uygulamayı asla yavaşlatmaz ya da bir çökme riski yaratmaz.

Çıkarımsal olduğu için, üretilmiş bir özet her zaman gerçekten yazdığınız cümlelerden oluşur. Ağzınıza asla laf koymaz.

---

## Özetler salt okunurdur — File-Over-App

Constellation **üretilmiş bir özeti notunuza asla geri yazmaz.** `.md` dosyalarınız doğruluğun kaynağıdır; bir kartta gördüğünüz özet anında hesaplanır ve dosyanın metnine ya da frontmatter'ına kaydedilmez, ayrı olarak önbelleğe alınır.

Bu kasıtlıdır ve Constellation'ın *File-Over-App* ilkesini izler: uygulama dosyalarınıza açılan bir penceredir, onları sessizce değiştiren bir düzenleyici değil. Bir özetin notun *içinde* yaşamasını istiyorsanız, kendiniz bir tane yazın (bir `summary:` alanı ya da bir `[!summary]` çağrı kutusu) — ve sonra, yukarıdaki öncelik kuralına göre, Constellation sizinkini gösterir ve üretmeyi durdurur.

Her şey **cihazınızda** hesaplanır. Hiçbir not metni özetlenmek üzere hiçbir yere asla gönderilmez.

---

## Özetler nerede görünür ve nasıl dolar

Özetler, Constellation boyunca bir notun göründüğü her yerde belirir:

- **Sınıflandırıcı / Kaynak Gözden Geçirme kuyruğu** — her kartın başlığının altında (özgün yüzey — *Sınıflandırıcı*'ya bakın).
- **Arama sonuçları** — her sonucun altında, parçacığın altında soluk italik bir satır. Parçacık bir sonucun sorgunuzla *neden* eşleştiğini gösterir; özet satırı notun ne *hakkında* olduğunu gösterir. Birlikte, hiçbir şeyi açmadan sonuçları taramanıza olanak tanırlar.
- **Düzenleyici** — bir notu açtığınızda not gövdesinin üstünde ince ve sönük bir şerit, okuduğunuz ya da yazdığınız sırada notun özü bağlam içinde kalsın diye. Henüz bir özet olmadığında (yepyeni bir not ya da özeti hâlâ hesaplanmakta olan bir not), şerit kendini gizler.
- **Geri Bağlantılar paneli** — okumakta olduğunuz nota bağlantı veren her kaynak satırının altında. Uzun bir gelen bahis listesi taranabilir hale gelir: bağlantı veren notun özünü başlığının altında tek bir italik satır olarak okursunuz, her birini ne olduğunu hatırlamak için açmak zorunda kalmazsınız.
- **Giden Bağlantılar paneli** — okumakta olduğunuz notun dışarıya bağlandığı her hedef satırının altında. Geri Bağlantılarla aynı biçim; her bir giden bağlantının ne hakkında olduğunu bir bakışta görürsünüz.
- **Dizin** — bir terimi genişlettiğinizde, terimi kullanan her not özeti başlığının altında soluk bir satır olarak gösterir (ve mevcutsa eşleşen bağlam parçacığının altında da). Onlarca notta görünen bir terim, yalnızca dosya adlarının değil fikirlerin bir listesi hâline gelir.
- **Sky View hover** — **Sky View** grafiğinde bir baloncuğun üzerine geldiğinizde, yüzen ipucu ilk satırda notun adını ve ikinci italik satırda özet başlığını gösterir, böylece bir baloncuğun ne *anlama geldiğini* grafikten ayrılmadan okuyabilirsiniz.
- **Evren Özeti** — özetlerin yaşadığı *birincil* yer: sol dock'ta Evrenizdeki her notu listeleyen özel bir panel (kademeli Library → Klasör → Not), her satır adın altında başlığı gösterir. Bir satıra tıklayın, genişlesin ve çok cümleli tam özeti satır içinde okuyun. Arama/filtre tüm listeyi daraltır. Tam konu için *Özet*'e bakın.

Varsayılan olarak özetler **tembelce ve nazikçe** dolar: kartlar görünüme kaydıkça, arama sonuçları belirdikçe, bir notu açtıkça, bir terimi genişlettikçe, bir baloncuğun üzerine geldikçe veya Özeti kaydırdıkça, Constellation eksik özetleri birkaçı bir arada hesaplar, bir Kütüphane sınıflandırma taraması çalışırken duraklayarak ikisinin kaynaklar için asla yarışmamasını sağlar. Bu, uygulamayı duyarlı tutar — bir kartı / sonucu / açılmış bir notu / satırı / ipucunu / Özet satırını özetinden önce kısaca görebilirsiniz, sonra özet bir an sonra belirir.

Her özetin önceden hazır olmasını tercih ederseniz — her yüzeyin özetleri anında göstermesi için — **Tüm özetleri oluştur**'u kullanın.

---

## Tüm özetleri oluştur — tüm Kütüphaneyi önceden hesaplama

**Tüm özetleri oluştur** düğmesi (**Sınıflandırıcı** başlığında), kartların kaydırdıkça doldurmak yerine özetlerini anında göstermesi için **henüz güncel bir özeti olmayan her not** için bir özet önceden hesaplar.

**Kullanmak için:**

1. **Sınıflandırıcıyı** açın (sol dock'taki istiflenmiş kartlar simgesi).
2. Başlıkta **Tüm özetleri oluştur**'a tıklayın. Düğme *Not özetleri oluşturuluyor…* olarak değişir.
3. İlerleme pencerenin altındaki **durum çubuğunda** görünür — çalışırken çalışmaya devam edebilirsiniz.
4. Erken durdurmak için durum çubuğu ilerleme şeridindeki **İptal** kontrolünü kullanın. Kısmi bir çalışma sorun değildir; bir sonraki sefer kaldığı yerden devam eder.

Bilinmeye değer birkaç şey:

- **Yalnızca siz istediğinizde** çalışır — kendiliğinden asla başlamaz, bu yüzden uygulama başlangıcını asla yavaşlatamaz.
- Ayrı bir iş parçacığında **arka planda** çalışır; yazma ve gezinme anında kalır.
- **Devam ettirilebilir**dir — onu iptal ederseniz ya da çalışma ortasında uygulamayı kapatırsanız, bir sonraki çalışma baştan başlamak yerine durduğu yerden devam eder.
- Yalnızca **eksik ya da güncel olmayan** özetleri hesaplar — özeti zaten güncel olan notlar atlanır, bu yüzden ikinci bir çalışma hızlıdır.

---

## Kendi özetinizin kullanıldığından emin olma

Bir kartta özet, tek bir **Özet** etiketi altında görünür — kart, metnin sizden mi yoksa motordan mı geldiğini rozetlemez. Buna karar veren şey yukarıdaki önceliktir: bir notun frontmatter alanlarından biri ya da özet çağrı kutularından biri varsa, Constellation *onu* gösterir ve asla bir tane üretmez.

Yani bir not, makinenin seçtiği gibi okunan bir özet gösteriyorsa, o notun ne bir frontmatter özeti ne de bir özet çağrı kutusu vardır — ve düzeltme bir tane eklemektir:

- Notun frontmatter'ına bir `summary:` (ya da `description:` / `abstract:` / `excerpt:`) alanı ekleyin, **ya da**
- Gövdeye bir `> [!summary]` (ya da `[!abstract]` / `[!tldr]`) çağrı kutusu ekleyin.

O notun özeti bir sonraki kez hesaplandığında — kartı bir sonraki yüklendiğinde ya da **Tüm özetleri oluştur**'u çalıştırdıktan sonra — sözleriniz devralır.

---

## Yaygın iş akışları

**"Bir not makine özeti gösteriyor ama ben bir tane yazdım."**
Constellation aradığı yerde özetinizi bulamadı. Frontmatter alanınızın `summary`, `description`, `abstract` ya da `excerpt` olarak adlandırıldığından, **ya da** çağrı kutunuzun `[!summary]`, `[!abstract]` ya da `[!tldr]` olduğundan emin olun. Sonra yenilemek için Sınıflandırıcıyı yeniden açın (ya da *Tüm özetleri oluştur*'a tıklayın).

**"Sınıflandırıcıyı açtığım an her kartın özetini göstermesini istiyorum."**
Bir kez **Tüm özetleri oluştur**'a tıklayın ve bitmesine izin verin. Bundan sonra özetler önceden hesaplanır ve anında görünür.

**"Özetin notun kendisinin bir parçası olmasını, diskte olmasını istiyorum."**
Kendiniz yazın — bir `summary:` frontmatter alanı ya da bir `> [!summary]` çağrı kutusu ekleyin. Constellation o zaman sizin sürümünüzü gösterir (ve bir tane üretmeyi durdurur) ve sözleriniz, başka herhangi bir uygulamanın da okuyabileceği şekilde dosyada yaşar.

---

## İlgili konular

- **Sınıflandırıcı** — özetlerin her kartın altında göründüğü ve *Tüm özetleri oluştur*'un bulunduğu tam pencere ana yer.
- **Kaynak Gözden Geçirme** — özetlerin üzerinde durduğu sınıflandırma kartları.
- **Properties** — `summary:` / `description:` / `abstract:` / `excerpt:` frontmatter alanları ve nasıl ekleneceği.
- **Editing and Formatting** — bir notta `> [!summary]` çağrı kutusunun nasıl yazılacağı.
