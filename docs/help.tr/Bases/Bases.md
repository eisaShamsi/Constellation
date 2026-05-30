---
translation_status: AI-generated 2026-05-30 — native-speaker review recommended
language: tr
source: docs/help.uConstellation.World/Bases/Bases.md
aliases:
  - Bases
  - Constellation Base
  - Note tables
  - Structured views
  - Base files
  - Temeller
  - Takımyıldız Temeli
  - Not tabloları
  - Yapılandırılmış görünümler
  - Temel dosyaları
description: Takımyıldız Temeli'ni nasıl kullanacağınızı öğrenin — notlarınızın canlı bir tablosu; her not için bir satır, her özellik için bir sütun; bir dosyayı hiç taşımadan sıralayabilir, düzenleyebilir ve yeniden biçimlendirebilirsiniz.
---

# Temeller

> **Çeviri notu:** Bu yardım konusu, kanonik İngilizce sürümün
> (`help.uConstellation.World/Bases/Bases.md` adresinde)
> yapay zekâ tarafından üretilmiş çevirisidir. Anadili konuşuru
> incelemesi beklenmektedir. Lütfen düzeltmeleri proje deposu
> üzerinden iletin.

Bir **Temel**, notlarınızdan bir kümeyi canlı bir tabloya dönüştürür: **her not için bir satır, her özellik için bir sütun**. Hiçbir şey kopyalanmaz veya taşınmaz — tablo, notlarınızı yerinde okur ve onları tam şu anki halleriyle yansıtır.

> [!tip] Varsayılan olarak Güçlü ama Yalın
> Bir Temel, tanıdık ve sade bir görünümle açılır — yalnızca notlarınızın adları ve önemsediğiniz alanlar. Takımyıldız'ın daha derin, bilişsel sütunları her zaman **bir tık ötede**dir, ama ilk ekranı asla kalabalıklaştırmazlar. Ne kadar yapı çekeceğinize siz karar verirsiniz.

> [!info] Yıkıcı Değil
> Bir Temel notlarınızı kendi başına asla değiştirmez. O, bir sorgu tutan küçük bir `.base` dosyasıdır — "şu notları, şu sütunlarla, şu sırayla göster." Markdown dosyalarınız tam olduğu yerde kalır.

---

## Bir Temel'i kullanmanın iki yolu

**1. Tam sekme olarak.** Bir `.base` dosyasını açın; sekmeyi etkileşimli bir tablo olarak doldurur.

**2. Bir notun içinde.** Herhangi bir nota çitlenmiş bir kod bloğu bırakın; satır içinde oluşturulur:

````markdown
```base
view: table
```
````

Her ikisi de aynı motorla çalışır, bu yüzden tıpatıp aynı davranırlar.

---

## Bir Temel oluşturma

Kenar çubuğundan **Yeni Temel**'i kullanın ("+" / Yeni Temel eylemi). Takımyıldız sizin için küçük bir **YAML** `.base` dosyası yazar:

```yaml
schema: 1
lens: My Notes
scope:
  libraries: all
  federation: auto
columns:
  - dimension: note.name
view: table
```

