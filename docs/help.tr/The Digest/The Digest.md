---
translation_status: AI-generated 2026-05-24 — native-speaker review recommended
language: tr
source: docs/help.uConstellation.World/The Digest/The Digest.md
aliases:
  - The Digest
  - Universe Digest
  - Digest
  - Digest pane
  - Evren Özeti
  - Özet
  - Özet paneli
description: Evren Özeti, bilgi tabanınızdaki her notu özet-başlık seviyesinde gösteren sol dock panelidir — kademeli Library → Klasör → Not — böylece hiçbir şey açmadan tüm Evreni göz gezdirebilirsiniz. Bir satıra tıklayın, genişlesin ve tam özet satır içinde görünsün. Filtre tüm listeyi daraltır; sıralama yenilik (varsayılan) ile alfabetik arasında geçiş yapar. Her yerde gördüğünüz aynı özetleri okur; ek hesaplama yok; ek disk alanı yok.
---

# Evren Özeti

> *Özeti aklınızın içindekiler tablosu olarak düşünün — dosyaların bir listesi değil, fikirlerin bir listesi.*

**Evren Özeti**, tüm bilgi tabanınızı *anlam* seviyesinde göz gezdirme yeridir. Dosya ağacı (sadece isimler) veya Sky View (sadece şekiller) yerine, Özet her notun altında size **notun ne hakkında olduğunu söyleyen tek cümleyi** gösterir. Bir satıra dokunun ve çok cümleli tam özet satır içinde genişler. Bir tane bile açmadan, bir dakikada elli notluk özü okuyabilirsiniz.

**Sol dock**'unuzda Dosya ağacı ve Sky View ile birlikte yaşar — Constellation'ın navigasyon için sunduğu yollardan biri.

---

## Özet neden var

Bir dosya ağacı size *neyiniz olduğunu* söyler. Bir arama size ne *istediğinizi* söyler. Özet size *ne bildiğinizi* söyler.

Evreniniz birkaç yüz notu aştığında, "ne söylediğini hatırlamak için her birini açmak" imkânsız hale gelir. Her notun **özünü** kaydırma hızında okumanın bir yoluna — ve onun hakkında daha dikkatlice düşünmek istediğiniz anda herhangi bir özü tam özete genişletmenin bir yoluna ihtiyacınız var. İşte Özet budur.

Bu, **Note Summary Creator (NSC)** Core Plug-In'in üçüncü sütunudur:
- **Sütun 1**: bir özet motoru (Faz 1 / MIG-043).
- **Sütun 2**: bir notun göründüğü her yere özeti yerleştiren bir servis (Faz 2 / MIG-044 — Sınıflandırıcı, Arama sonuçları, Düzenleyici şeridi, Geri Bağlantılar, Giden Bağlantılar, Dizin, Sky View hover).
- **Sütun 3**: bu görünüm — Evren Özeti (Faz 3 / MIG-045).

---

## Özeti açma

**Sol kenar çubuğunda**, **Evren Özeti simgesine** tıklayın (köşesinde daire olan küçük bir liste) — Dosya ağacı ve Sky View simgelerinin yanında. Kenar çubuğu Özet paneline geçer.

