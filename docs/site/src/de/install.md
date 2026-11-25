[English](../install.md) · [Polski](../pl/install.md) · **Deutsch** · [Español](../es/install.md) · [Français](../fr/install.md) · [Italiano](../it/install.md) · [Nederlands](../nl/install.md) · [Português (Brasil)](../pt-br/install.md) · [Українська](../uk/install.md)

# Installation

## Eine fertige Version herunterladen

Die neueste Version ist [{{VERSION}}]({{RELEASE_URL}}) (Vorabversionen eingeschlossen). Alle Versionen stehen auf der
[Release-Seite](https://github.com/beoArcha/oXyFlow.Solitaire/releases).

| System | Archiv |
|---|---|
| Windows (x86-64) | `oxysol-<version>-x86_64-pc-windows-msvc.zip` |
| Linux (x86-64) | `oxysol-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `oxysol-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS (Apple silicon) | `oxysol-<version>-aarch64-apple-darwin.tar.gz` |

Entpacke das Archiv und starte `oxysol` (unter Windows `oxysol.exe`) in einem Terminal. Die Archive mit `-portable` im
Namen legen die Dateien des Spiels in einem Ordner `data` neben dem Programm ab. Die Linux-Versionen haben keinen Ton.
Mit `SHA256SUMS` auf der Release-Seite kannst du einen Download prüfen.

## Aus dem Quellcode bauen

Gebaut wird der Branch `release`. Er enthält den letzten Commit, der die vollständige Prüfung unter Linux, Windows und
macOS bestanden hat.

### Mit den Installationsskripten (empfohlen)

Die Skripte im Ordner `build/` prüfen zuerst alle benötigten Abhängigkeiten und bauen und installieren `oxysol` erst
danach in `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` unter Windows):

- `--check` — hält bei der ersten fehlenden Abhängigkeit mit einem Exit-Code ungleich null an.
- `--list` — listet alle fehlenden Abhängigkeiten und den Installationsbefehl für dein System auf.
- `--install` — fragt vor der Installation fehlender Abhängigkeiten über deinen Paketmanager nach, prüft erneut und baut
  nur, wenn die zweite Prüfung besteht.
- `--no-audio` — überspringt unter Linux die Prüfung von ALSA/`pkg-config` und baut mit `--no-default-features`.

Ohne Modus-Flag fragt ein interaktives Terminal, welcher Modus benutzt werden soll; ein nicht interaktiver Lauf verhält
sich wie `--list`.

**Linux und macOS** (`--check` und `--list` in der CI geprüft; `--install` noch nicht geprüft):

```sh
curl -fsSL https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.sh | sh -s -- --list
```

**Windows (PowerShell)** (`--check` und `--list` unter Windows geprüft; `--install` noch nicht geprüft):

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.ps1))) --list
```

Ersetze `--list` durch `--install`, um fehlende Abhängigkeiten zu installieren und zu bauen.

### Manuelle Schritte

**Rust wird benötigt.** Installiere es mit [rustup](https://rustup.rs). Die genaue Toolchain ist in
`rust-toolchain.toml` festgelegt; rustup erkennt sie von selbst und installiert sie beim ersten Build, weitere
Einrichtung ist nicht nötig. Der Build kompiliert auch ein wenig C-Code (SQLite), daher wird eine C-Toolchain gebraucht:

- **Windows:** die Visual Studio Build Tools (oder Visual Studio) mit der Workload „Desktopentwicklung mit C++“
  („Desktop development with C++“). rustup bietet an, sie zu installieren.
- **macOS:** die Xcode-Befehlszeilentools: `xcode-select --install`.
- **Linux:** ein C-Compiler (`gcc` oder `cc`), `curl` und `tar`. Für Ton zusätzlich `pkg-config` und die ALSA-Header,
  zum Beispiel `sudo apt install build-essential pkg-config libasound2-dev` unter Debian und Ubuntu. Ohne sie baust du mit
  `cargo build --release --no-default-features`, dann ohne Ton.

Der erste Build dauert ein paar Minuten.

### Windows (PowerShell)

Geprüft unter Windows 10 am 19.11.2026.

```powershell
curl.exe -L -o oxysol-source.zip https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.zip
Expand-Archive oxysol-source.zip -DestinationPath oxysol-source
cd oxysol-source\oXyFlow.Solitaire-release
cargo build --release
.\target\release\oxysol.exe
```

### Linux

Noch nicht geprüft: Die Befehle sind aufgeschrieben, aber nicht ausprobiert.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### macOS

Noch nicht geprüft: Die Befehle sind aufgeschrieben, aber nicht ausprobiert.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### Wo das Programm landet

Der Build schreibt das Programm nach `target/release/oxysol` (unter Windows `target\release\oxysol.exe`) im entpackten
Ordner. Starte es dort oder kopiere es in einen Ordner in deinem `PATH`.

### Alternative: cargo install

Wenn du magst, kann cargo das Programm in einem Schritt holen, bauen und installieren. Die Voraussetzungen sind
dieselben wie oben.

```sh
cargo install --git https://github.com/beoArcha/oXyFlow.Solitaire --branch release --locked oxysol
```

Damit landet `oxysol` im `bin`-Ordner von cargo (`~/.cargo/bin`, unter Windows `%USERPROFILE%\.cargo\bin`), den rustup
zu deinem `PATH` hinzufügt. Starte dann `oxysol`. Geprüft unter Windows 10 am 19.11.2026; unter Linux und macOS noch
nicht.

Zum Aktualisieren wiederholst du die Schritte: Der Download enthält immer den neuesten Stand des Branches `release`.
