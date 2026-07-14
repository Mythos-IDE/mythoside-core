# MythosIDE

[![License: FSL-1.1-ALv2](https://img.shields.io/badge/license-FSL--1.1--ALv2-blue)](./LICENSE.md)
[![GitHub Discussions](https://img.shields.io/github/discussions/Mythos-IDE/mythoside-core)](https://github.com/Mythos-IDE/mythoside-core/discussions)
[![GitHub issues](https://img.shields.io/github/issues/Mythos-IDE/mythoside-core)](https://github.com/Mythos-IDE/mythoside-core/issues)

<p align="center"><a href="./README.md">English</a> · Türkçe</p>

**Karmaşık dünyalar inşa eden romancılar için geliştirilmiş bir yazar IDE'si.**

MythosIDE; Scrivener gibi araçların yapılandırılmış, uzun soluklu yazma yaklaşımını bir yazılım IDE'sinin akıllı ve bağlam odaklı deneyimiyle birleştirir. Dünyalarını tutarlı tutmak içinamp; beş farklı uygulamayı birbirine bağlamaktan yorulmuş fantastik, bilimkurgu ve epik kurgu yazarları için özel olarak tasarlanmıştır.

> Durum: Erken geliştirme aşaması. Hatalar ve eksiklikler olabilir. Katkı ve geri bildirimleriniz memnuniyetle karşılanır.

## Neden MythosIDE?

- **Kurgu için özel yapısal hiyerarşi** — Kendi kendinize yapılandırmak zorunda kaldığınız genel taslak oluşturucuların aksine, yerleşik olarak gelen Seri → Kitap → Bölüm → Sahne hiyerarşisi.
- **Akıllı dünya inşası** — Taslağınızdan ayrılmadan `@KarakterAdi` yazarak anında bağlamsal bir profil kartı alın.
- **Her zaman yerel öncelikli (Local-first)** — Gerçek bilgi kaynağı, diskinizdeki düz Markdown + YAML üst verileridir (frontmatter). Platforma bağımlılık veya "bu şirket kapanırsa romanıma ne olur?" endişesi yoktur.
- **Arka planda hızlı çalışır** — Yerel bir SQLite (FTS5) indeksi, çapraz referansları ve ilişki sorgularını ("Hangi klan Bölüm 4'te görünüyor?") hiçbir zaman ana bilgi kaynağı kaynağı haline gelmeden anında gerçekleştirir.

## Repo Yapısı

MythosIDE, biri yerel client biri yerel sunucu olan iki ayrı repo'ya bölünmüştür:

- **Bu repo (`mythoside-core`)** — motor katmanı. Hiçbir Tauri veya arayüz bağımlılığı olmayan bağımsız bir Rust crate'i (kütüphane + binary): elyazması veri modeli, Markdown+YAML dosya formatı, native dosya izleme, ve entity işlemleri (karakter/sahne vb. oluşturma/okuma/güncelleme/silme). Kendi stdin/stdout'u üzerinden JSON-RPC benzeri bir protokol konuşan küçük bir yerel sunucu process'i olarak çalışır — asla bir ağ portu açmaz, bu yüzden kendisini başlatan process dışında makinedeki hiçbir şey ona erişemez.
- [`mythoside-ts`](https://github.com/Mythos-IDE/mythoside-ts) — masaüstü client'ı. Bu crate'in binary'sini yönetilen bir "sidecar" process olarak başlatan ve ona proxy yapan bir Tauri + TypeScript uygulaması. Tüm arayüz, editör ve render işi orada gerçekleşir; bu repo'da hiçbiri yok.

Neden tek binary yerine iki process: gerçek yazma/parse mantığını yeniden kullanılabilir ve bağımsız test edilebilir tutar (buradaki `cargo test` arayüz, Tauri veya pencere gerektirmez), ve local-first garantisini sadece bir vaat değil yapısal bir gerçek haline getirir — bu crate asla bir port dinlemez, yani bir tarayıcı sekmesinin ya da başka bir yerel process'in araştırabileceği bir servis burada yoktur.

## Teknoloji Yığını

- Rust (bu crate) — elyazması veri modeli, Markdown+YAML ayrıştırma, dosya izleme (`notify`), JSON-RPC sunucusu
- [Tauri](https://tauri.app/) + TypeScript ([`mythoside-ts`](https://github.com/Mythos-IDE/mythoside-ts)) — masaüstü client'ı
- Yerel indeksleme için SQLite + FTS5 (planlandı, henüz başlanmadı)
- Özelleştirilmiş web tabanlı metin editörü (ProseMirror/Monaco tabanlı) (planlandı, `mythoside-ts`'de)

## Başlarken

```bash
cargo build   # mythoside-core kütüphanesini + binary'sini derler
cargo test    # format/watcher/RPC test paketini çalıştırır
```

Bu crate, [`mythoside-ts`](https://github.com/Mythos-IDE/mythoside-ts) tarafından bir bağımlılık olarak kullanılır (ve binary'si bir sidecar olarak paketlenir) — uygulamayı fiilen orada çalıştırırsınız. Gelişmeleri [Issues](../../issues) ve [Discussions](../../discussions) üzerinden takip edebilirsiniz.

## Lisans

MythosIDE, [Functional Source License, v1.1 (ALv2 Future License)](./LICENSE.md) kapsamında kaynak kodları erişilebilir durumdadır. Özetle: Kendi yazma süreçleriniz için uygulamayı özgürce kullanabilir, inceleyebilir, değiştirebilir ve kendi sunucunuzda barındırabilirsiniz; sadece bunu rakip bir ticari ürün veya hizmet olarak yeniden paketleyemezsiniz. Her sürüm, yayınlandıktan iki yıl sonra otomatik olarak Apache 2.0 lisansına dönüşür.

"MythosIDE" ve logosu projenin ticari markalarıdır ve yukarıdaki lisans kapsamında değildir — ayrıntılar için [LICENSE.md](./LICENSE.md) dosyasına bakın.

## Katkıda Bulunma

Bir pull request açmadan önce [CONTRIBUTING.md](https://github.com/Mythos-IDE/.github/blob/main/CONTRIBUTING.md) belgesini inceleyin.

## Güvenlik

Güvenlik açıklarını nasıl bildireceğinizi öğrenmek için [SECURITY.md](https://github.com/Mythos-IDE/.github/blob/main/SECURITY.md) belgesini inceleyin.
