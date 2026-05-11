# Epistemik Üst Veri

> **Çeviri notu:** Bu yardım konusu, kanonik İngilizce sürümün
> (`help.uConstellation.World/Epistemic Metadata/Epistemic Metadata.md` adresinde)
> yapay zekâ tarafından üretilmiş çevirisidir. Anadili konuşuru
> incelemesi beklenmektedir. Lütfen düzeltmeleri proje deposu
> üzerinden iletin.

*(MIG-022 §A — boşluk analizi §6.1 şema uzantıları)*

Bu konu, notlarınızın daha zengin epistemik sınıflandırması için Constellation'ın artık tanıdığı küçük bir **isteğe bağlı frontmatter alanı** kümesini açıklar. Bunlar boşluk analizine (`docs/epistemic-content-gap-analysis.md`) yanıt olarak eklendi — Constellation Epistemik İçerik Motoru'nun (CECE) sınıflandırmada kullandığı iki eksenli "Source × Content Type" modelinin, bildiğinizi nasıl bildiğinize dair kaydetmek isteyebileceğiniz her şeyi kapsamadığının kabul edilmesi.

Bu alanlar **tamamen isteğe bağlıdır**. Bunlara sahip olmayan mevcut notlar değişmeden çalışır. Bunları, bir not ek sinyalden yararlanan türde bir bilgi olduğunda elle (veya gelecekte yapılandırılmış bir editör aracılığıyla) eklersiniz.

---

## Alanlar

### `held_by` — *bu kimin tutumu?*

Notun tanımladığı tutumu üstlenen kişiyi gösteren kısa bir dize. Varsayılan değer `user`'dır (kendi tutumunuz). Kullanabileceğiniz diğer değerler:
- Bir alimin adı: `held_by: "al-Shāfiʿī"`
- Bir mezhep (madhhab): `held_by: "Ḥanafī"`
- Tarihî bir figür: `held_by: "Aristotle"`

Kendi tutumunuzdan ziyade *başka birinin* tutumunu kaydeden bir not yazdığınızda, bunu söyleyen alan `held_by`'dır. O olmadan, Constellation notun epistemik durumunun sizin kendinizinki olduğunu örtük olarak varsayar — ki ciddi akademik çalışma için bu çoğunlukla yanlıştır.

### `domain` — *bu hangi konu hakkında?*

Disiplin etiketleri listesi. Serbest biçimli `tags` alanından (folksonomy / ruh hali / proje) farklı olarak, `domain` getirme ve filtreleme için yapılandırılmış disiplin/konu alanıdır. Örnekler:

```yaml
domain: [fiqh, ʿibādāt]
domain: [photography, optics]
domain: [overland-travel, mechanics]
```

`content_type: "proposition"` VE `source: "inference"` olarak sınıflandırılan bir not, mantık teoremi (domain: `[logic, mathematics]`) veya hukuki bir görüş (domain: `[fiqh, ʿibādāt]`) olabilir — aynı epistemik biçim, çok farklı getirme bağlamları. `domain`, hangisi olduğunu söylemenize olanak tanır.

### `function` — *bu not ne için?*

Notun amaçlanan kullanımını tanımlayan tek bir dize. Tanınan değerler:

- `reference` — gerektiğinde okuma (bir tanım, bir alıntı, daha sonra bakacağınız bir gerçek)
- `seed` — kuluçkaya yatırma (hâlâ geliştirmekte olduğunuz erken aşama bir fikir)
- `actionable` — bununla bir şey yapma (bir görev, bir takip, verilecek bir karar)
- `shipped` — bitmiş ürün (yayımlanmış bir makale, teslim edilmiş bir analiz, kapatılmış bir döngü)

CECE'nin içerik türü ekseninden farklı (hangi TÜR bilgi olduğunu söyler) — `function` notla NE YAPACAĞINIZI söyler.

### `provenance_civilization` — *hangi geleneğin söz dağarcığı işbaşında?*

Notun söz dağarcığının medeniyet ayak izini tanımlayan isteğe bağlı bir dize. Geleneğe özgü kaynaklara karşı getirme için yararlıdır. Örnekler:

