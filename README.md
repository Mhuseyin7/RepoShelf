# RepoShelf

> Developer workstation'ınızdaki Git repository'leri tek bir yerde keşfeden, analiz eden ve düzenleyen local-first workspace inventory uygulaması.

RepoShelf, onlarca repository ile çalışan developer'lar için tasarlanmış açık kaynak bir **software** aracıdır. Bilgisayarınızdaki Git repository'lerini scan eder; branch, değişiklik, remote durumu, disk kullanımı ve generated directory'ler gibi bilgileri anlaşılır bir arayüzde sunar.

Bu proje açık kaynak kodludur ve [muhammedkoca.com.tr](https://muhammedkoca.com.tr) tarafından geliştirilmiştir.

## Neler yapar?

- Normal repository, bare repository ve Git worktree keşfi
- Current branch, detached HEAD, dirty/clean status ve staged/untracked change bilgileri
- Local upstream verisine göre ahead / behind görünümü
- Repository, `.git` ve generated directory disk usage breakdown'ı
- `node_modules`, `target`, `.next`, `dist`, `build`, `coverage`, `.venv` gibi büyük directory'leri tespit etme
- Lightweight technology detection: Rust, Node.js / TypeScript, Python, PHP ve Go
- Fuzzy search ile repository name, path, branch ve remote üzerinde arama
- Local SQLite cache, groups, tags, notes ve settings altyapısı
- Güvenli folder açma ve kullanıcının açıkça başlattığı `fetch` işlemi

## Stack

| Katman | Technology |
| --- | --- |
| Core | Rust, `git2`, `walkdir` |
| CLI | Rust, `clap` |
| Storage | SQLite, `rusqlite` |
| Desktop | Tauri, React, TypeScript (strict), Tailwind CSS |
| Test / Quality | Rust tests, Clippy, GitHub Actions |

## Hızlı başlangıç

### Gereksinimler

- Rust stable toolchain
- Node.js 20+ (desktop application için)
- Git

### CLI

```powershell
# Repository root içinden
cargo run -p reposhelf -- scan C:\Projects
cargo run -p reposhelf -- list C:\Projects --json
cargo run -p reposhelf -- status C:\Projects
cargo run -p reposhelf -- dirty C:\Projects
cargo run -p reposhelf -- stale C:\Projects --days 90
cargo run -p reposhelf -- large C:\Projects --min-mb 2048
```

Bir remote'u yenilemek isterseniz bu işlem bilinçli olarak explicit'tir:

```powershell
cargo run -p reposhelf -- fetch C:\Projects\my-repository
```

### Desktop application

```powershell
cd apps/desktop
npm install
npm run tauri dev
```

Desktop app açıldıktan sonra scan root olarak örneğin `C:\Projects`, `D:\Work` veya `~/Code` girin. RepoShelf repository'leri listeler, search yapmanızı sağlar ve seçtiğiniz repository folder'ını sistem file manager'ında güvenli şekilde açar.

### Release build

```powershell
cd apps/desktop
npm run tauri build
```

## Privacy ve safety

RepoShelf local-first çalışır.

- Repository path, branch, remote veya commit metadata'sını upload etmez.
- Git credential saklamaz.
- Scan sırasında repository'leri değiştirmez.
- Otomatik `reset`, branch silme veya repository silme yapmaz.
- Sürekli/background fetch yapmaz; network access yalnızca kullanıcının istediği `fetch` action ile gerçekleşir.
- Shell command oluşturmak yerine typed path ve güvenli process argument kullanır.

Detaylar için [PRIVACY.md](PRIVACY.md) ve [SECURITY.md](SECURITY.md) dosyalarına bakın.

## Quality checks

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cd apps/desktop
npm run build
```

GitHub Actions; Windows, Linux ve macOS üzerinde Rust quality kontrollerini, ayrıca desktop frontend build'ini çalıştırır.

## Project structure

```text
crates/             Rust core, discovery, git, disk, health, search, storage, platform
apps/cli/           RepoShelf command-line application
apps/desktop/       Tauri + React desktop application
fixtures/           Test fixture alanı
docs/               Project documentation
```

## Contributing

Contributions, issue ve feature request'ler memnuniyetle karşılanır. Contribution öncesinde [CONTRIBUTING.md](CONTRIBUTING.md) içindeki quality commands'leri çalıştırın.

## License

Bu project [MIT License](LICENSE) ile lisanslanmıştır.

---

Built by [muhammedkoca.com.tr](https://muhammedkoca.com.tr) · Open source developer tooling for better repository visibility.
