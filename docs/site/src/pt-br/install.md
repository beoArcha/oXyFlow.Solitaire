[English](../install.md) · [Polski](../pl/install.md) · [Deutsch](../de/install.md) · [Español](../es/install.md) · [Français](../fr/install.md) · [Italiano](../it/install.md) · [Nederlands](../nl/install.md) · **Português (Brasil)** · [Українська](../uk/install.md)

# Instalação

## Baixe uma versão pronta

A versão mais recente é [{{VERSION}}]({{RELEASE_URL}}) (incluindo as de pré-lançamento). Todas as versões estão na
[página de lançamentos](https://github.com/beoArcha/oXyFlow.Solitaire/releases).

| Sistema | Arquivo |
|---|---|
| Windows (x86-64) | `oxysol-<version>-x86_64-pc-windows-msvc.zip` |
| Linux (x86-64) | `oxysol-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `oxysol-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS (Apple silicon) | `oxysol-<version>-aarch64-apple-darwin.tar.gz` |

Descompacte o arquivo e execute `oxysol` (`oxysol.exe` no Windows) em um terminal. Os arquivos com `-portable` no nome
guardam os dados do jogo em uma pasta `data` ao lado do programa. As versões para Linux não têm som. O arquivo
`SHA256SUMS` na página do lançamento permite conferir um download.

## Compile a partir do código-fonte

Você compila o branch `release`, que contém o último commit aprovado na verificação completa no Linux, Windows e
macOS.

### Com os scripts de instalação (recomendado)

Os scripts na pasta `build/` verificam primeiro todas as dependências necessárias e só então compilam e instalam o
`oxysol` em `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` no Windows):

- `--check` — para na primeira dependência ausente, com código de saída diferente de zero.
- `--list` — lista todas as dependências ausentes e o comando de instalação para o seu sistema.
- `--install` — pede confirmação antes de instalar as dependências ausentes pelo gerenciador de pacotes, verifica de novo
  e compila somente se a segunda verificação passar.
- `--no-audio` — pula no Linux a verificação de ALSA/`pkg-config` e compila com `--no-default-features`.

Sem uma flag de modo, um terminal interativo pergunta qual modo usar; uma execução não interativa age como `--list`.

**Linux e macOS** (`--check` e `--list` verificados no CI; `--install` ainda não verificado):

```sh
curl -fsSL https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.sh | sh -s -- --list
```

**Windows (PowerShell)** (`--check` e `--list` verificados no Windows; `--install` ainda não verificado):

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.ps1))) --list
```

Troque `--list` por `--install` para instalar as dependências ausentes e compilar.

### Passos manuais

**É preciso ter Rust.** Instale-o com o [rustup](https://rustup.rs). A toolchain exata está fixada em
`rust-toolchain.toml`; o rustup a detecta sozinho e a instala na primeira compilação, então nenhuma outra configuração é
necessária. A compilação também inclui um pouco de código C (SQLite), por isso é preciso uma toolchain de C:

- **Windows:** as Build Tools do Visual Studio (ou o Visual Studio) com a carga de trabalho “Desenvolvimento para
  desktop com C++” (“Desktop development with C++”). O rustup oferece a instalação.
- **macOS:** as ferramentas de linha de comando do Xcode: `xcode-select --install`.
- **Linux:** um compilador C (`gcc` ou `cc`), `curl` e `tar`. Para ter som, também `pkg-config` e os cabeçalhos do ALSA,
  por exemplo `sudo apt install build-essential pkg-config libasound2-dev` no Debian e no Ubuntu. Sem eles, compile com
  `cargo build --release --no-default-features`, que deixa o som de fora.

A primeira compilação leva alguns minutos.

### Windows (PowerShell)

Verificado no Windows 10 em 19/11/2026.

```powershell
curl.exe -L -o oxysol-source.zip https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.zip
Expand-Archive oxysol-source.zip -DestinationPath oxysol-source
cd oxysol-source\oXyFlow.Solitaire-release
cargo build --release
.\target\release\oxysol.exe
```

### Linux

Ainda não verificado: os comandos foram escritos, mas não executados.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### macOS

Ainda não verificado: os comandos foram escritos, mas não executados.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### Onde o programa fica

A compilação grava o programa em `target/release/oxysol` (`target\release\oxysol.exe` no Windows) dentro da pasta
descompactada. Execute-o de lá ou copie-o para uma pasta do seu `PATH`.

### Alternativa: cargo install

Se preferir, o cargo pode baixar, compilar e instalar o programa em um só passo. Os requisitos são os mesmos de cima.

```sh
cargo install --git https://github.com/beoArcha/oXyFlow.Solitaire --branch release --locked oxysol
```

Isso coloca o `oxysol` na pasta `bin` do cargo (`~/.cargo/bin`, ou `%USERPROFILE%\.cargo\bin` no Windows), que o rustup
adiciona ao seu `PATH`. Depois execute `oxysol`. Verificado no Windows 10 em 19/11/2026; no Linux e no macOS ainda não
foi verificado.

Para atualizar, repita os passos: o download sempre traz a versão mais recente do branch `release`.
