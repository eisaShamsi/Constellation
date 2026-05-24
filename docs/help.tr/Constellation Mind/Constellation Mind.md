---
aliases:
  - Constellation Zihni
  - Constellation Mind
  - Mind
  - Yerel LLM
  - Yerel Büyük Dil Modeli
  - Fanar
  - YZ Sohbet
  - Kişisel YZ
description: Constellation Mind, Constellation'ın yerel Büyük Dil Modeli (LLM) katmanıdır — kendi notlarınız hakkında sohbet edebileceğiniz, tamamen cihazınızda çalışan bir YZ. Faz 0b, Ayarlar → Mind'dan yüklenebilen Arapça-öncelikli Fanar-1-9B modeli ile 2026-05-24'te yayınlandı. Sohbet yüzeyi Faz 1'de iniyor.
---

# Constellation Mind (عقل Constellation)

## Bu nedir?

Constellation Mind, Constellation'ın yerel Büyük Dil Modeli (LLM) katmanıdır — Evreninizi tanıyan ve notlarınız hakkında sizinle konuşabilen bir YZ asistanı, **hiçbirini buluta göndermeden**.

Onu diğer her "notlar için YZ" aracından ayıran üç şey vardır:

1. **Yerel-öncelikli.** Model cihazınızda çalışır. Notlarınız asla onu terk etmez. Bulut gidiş-dönüşü yoktur — sohbet yerel ve çevrimdışı çalışabilir.
2. **Arapça-öncelikli.** Pakette gelen varsayılan model, Katar Bilgisayar Araştırma Enstitüsü'nün (QCRI) Arapça-merkezli, Sünni bilinçli modeli olan **Fanar-1-9B**'dir. MSA + Körfez lehçesinde yerel yetkinlik; İngilizce ikinci dildir, tek dil değildir.
3. **Atfa bağlı.** YZ'nin notlarınız hakkında yaptığı her olgusal iddia, kaynak notu atıf olarak göstermelidir. Halüsinasyonlu atıflar, üretim sonrası bir doğrulayıcı tarafından yakalanır (Faz 1).

## Bugün gönderilen (Faz 0b — 2026-05-24)