Geri dönmek için, diğer üç simgeden herhangi birine tıklayın (veya **Escape**'e basın).

---

## Gördükleriniz

Yukarıdan aşağıya:

1. **Araç çubuğu.** Bir arama girişi + küçük bir saat simgesi (sıralama anahtarı, varsayılan "yeniliğe göre").
2. **Library başlıkları.** Büyük harflerle mor çubuklar — Evrenizdeki her library için bir tane. Her biri library'nin adını ve içerdiği not sayısını gösterir.
3. **Klasör başlıkları.** Küçük soluk etiketler — *not içeren* her klasör için bir tane. Library kökünde yaşayan notlar klasör başlığı almaz.
4. **Not satırları.** Her satırın:
   - Solda bir chevron (▶) — satırı genişletmek için tıklayın.
   - Etkileşimli vurgu renginde **not adı** — düzenleyicide **notu açmak için** tıklayın.
   - Adın altında soluk bir italik satır — **özet başlığı** (diğer her Faz 1/2 yüzeyinde görünen aynısı).

---

## Tam özeti okumak için bir satırı genişletme

Bir satırın solundaki **chevron**'a (▶) tıklayın — ya da **başlığın italik satırına** doğrudan tıklayın. Chevron ▼'ye döner ve **çok cümleli tam özet** başlığın altında satır içinde görünür, ihtiyaç duyduğu kadar satıra doğal olarak sarılır.

Daraltmak için chevron'a (veya başlığa) yeniden tıklayın.

"Genişletmek için chevron'a tıkla, açmak için adı tıkla" bölünmesi iki jesti ayrı tutar: bir not *hakkında okumak için* genişletebilir, ardından onu geçerek kaydırmaya devam edebilirsiniz; yalnızca adı tıkladığınızda not gerçekten açılır ve odağı alır.

---

## Filtreleme

Üstteki **arama girişine** yazın. Yazdıkça liste daralır — yalnızca **adı, başlığı veya tam özeti** sorgunuzu içeren notlar görünür kalır. Eşleşen sıfır notu olan Library başlıkları ve klasör başlıkları tamamen kaybolur (boş başlık yok).

Tam listeyi geri yüklemek için girişi temizleyin (× düğmesi veya backspace).

Filtre **anlıktır** — Constellation diskinize veya veritabanına dokunmaz. Halihazırda bellekteki özetleri okur, böylece 10.000 notluk bir Evren bile yazma hızında filtrelenir.

---

## Sıralama: yenilik veya alfabetik

İki sıralama modu arasında geçiş yapmak için araç çubuğundaki **saat simgesine** tıklayın:

- **Yenilik** (varsayılan) — her klasör içinde, notlar **oluşturma zamanı, en yeniler önce** sırasıyla görünür. Bir library içindeki klasörler içerdikleri en yeni nota göre sıralanır (böylece en aktif klasör önce görünür). Bu varsayılandır çünkü *son zamanlarda üzerinde çalıştığınız şeyi* yüzeye çıkarır.
- **Alfabetik** — klasörler ada göre sıralanır, her klasör içindeki notlar ada göre sıralanır. Yeniliğe dönmek için tekrar tıklayın.

Anahtar oturum başınadır; Özeti kapatıp yeniden açın, yeniliğe geri döner.

---

## Federasyon: alt Evrenler satır içinde görünür

Evrenizin **bağlı alt Evrenleri** (cUniverses) varsa, bir alt Evrenden her library, ebeveyn Evrenin library'leriyle birlikte, Özette **kendi eş Library başlığı** olarak görünür. Özet, bu Evrenden ulaşılabilir her şeyin birleşik bir görünümüdür, sadece burada fiziksel olarak yaşayan library'lerin değil.

(Gelecekteki bir Constellation güncellemesi, alt Evren library'lerini geçici olarak Özetten gizlemek için bir aç/kapa anahtarı ekleyecek; şimdilik her zaman görünürler.)

---

## Özet devasa Evrenlerde nasıl hızlı kalır

Özet **sanallaştırılmıştır**: tüm ağacı değil, yalnızca kaydırma alanınızda şu anda görünür olan satırları render eder. 10.000 notluk bir Evren, 50 notluk biri kadar pürüzsüz kayar. Satırlar görünüme kaydıkça, özetleri Constellation'ın bellek içi önbelleğinden gruplar halinde alınır (diğer her Faz 1/2 yüzeyini besleyen aynı önbellek — ayrı bir iş yok, ayrı bir depolama yok).

Özet notlarınızı asla diskten yeniden okumaz. Özetleri asla yeniden hesaplamaz. Bu, motorun Faz 1'den doldurduğu aynı `note_summaries` tablosu üzerindeki bir **okuma** görünümüdür.

---

## Yaygın iş akışları

**"Bu hafta neye çalıştığımı görmek istiyorum."**
Özeti sıralama = Yenilik (varsayılan) ile açın. En son oluşturulan notlar her library/klasörün üstünde görünür. Başlıkları tarayın.

**"X hakkında yarı hatırlanan bir not arıyorum."**
Özeti açın. X (notun başlığında, özet başlığında veya tam özetinde görünecek bir kelime) yazın. Liste adaylara daralır. Tam özetleri okumak için chevron'lara tıklayın; kazananı açmak için ada tıklayın.

**"Library'mın yukarıdan aşağı bir incelemesini yazmak istiyorum."**
Özeti açın, sıralama = Alfabetik. Başlıkları sırayla dolaşın. Bir şey sizi yakaladığında daha dolu özetleri okumak için chevron'lara tıklayın. Bunu yeni bir MOC (Map of Content) notunun belkemiği olarak kullanın.

**"Federe bir cUniverse'i ilk kez keşfediyorum."**
Özeti açın. Kendi library'lerinizi geçip cUniverse'in library'lerine kaydırın — onlar eş satırlardır. Bağlı Evrenin neyi içerdiğini öğrenmek için başlıkları okuyun, ondan hiçbir şey açmadan.

---

## Özette OLMAYAN şeyler

- Satırlar üzerinde **sağ tıklama bağlam menüsü** — yeni sekmede açma, arşivleme vb. (v1 için, birincil eylemler tıkla-adı-aç ve tıkla-chevron-genişlet'tir. Gelecekteki bir güncelleme bağlam menüsü ekleyecek.)
- **Özel gruplandırmalar** — Library → Klasör v1 için tek katmanlamadır. (Henüz "etikete göre grupla" veya "aşamaya göre grupla" yok.)
- **Sürükle-yeniden-sırala** — Özet salt okunurdur; sıralama kurallardan gelir, manuel sıralamadan değil.
- **Sınıflandırıcı benzeri sınıflandırma kontrolleri** — Özet bir *gözatma* görünümüdür; sınıflandırma **Sınıflandırıcı**'da (ayrı panel) yaşar.

---

## İlgili konular

- **Not Özetleri** — özetlerin nereden geldiği, öncelik kuralı (sizinki kazanır) ve onları gösteren yüzeylerin tam listesi.
- **Sınıflandırıcı** — *Tüm özetleri oluştur*'un evi (Library'nizdeki her özeti bir kerede önceden hesaplayın, böylece Özet anında dolar).
- **Sky View** — bilginizin *şekil* görünümü (baloncuklar + bağlantılar); Özet onun tamamlayıcı *anlam* görünümüdür.
- **Bilgi Formülasyonu** — Constellation'ın bilgiyi yalnızca dosya depolaması yerine *bağlantı* ve *özet* ile düzenlemesinin nedeni.