| Alan | Anlamı |
|-------|---------|
| `schema` | Biçim sürümü (şu anda `1`). |
| `lens` | Tablonun üstünde gösterilen ad. |
| `scope.libraries` | `all`, ya da dahil edilecek belirli kütüphanelerin bir listesi. |
| `scope.federation` | `auto` — bağlı Evrenlerden (cUniverse'ler) gelen notları da dahil eder. |
| `columns` | Gösterilecek sütunlar. Yeni bir Temel yalnızca not **Adı** ile başlar. |
| `view` | `table` (tablo, Temel görünümüdür). |

Bunu elle düzenlemeniz nadiren gerekir — tablonun kendi denetimleri (aşağıda) her değişikliği sizin için dosyaya geri yazar.

---

## Tablo

- **Ad sütunu** — her zaman ilk sıradadır. Bir notu açmak için adına tıklayın.
- **Eşleşen her not bir satır olur.** **Satır sınırı yoktur.** Tablo *sanallaştırılmıştır* — yalnızca o anda ekranda olan satırları çizer — bu yüzden binlerce not üzerindeki bir Temel anında açılır ve sorunsuz kayar.
- **Hücre bazında yön** — her değer kendi soldan sağa veya sağdan sola yazımını algılar, böylece karışık dilli tablolar doğru okunur.
- Alt bilgi, sorgunun ne kadar sürdüğünü gösterir.

---

## Sütunlar — ekle, kaldır, yeniden sırala

### Bir sütun ekleyin

**+ Sütun ekle**'ye tıklayın. Seçici iki gruba ayrılmıştır:

- **Alanlarınız** — Takımyıldız'ın notlarınızda bulduğu frontmatter özellikleri (örneğin `status`, `maturity`, `author`). Bunlar *sizin* verilerinizdir.
- **Takımyıldız** — uygulamanın her zaman bildiği yerleşik alanlar: **Ad**, **Yol**, **Oluşturulma** ve **Özet**.

Listeyi süzmek için yazmaya başlayın. Tabloda zaten bulunan alanlar işaretlenir, böylece onları iki kez eklemezsiniz.

### Bir sütunu kaldırın

Bir sütun başlığının üzerine gelin ve **×**'e tıklayın.

### Sütunları yeniden sıralayın

**Bir sütun başlığını basılı tutup yana sürükleyin.** Tüm sütun kalkar (sönükleşir ve başlık bir tutma çerçevesi gösterir) ve dikey bir çizgi nereye bırakılacağını işaretler. Taşımak için bırakın. Ad sütunu ilk sütun olarak sabit kalır.

Her ekleme, kaldırma ve yeniden sıralama otomatik olarak `.base` dosyasına geri kaydedilir.

---

## Sıralama

**Bir sütuna göre sıralamak için başlığına tıklayın.** Her tıklama **artan → azalan → kapalı** arasında geçiş yapar (bir ok geçerli yönü gösterir).

Birden fazla sütuna göre sıralamak için **Sıralama** panelini açın:

- Birkaç sütun ekleyin — ilki birincil sıralamadır, sonrakiler eşitlikleri çözer.
- Herhangi bir düzeyi artan ve azalan arasında çevirin.
- Önceliği değiştirmek için düzeyleri yukarı veya aşağı taşıyın ya da onları kaldırın.

---

## Tablodan bir notu düzenleme

**Alanlarınız**dan birinin frontmatter sütunundaki bir hücreye çift tıklayarak düzenleyin:

- **Serbest metin alanları** — yeni değeri yazın; **Enter** kaydeder, **Escape** iptal eder.
- **Liste türü alanlar** (`maturity` gibi) — geçerli değerlerin **doğal sıralarıyla** bir **açılır liste** belirir (`maturity` için: *seed → sapling → evergreen → canonical*). Birini seçin ya da kendinizinkini yazın.

Değişiklik doğrudan o notun diskteki YAML frontmatter'ına yazılır ve tablo yerinde güncellenir.

> [!note] Salt okunur sütunlar
> **Ad** ve **Oluşturulma** (ve diğer yerleşik Takımyıldız sütunları) sizin için hesaplanır, bu yüzden düzenlenemezler. Burada yalnızca kendi frontmatter alanlarınız değiştirilebilir.

---

## Eski bir Temel'i açma

Obsidian'dan ya da Takımyıldız'ın daha eski bir sürümünden geçtiyseniz, mevcut `.base` dosyalarınız daha eski bir biçim kullanır.

**Dosyanıza asla dokunulmaz.** Takımyıldız bir tanesini açtığında, biçimin daha eski olduğunu açıklayan sakin bir bildirim gösterir ve bir **Takımyıldız Temeline dönüştür** düğmesi sunar. Dönüştürme **yalnızca siz tıkladığınızda** gerçekleşir — dosyayı yerinde yeni YAML biçimine yükseltir (taşıyabildiğini taşır: adı, sütunları ve basit metin süzgeçlerini). Dönüştürmeyi seçene kadar, özgün dosya tam olduğu gibi bırakılır.

---

## Federasyon

Bir Temel, Evren-farkındadır. `federation: auto` ile, bağlı Evrenlerden (cUniverse'ler) gelen notları kendinizinkilerle birlikte dahil eder. Bağlı bir Evrende yaşayan notlar salt okunurdur — onları Temel'de görüntüleyebilir ve sıralayabilirsiniz, ama düzenleme size ait notlara ayrılmıştır.

---

## Yerel-öncelikli ve dosya-uygulamadan-üstün

Temellerin kendine ait verisi yoktur. Gördüğünüz her değer, diskinizdeki gerçek bir `.md` dosyasından gelir, canlı olarak okunur. `.base` dosyasını silin; notlarınız tamamen etkilenmez — bir Temel, halihazırda sahip olduğunuz notlara doğrulttuğunuz bir mercekten ibarettir.

---

## Klavye ve fare

| Eylem | Ne yapar |
|--------|--------------|
| Bir sütun başlığına **Tıkla** | Ona göre sırala (artan → azalan → kapalı) |
| Bir sütun başlığını **Sürükle** | O sütunu yeniden sırala |
| Bir başlıktaki ×'e **Tıkla** | O sütunu kaldır |
| Bir frontmatter hücresine **Çift tıkla** | Onu düzenle (liste alanları için açılır liste) |
| **Enter** | Düzenlemeyi kaydet |
| **Escape** | Düzenlemeyi iptal et |
| Bir notun adına **Tıkla** | Notu aç |