- **Ayarlar → Mind paneli** — yüklenebilir modelleri listeler (şu anda yalnızca Fanar 1.9B Q4_K_M, ~5 GiB), modeli indiren ve doğrulayan bir Yükle düğmesi ile.
- **Model yükleme** — bir GitHub Release'den parçalı indirme (üçüncü taraf bulutu yok), parça başına ve birleştirilmiş bütün üzerinde SHA-256 doğrulanmış.
- **Gerçek çıkarım çalışma zamanı** — `llama-cpp-2` (v1'de yalnızca CPU) Q4_K_M GGUF'u yükler ve token'ları akıtır.
- **Henüz sohbet yüzeyi yok** — bu Faz 1'dir (sonraki kilometre taşı). Bugün modeli yükleyip doğrulayabilirsiniz; sohbet UI'si MIG-048'de iniyor.

## Fanar nasıl yüklenir

1. **Ayarlar → Mind**'ı açın.
2. Katalogdan **Fanar 1.9B (Q4_K_M)**'yi bulun. Kart, boyutu (5,01 GiB), lisansı (savunmacı Gemma bildirimleriyle birlikte Apache-2.0) ve "Etkin olarak ayarla" veya "Yükle" düğmesini gösterir.
3. **Yükle**'ye tıklayın. Bir ilerleme çubuğu, indirme + SHA doğrulama + birleştirme'yi üç fazda gösterir.
4. Rozet **Yüklendi** + **Etkin**'e döndüğünde, model hazırdır. Fanar `<app-data>/Constellation/models/fanar-1-9b-q4km-v1.gguf`'ta yaşar ve mmap ile desteklenir (RAM'e kopya yok).

Hepsi bu. Faz 1 sohbet yüzeyini gönderene kadar, yüklü model bekleme modundadır.

## Faz 1'de gelenler (sonraki kilometre taşı)

- **Sohbet yüzeyi** — Evreniniz hakkında Fanar ile Arapça veya İngilizce konuştuğunuz bir Constellation paneli (mesaj başına RTL farkında).
- **Okuma araçları** — Mind, yanıtlarını gerçek notlarınızda temellendirmek için `search_notes`, `read_note`, `find_similar`, `list_recent`'i çağırabilir.
- **Atıf doğrulayıcı** — her iddia gerçek bir notu atıf gösterir; uydurulmuş `note:UUID` referansları size ulaşmadan önce reddedilir.
- **Uygulama başlangıcında ön-ısıtma** — Mind arka planda yüklenir, böylece ilk sohbetinizde 10 saniyelik soğuk yükleme bedelini ödemezsiniz.
- **Konuşma geçmişi** — Evren başına kaydedilir; Nota terfi ettirilebilir.

Tam mimari için `docs/Constellation-Mind-Concept-Paper-v1.1.md` ve faz-faz yol haritası için `docs/Constellation-Mind-Implementation-Plan-v1.0.md`'a bakın.

## Sonra gelenler

- **Faz 2 — Yazma araçları** (Mind, açık onayınız altında düzenlemeler / yeni notlar / bağlantılar önerir).
- **Faz 2.5 — RoutedProvider + Jais** (ikinci bir model, G42/MBZUAI'den Jais-2-8B, Fanar'a ortak-varsayılan olarak katılır; Mind isteğe göre bunlar arasında yönlendirme yapar).
- **Faz 3 — Otomatik-sınıflandırma + akıllı bağlama** (Mind, not kaydedildiğinde yönler ve bağlantılar önerir).
- **Faz 4 — Yetenek araçları** (ses → not, OCR → not, çeviri).
- **Faz 5 — Bulut katılımı** (kendi Anthropic / OpenAI anahtarınız, Evren başına maliyet üst sınırı ve tur başına çıkış günlüğü ile).

## Gizlilik ve veri akışı

- **Yalnızca bir model yüklenirken giden HTTP** — Constellation, model dosyalarını bu reponun [`models/*` GitHub Releases](https://github.com/eisaShamsi/Constellation/releases)'inden indirir. Telemetri yok. Bulut çıkarımı yok (henüz — bu Faz 5'tir ve yalnızca açık katılımınızla).
- **Diskte:** modelin GGUF'u + hangi modellere sahip olduğunuzu ve hangisinin etkin olduğunu izleyen bir `installed_models.json` kaydı.
- **Çalışma zamanında:** yüklü model dosyası bellek-eşlemelidir; istemleriniz ve yanıtlarınız yalnızca RAM'de yaşar.

## Lisanslar

Her model, GitHub Release'de yanında kendi LICENSE.txt'sini taşır. Fanar için:

- **Apache License 2.0** (QCRI'nin Fanar-1-9B-Instruct reposunda beyan ettiği lisans).
- **Gemma Kullanım Koşulları** — Fanar, `google/gemma-2-9b`'nin devam eden ön-eğitimidir; QCRI sonucu yalnızca Apache-2.0 olarak yeniden etiketlese bile Constellation, Gemma bildirimlerini savunmacı olarak gönderir.
- **Fanar atıfı** (Fanar Team 2025, arXiv:2501.13944).
- **Constellation yeniden dağıtım bildirimi** — Constellation'ın GitHub Release'indeki GGUF, QCRI'nin yukarı akış safetensors'unun bir kuantizasyonudur, `.github/workflows/model-pipeline.yml` tarafından üretilmiş ve orijinal LICENSE eşlik ederek Apache-2.0 altında dağıtılmıştır.

Tam LICENSE.txt, her modelin yanında sürümünde yaşar: <https://github.com/eisaShamsi/Constellation/releases/tag/models/fanar-1-9b-q4km-v1>.

## Sorun giderme

**Yükle düğmesi yerine "Henüz hazır değil" rozeti.** Pakete dahil katalogda o model için yer tutucu bir SHA-256 vardır. Normal bir Constellation yüklemesinde bu olmamalıdır; eğer görürseniz, katalog o model sürümü için güncellenmemiştir. Bir issue açın.

**Yükleme "Parça X/Y indiriliyor" konumunda donuyor.** Ağ sorunu. Ayarlar → Mind'dan iptal edin, Yükle'yi yeniden tetikleyin — kısmi parçalar otomatik olarak temizlenir.

**Yükleme başarılı, dosya SHA-256 eşleşmiyor.** İndirmede bir bit-flip. Yeniden yükleme taze bir kopya getirecektir.

**Sohbet yüzeyi eksik.** Faz 1 (MIG-048) henüz gönderilmedi. Model bugün yüklenip doğrulanabilir; sohbet UI'si sonraki sürümde iniyor.

---

*Alt konular Faz 1 gönderildikçe bu klasöre katılacak: sohbet UI'si rehberi, atıf-çipi dokunma davranışı, çoklu-model seçici, uzun sohbetlerin ikinci ekranda render edilmesi.*
