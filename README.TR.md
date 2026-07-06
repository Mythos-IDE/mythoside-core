# MythosIDE

[English](./README.md) · Türkçe
[![License: FSL-1.1-ALv2](https://img.shields.io/badge/license-FSL--1.1--ALv2-blue)](./LICENSE.md)
[![GitHub Discussions](https://img.shields.io/github/discussions/Mythos-IDE/mythoside-core)](https://github.com/Mythos-IDE/mythoside-core/discussions)
[![GitHub issues](https://img.shields.io/github/issues/Mythos-IDE/mythoside-core)](https://github.com/Mythos-IDE/mythoside-core/issues)

**Karmaşık dünyalar inşa eden romancılar için geliştirilmiş bir yazar IDE'si.**

MythosIDE; Scrivener gibi araçların yapılandırılmış, uzun soluklu yazma yaklaşımını bir yazılım IDE'sinin akıllı ve bağlam odaklı deneyimiyle birleştirir. Dünyalarını tutarlı tutmak içinamp; beş farklı uygulamayı birbirine bağlamaktan yorulmuş fantastik, bilimkurgu ve epik kurgu yazarları için özel olarak tasarlanmıştır.

> Durum: Erken geliştirme aşaması. Hatalar ve eksiklikler olabilir. Katkı ve geri bildirimleriniz memnuniyetle karşılanır.

## Neden MythosIDE?

- **Kurgu için özel yapısal hiyerarşi** — Kendi kendinize yapılandırmak zorunda kaldığınız genel taslak oluşturucuların aksine, yerleşik olarak gelen Seri → Kitap → Bölüm → Sahne hiyerarşisi.
- **Akıllı dünya inşası** — Taslağınızdan ayrılmadan `@KarakterAdi` yazarak anında bağlamsal bir profil kartı alın.
- **Her zaman yerel öncelikli (Local-first)** — Gerçek bilgi kaynağı, diskinizdeki düz Markdown + YAML üst verileridir (frontmatter). Platforma bağımlılık veya "bu şirket kapanırsa romanıma ne olur?" endişesi yoktur.
- **Arka planda hızlı çalışır** — Yerel bir SQLite (FTS5) indeksi, çapraz referansları ve ilişki sorgularını ("Hangi klan Bölüm 4'te görünüyor?") hiçbir zaman ana bilgi kaynağı kaynağı haline gelmeden anında gerçekleştirir.

## Teknoloji Yığını

- [Tauri](https://tauri.app/) — Hafif, yerel uygulama hissi veren çapraz platform masaüstü kabuğu.
- TypeScript / Node.js
- Yerel indeksleme için SQLite + FTS5
- Özelleştirilmiş web tabanlı metin editörü (ProseMirror/Monaco tabanlı)

## Başlarken

Proje stabilize edildikçe kurulum talimatları buraya eklenecektir. Gelişmeleri [Issues](../../issues) ve [Discussions](../../discussions) üzerinden takip edebilirsiniz.

## Lisans

MythosIDE, [Functional Source License, v1.1 (ALv2 Future License)](./LICENSE.md) kapsamında kaynak kodları erişilebilir durumdadır. Özetle: Kendi yazma süreçleriniz için uygulamayı özgürce kullanabilir, inceleyebilir, değiştirebilir ve kendi sunucunuzda barındırabilirsiniz; sadece bunu rakip bir ticari ürün veya hizmet olarak yeniden paketleyemezsiniz. Her sürüm, yayınlandıktan iki yıl sonra otomatik olarak Apache 2.0 lisansına dönüşür.

"MythosIDE" ve logosu projenin ticari markalarıdır ve yukarıdaki lisans kapsamında değildir — ayrıntılar için [LICENSE.md](./LICENSE.md) dosyasına bakın.

## Katkıda Bulunma

Bir pull request açmadan önce [CONTRIBUTING.md](https://github.com/Mythos-IDE/.github/blob/main/CONTRIBUTING.md) belgesini inceleyin.

## Güvenlik

Güvenlik açıklarını nasıl bildireceğinizi öğrenmek için [SECURITY.md](https://github.com/Mythos-IDE/.github/blob/main/SECURITY.md) belgesini inceleyin.