- `provenance_civilization: "sunni-usuli"` — Sünni *uṣūl al-fiqh* geleneği (al-Bukhārī, al-Ghazālī, al-Āmidī)
- `provenance_civilization: "analytic-western"` — Frege sonrası analitik felsefe
- `provenance_civilization: "nyaya"` — pramāṇa epistemolojisinin Hint Nyāya okulu
- `provenance_civilization: "buddhist-pramana"` — Budist epistemoloji geleneği (Dignāga, Dharmakīrti)

Çoğu notun buna ihtiyacı yoktur. Hem Sünni *uṣūl*'a hem de analitik Anglo-Amerikan epistemolojisine dayanan bir notunuz olduğunda, birincil ayak izini kaydetmek geleceğin sizin doğru karşılaştırmalı malzemeyi bulmasına yardımcı olur.

### `updated_at` — *tutumunuz en son ne zaman değişti?*

Notun epistemik içeriğinin en son bilinçli revizyonunun ISO tarihi. Dosya sisteminin `modified` zaman damgasından farklıdır (her kaydetmeyi yakalar, hatta yazım hatası düzeltmelerini bile); `updated_at`, gerçekten tutumu yeniden düşündüğünüzde SİZİN ayarladığınız zaman damgasıdır.

```yaml
updated_at: 2026-05-09
```

§6.3 zamansal eksenin geri kalanı geldiğinde yararlıdır (not durum geçmişi) — o zamana kadar bu, "görüşümü en son revize ettiğim zamanı" kaydeden tek anlık görüntü alanıdır.

### `ikhtilāf` — *yapılandırılmış akademik anlaşmazlık*

Yeni alanların en karmaşığı. Bir mesele üzerinde alimler veya mezhepler arasındaki yapılandırılmış anlaşmazlığı — *ikhtilāf*'ı — `{school, position}` çiftleri listesi olarak kaydeder. Constellation bunu düzenlemek için özel bir Properties paneli widget'ı sağlar; YAML'i doğrudan da düzenleyebilirsiniz.

Örnek:

```yaml
ikhtilāf:
  - school: Ḥanafī
    position: permissible
  - school: Mālikī
    position: discouraged
  - school: Shāfiʿī
    position: permissible with conditions
  - school: Ḥanbalī
    position: forbidden
```

`ikhtilāf` ile bir not herhangi bir tek epistemik durumda değildir — birden çok aktör arasındaki *yapılandırılmış anlaşmazlığı* kaydeder. Bu alan olmadan Constellation böyle bir notu sanki bu pozisyonlardan birini kendisi tutuyormuş gibi ele alırdı, bu da yanlıştır.

Properties paneli her satırı iki giriş (school + position) ve bir kaldırma düğmesiyle birlikte bir editör kartı olarak ve altta bir "Mezhep ekle" düğmesiyle birlikte oluşturur.

### `warrant` ve `warrant_notes` — *ayrıştırıldı ama (şimdilik) atıl*

İki alan ayrıştırılır ve diske kaydedilir, ancak **henüz herhangi bir UI'da yüzeye çıkmaz**:

- `warrant: "mutawātir"` — notun iddiasının dayanağı (warrant) için bir derece etiketi. Sünni *uṣūl* hiyerarşisi *mutawātir / mashhūr / āḥād*'i kullanır ve özellikle hadis içinde *ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ*'i kullanır. Diğer geleneklerin kendi derece sözlükleri vardır.
- `warrant_notes: "transmitted by 30+ companions in al-Bukhārī"` — warrant derecesini destekleyen serbest metin.

Bunlar **Constellation Warrant Research iş akışı** sınıflandırıcısını yayınladığında kullanıma hazırdır (çok aylık araştırma projesi; bkz. boşluk analizi §6.2). O zamana kadar elle doldurabilirsiniz ve veriler kalıcıdır; hiçbir şey bunu görüntülemez. Gelecekteki warrant duyarlı sorgular ve rozetler bu değerleri doğrudan okur.

---

## Bu alanların göründüğü yer

Notunuzun frontmatter'ına yeni alanlardan herhangi birini doldurduğunuzda, diğer her YAML alanı gibi **Properties panelinde** (sağ kenar çubuğu) görünürler — anahtar başına bir satır, türe uygun editörle birlikte:

- `held_by`, `function`, `provenance_civilization`, `warrant`, `warrant_notes` → metin girişi
- `domain` → etiket listesi (yazıp + Enter ile ekleyin, her etiketteki × ile kaldırın)
- `updated_at` → tarih seçici
- `ikhtilāf` → `school` / `position` satırları + ekle/kaldır düğmeleri ile özel widget

---

## `supersedes` ne olacak?

`supersedes` teknik olarak tek bir notun özelliğinden ziyade *notlar arasındaki bir ilişkidir*. Constellation bunu YAML skaleri olarak değil, **türlü bağlantı** olarak ele alır:

```markdown
This note replaces my earlier analysis: [[old-note-id|supersedes]]
```

Wikilink üzerindeki `|supersedes` sonek, Constellation'a bunun `supersedes` türünde bir türlü bağlantı olduğunu söyler — farklı bir hap rengine sahiptir (kayrak mavi-gri), diğer türlü bağlantılarla birlikte Backlinks + Outgoing Links panellerinde görünür ve Living Link Architecture'a (ağırlık, yaşam döngüsü, geçiş sayıları) katılır.

Bu, not-not ilişkilerini tek bir yerde — türlü bağlantı sisteminde — tutar, bunları türlü bağlantılar ve frontmatter skalerleri arasında bölmek yerine. Aynısı `contradicts:` için de geçerlidir (MIG-022 öncesi söz dağarcığında zaten türlü bağlantı).

---

## Bu NE DEĞİLDİR

Bu alanlar bugün CECE sınıflandırması tarafından **TÜKETİLMEZ**. CECE yalnızca Source × Content Type üzerinden sınıflandırır; yeni üst veri alanları insan odaklı getirme, gelecekteki warrant duyarlı sınıflandırıcılar ve zamansal eksen (yayınlandığında) için kaydedilir.

Özellikle:
- `function: "actionable"` Tasks panelinde otomatik olarak bir görev oluşturmaz
- `held_by: "al-Shāfiʿī"` CECE'nin notu sınıflandırma şeklini değiştirmez
- `domain: [fiqh]` arama sorgunuzu onu içerecek şekilde yazmadığınız sürece arama sonuçlarınızı filtrelemez

Alanlar bir **şemadır** — ekleyebileceğiniz tanınmış bir söz dağarcığı. Gelecekteki MIG'ler bunları tüketen özellikler yayınlayacaktır (warrant sınıflandırıcı, zamansal sorgular, alan duyarlı filtreleme vb.).

---

## İşlenmiş örnek

Sünni mezheplerinin "şafak vakti orucu bozma yükümlülüğünün günün geçerliliği için önemli olup olmadığı" konusundaki tutumlarını kaydeden bir not:

```yaml
---
title: Niyyah for Ramadan fasting
held_by: user
domain: [fiqh, ʿibādāt, sawm]
function: reference
provenance_civilization: sunni-usuli
updated_at: 2026-05-09
warrant: mashhūr
ikhtilāf:
  - school: Ḥanafī
    position: night-before niyyah valid; same-day niyyah valid before zawāl
  - school: Mālikī
    position: night-before niyyah required; one general niyyah for the month suffices
  - school: Shāfiʿī
    position: night-before niyyah required for each obligatory fast
  - school: Ḥanbalī
    position: night-before niyyah required for each obligatory fast
---

The classical Mālikī position (one niyyah for the month) is described
by [[Ibn-Rushd-bidayah|derives-from]] in the bidāyat al-mujtahid passage
on niyyah. My current view: [[ramadan-niyyah-personal|supersedes]]
my earlier note that conflated the Mālikī position with the Shāfiʿī one.
```

Yedi yeni alandan altısı doldurulmuş; `warrant_notes` atlandı (henüz kaydedilecek bir aktarım zinciri ayrıntısı yok); `supersedes` ve `derives-from`, YAML skalerleri olarak değil, gövdedeki türlü bağlantılar olarak.

---

*MIG-022 §A — şema uzantıları bu Constellation derlemesine geliyor. Warrant Research iş akışı (ayrı Concept Paper, çok aylık), `warrant` alanını tüketen warrant sınıflandırıcısını yayınlar. Zamansal eksen (MIG-023, ayrı Architect döngüsü), `updated_at`'ı artı daha geniş not durum geçmişini tüketir.*
