[English](../install.md) · [Polski](../pl/install.md) · [Deutsch](../de/install.md) · [Español](../es/install.md) · **Français** · [Italiano](../it/install.md) · [Nederlands](../nl/install.md) · [Português (Brasil)](../pt-br/install.md) · [Українська](../uk/install.md)

# Installation

## Télécharger une version prête à l’emploi

La version la plus récente est [{{VERSION}}]({{RELEASE_URL}}) (préversions comprises). Toutes les versions sont sur la
[page des publications](https://github.com/beoArcha/oXyFlow.Solitare/releases).

| Système | Archive |
|---|---|
| Windows (x86-64) | `oxysol-<version>-x86_64-pc-windows-msvc.zip` |
| Linux (x86-64) | `oxysol-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `oxysol-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS (Apple silicon) | `oxysol-<version>-aarch64-apple-darwin.tar.gz` |

Décompressez l’archive et lancez `oxysol` (`oxysol.exe` sous Windows) dans un terminal. Les archives dont le nom
contient `-portable` gardent les fichiers du jeu dans un dossier `data` à côté du programme. Les versions Linux n’ont
pas de son. Le fichier `SHA256SUMS` de la page de publication permet de vérifier un téléchargement.

## Compiler depuis les sources

On compile la branche `release`, qui contient le dernier commit ayant passé la vérification complète sous Linux,
Windows et macOS.

**Rust est nécessaire.** Installez-le avec [rustup](https://rustup.rs). La chaîne d’outils exacte est fixée dans
`rust-toolchain.toml` ; rustup la détecte tout seul et l’installe lors de la première compilation, aucune autre
configuration n’est donc nécessaire. La compilation inclut aussi un peu de code C (SQLite), il faut donc une chaîne
d’outils C :

- **Windows :** les Build Tools de Visual Studio (ou Visual Studio) avec la charge de travail « Développement Desktop
  en C++ » (« Desktop development with C++ »). rustup propose de les installer.
- **macOS :** les outils en ligne de commande Xcode : `xcode-select --install`.
- **Linux :** un compilateur C (`gcc` ou `cc`) et `curl`. Pour le son, il faut aussi les en-têtes ALSA, par exemple
  `sudo apt install build-essential libasound2-dev` sous Debian et Ubuntu. Sans eux, compilez avec
  `cargo build --release --no-default-features`, qui laisse le son de côté.

La première compilation prend quelques minutes.

### Windows (PowerShell)

Vérifié sous Windows 10 le 19/11/2026.

```powershell
curl.exe -L -o oxysol-source.zip https://github.com/beoArcha/oXyFlow.Solitare/archive/refs/heads/release.zip
Expand-Archive oxysol-source.zip -DestinationPath oxysol-source
cd oxysol-source\oXyFlow.Solitare-release
cargo build --release
.\target\release\oxysol.exe
```

### Linux

Pas encore vérifié : les commandes sont écrites, mais n’ont pas été exécutées.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitare/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitare-release
cargo build --release
./target/release/oxysol
```

### macOS

Pas encore vérifié : les commandes sont écrites, mais n’ont pas été exécutées.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitare/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitare-release
cargo build --release
./target/release/oxysol
```

### Où se trouve le programme

La compilation place le programme dans `target/release/oxysol` (`target\release\oxysol.exe` sous Windows), à
l’intérieur du dossier décompressé. Lancez-le depuis là, ou copiez-le dans un dossier de votre `PATH`.

### Autre possibilité : cargo install

Si vous préférez, cargo peut récupérer, compiler et installer le programme en une seule étape. Les prérequis sont les
mêmes que ci-dessus.

```sh
cargo install --git https://github.com/beoArcha/oXyFlow.Solitare --branch release --locked oxysol
```

`oxysol` est alors placé dans le dossier `bin` de cargo (`~/.cargo/bin`, ou `%USERPROFILE%\.cargo\bin` sous Windows),
que rustup ajoute à votre `PATH`. Lancez ensuite `oxysol`. Vérifié sous Windows 10 le 19/11/2026 ; pas encore vérifié
sous Linux et macOS.

Pour mettre à jour, refaites les mêmes étapes : le téléchargement contient toujours la version la plus récente de la
branche `release`.
