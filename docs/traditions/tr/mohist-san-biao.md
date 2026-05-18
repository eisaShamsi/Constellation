---
id: mohist-san-biao
name: Mohist sān biǎo
family: chinese-pragmatist
shape: horizontal-bands
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# Mohist sān biǎo (三表)

**Aile**: Çin pragmatisti · **Şekil**: yatay bantlar (3 bölge)

## Ana metafor

Kubbe, bir doktrini değerlendirmek için her bir Mohist ölçütüne bir
tane olmak üzere **üstten alta yığılmış üç yatay bölgeye** bölünür:

- **本 běn (kök)** — üst. Bilge-kralların tarihsel emsalleri:
  doktrinin miras alınan gelenekte bir dayanağı var mı?
- **原 yuán (köken)** — orta. Doğrudan gözlemsel kanıt: sıradan
  insanlar bunun böyle olduğunu görüp duyuyor mu?
- **用 yòng (kullanım)** — alt. Pratik sosyal fayda: bu doktrini
  benimsemek insanların yaşamlarını iyileştiriyor mu?

Bir doktrin yalnızca üç testi de geçtiğinde tutmaya değerdir — ancak
Sight işleyişi, evreninizdeki hangi dayanak-türünün en çok işi
yaptığına dair bir his elde etmek için notları üçü arasında dağılmış
olarak görmenizi sağlar.

Yatay eksen belirli bir kodlama taşımaz — Mohist'in üç ölçütü
*sıralı* değil, *kategoriktir*, bu yüzden bant içindeki konumlandırma
deterministik not-başı titreşim yoluyla yapılır.

## Kapsam

**Bu geleneği ne zaman kullanmalı.** Testin *bir doktrinin tutmaya
değer olup olmadığı* olduğu, hangi tür dayanağın altta yattığı
olmadığı içerikle çalışırken. Tarihsel emsal / gözlem / fayda'nın üç
gerekçe ekseni olduğu politik, etik, uygulamalı-ampirik ve
pratik-karar içeriği için yararlıdır.

**Bu geleneği ne zaman kullanmamalı.** İçeriğin doktriner veya
değerlendirici boyutu olmadığında. Saf betimleyici içerik, yaratıcı
çalışma ve öznel deneyim hakkındaki notlar kötü uyar.

## Uygulanabilirlik

- Politik öneriler ve gerekçeleri.
- Karşılaştırmalı-etik analizi (bu kural üç testi geçiyor mu?).
- İnsanlara faydanın açık olduğu mühendislik ve uygulamalı bilim.

## Soy hattı

Klasik Çin pragmatist epistemolojisi. Mòzǐ 墨子 (~MÖ 5. yy), kendisini
Konfüçyüsçülüğe eleştirel bir alternatif olarak sunan Mohist okulunu
kurdu. Sān biǎo, Mohistlerin miras alınan kaderci doktrine
uyguladıkları test olarak «Anti-Kadercilik» bölümünde geçer — ve
üçü de başarısız olarak sonuçlandığı sonucuna varırlar. Okul kısa bir
süre serpildi, ardından Konfüçyüsçü ve Legalist yükselişi tarafından
gölgelendi; bugün *Mòzǐ jiāngǔ* gibi baskılar aracılığıyla incelenen
geri kazanılabilir kanonik metin olarak hayatta kalır.

## Eleştiri

Sān biǎo, kanıtsal dayanağı yararla birleştiren erken bir pragmatizm
biçimi olarak bazen eleştirilir — özellikle «insanlara fayda» ölçütünü
biçimselleştirmek zordur. Modern akademisyenler ayrıca sān biǎo'nun
tam olarak gelişmiş bir epistemik teori mi yoksa belirli bir
anti-kaderci argümanda konuşlandırılmış polemik-retorik bir araç mı
olduğunu tartışır. Metodolojik özü laik olduğu için Cennet-teoloji
bağlamına rağmen din-soyu kuralı kapsamında küratörlü temele dahil
edildi.

## Atıf

**Birincil.** *Mòzǐ* 墨子, Book IX, "Fēi Mìng Shàng" 非命上
(«Anti-Kadercilik, Bölüm I»). Eleştirel baskı: Sūn Yíràng, ed.,
*Mòzǐ jiāngǔ* 墨子閒詁, 2 vols. (Beijing: Zhonghua Shuju, 1986).
İngilizce: Ian Johnston, trans., *The Mozi: A Complete Translation*
(New York: Columbia University Press, 2010).

**Modern.** A. C. Graham, *Disputers of the Tao: Philosophical Argument
in Ancient China* (La Salle, IL: Open Court, 1989), ch. 1; Chris
Fraser, "Mohism," *Stanford Encyclopedia of Philosophy* (2020).

## Not başı ön ek

`mohist_zone: ben | yuan | yong`. Şu anda yok — notlar görsel yapının
doldurulması için notPath ile üç bölge arasında deterministik olarak
hash-bölünür. Rust tarafı `LayoutCacheRow` uzantısı geldiğinde, bu
alan hash-bölme atamasını geçersiz kılar.
