$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$RepoTarballUrl = 'https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz'
$WingetRustupId = 'Rustlang.Rustup'
$WingetBuildToolsId = 'Microsoft.VisualStudio.2022.BuildTools'

$script:Mode = ''
$script:NoAudio = $false
$script:MissingCurl = $false
$script:MissingTar = $false
$script:MissingRust = $false
$script:MissingMsvc = $false
$script:MissingCount = 0
$script:HasWinget = $false

function Write-Line {
    param(
        [string]$Text = '',
        [switch]$ErrorStream
    )
    if ($ErrorStream) {
        [Console]::Error.WriteLine($Text)
    } else {
        [Console]::Out.WriteLine($Text)
    }
}

function Show-Usage {
    Write-Line 'Usage: install.ps1 [MODE] [--no-audio] [--help]'
    Write-Line ''
    Write-Line 'Modes:'
    Write-Line '  --check, -Check       Check dependencies and stop at the first problem (non-zero exit).'
    Write-Line '  --list, -List         List every missing dependency and the command to install it.'
    Write-Line '  --install, -Install   Install missing dependencies (after asking), re-check, then build'
    Write-Line '                        and install oxysol from the release branch into %USERPROFILE%\.cargo\bin.'
    Write-Line ''
    Write-Line 'Options:'
    Write-Line '  --no-audio, -NoAudio  Build with --no-default-features.'
    Write-Line '  -h, --help, -Help     Show this help message.'
    Write-Line ''
    Write-Line 'Without a mode flag, an interactive terminal asks which mode to run;'
    Write-Line 'a non-interactive run behaves like --list.'
}

function Set-InstallMode {
    param([string]$SelectedMode)
    if ($script:Mode -ne '' -and $script:Mode -ne $SelectedMode) {
        Write-Line "Conflicting mode flags: --$($script:Mode) and --$SelectedMode" -ErrorStream
        exit 2
    }
    $script:Mode = $SelectedMode
}

function Read-Arguments {
    param([string[]]$CliArgs)
    foreach ($arg in $CliArgs) {
        switch ($arg.ToLowerInvariant()) {
            '--check' { Set-InstallMode 'check' }
            '-check' { Set-InstallMode 'check' }
            '--list' { Set-InstallMode 'list' }
            '-list' { Set-InstallMode 'list' }
            '--install' { Set-InstallMode 'install' }
            '-install' { Set-InstallMode 'install' }
            '--no-audio' { $script:NoAudio = $true }
            '-noaudio' { $script:NoAudio = $true }
            '-h' { Show-Usage; exit 0 }
            '--help' { Show-Usage; exit 0 }
            '-help' { Show-Usage; exit 0 }
            default {
                Write-Line "Unknown option: $arg" -ErrorStream
                Show-Usage
                exit 2
            }
        }
    }
}

function Add-CargoToPath {
    $cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
    $cargoBin = Join-Path $cargoHome 'bin'
    if (Test-Path -LiteralPath $cargoBin) {
        $entries = $env:PATH -split ';'
        if ($entries -notcontains $cargoBin) {
            $env:PATH = "$cargoBin;$env:PATH"
        }
    }
}

function Test-MsvcLink {
    $pathLinks = @(Get-Command link.exe -All -ErrorAction SilentlyContinue | Where-Object {
        $_.Source -notmatch '\\usr\\bin\\link\.exe$' -and $_.Source -notmatch '\\Git\\'
    })
    if ($pathLinks.Count -gt 0) {
        return $true
    }

    $pf86 = [Environment]::GetFolderPath('ProgramFilesX86')
    if (-not [string]::IsNullOrEmpty($pf86)) {
        $vswhere = Join-Path $pf86 'Microsoft Visual Studio\Installer\vswhere.exe'
        if (Test-Path -LiteralPath $vswhere) {
            $installPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
            if (-not [string]::IsNullOrEmpty($installPath)) {
                $vcTools = Join-Path $installPath 'VC\Tools\MSVC'
                if (Test-Path -LiteralPath $vcTools) {
                    $found = Get-ChildItem -Path $vcTools -Filter 'link.exe' -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
                    if ($null -ne $found) {
                        return $true
                    }
                }
            }
        }
    }

    return $false
}

function Test-Dependencies {
    Add-CargoToPath

    $script:MissingCurl = $false
    $script:MissingTar = $false
    $script:MissingRust = $false
    $script:MissingMsvc = $false
    $script:MissingCount = 0

    $script:HasWinget = ($null -ne (Get-Command winget.exe -ErrorAction SilentlyContinue))

    if ($null -eq (Get-Command curl.exe -ErrorAction SilentlyContinue)) {
        $script:MissingCurl = $true
        $script:MissingCount++
    }

    if ($null -eq (Get-Command tar.exe -ErrorAction SilentlyContinue)) {
        $script:MissingTar = $true
        $script:MissingCount++
    }

    $hasRustup = ($null -ne (Get-Command rustup.exe -ErrorAction SilentlyContinue))
    $hasCargo = ($null -ne (Get-Command cargo.exe -ErrorAction SilentlyContinue))
    if (-not $hasRustup -or -not $hasCargo) {
        $script:MissingRust = $true
        $script:MissingCount++
    }

    if (-not (Test-MsvcLink)) {
        $script:MissingMsvc = $true
        $script:MissingCount++
    }
}

function Invoke-CheckMode {
    if ($script:MissingCurl) {
        Write-Line 'Missing dependency: curl.exe' -ErrorStream
        exit 1
    }
    if ($script:MissingTar) {
        Write-Line 'Missing dependency: tar.exe' -ErrorStream
        exit 1
    }
    if ($script:MissingMsvc) {
        Write-Line 'Missing dependency: MSVC Build Tools with a working link.exe' -ErrorStream
        exit 1
    }
    if ($script:MissingRust) {
        Write-Line 'Missing dependency: Rust toolchain (rustup and cargo)' -ErrorStream
        exit 1
    }
    Write-Line 'All build dependencies are present.'
}

function Invoke-ListMode {
    if ($script:MissingCount -eq 0) {
        Write-Line 'All build dependencies are present.'
        return
    }

    Write-Line 'Missing build dependencies:'
    if ($script:MissingCurl) { Write-Line '  - curl.exe (download tool)' }
    if ($script:MissingTar) { Write-Line '  - tar.exe (archive extractor)' }
    if ($script:MissingMsvc) { Write-Line '  - MSVC Build Tools with C++ workload (link.exe)' }
    if ($script:MissingRust) { Write-Line '  - Rust (rustup / cargo)' }

    Write-Line ''
    Write-Line 'Install commands:'
    if ($script:HasWinget) {
        if ($script:MissingCurl) {
            Write-Line '  winget install --id cURL.cURL -e --source winget'
        }
        if ($script:MissingTar) {
            Write-Line '  winget install --id GnuWin32.Tar -e --source winget'
        }
        if ($script:MissingMsvc) {
            Write-Line "  winget install --id $WingetBuildToolsId -e --source winget --override `"--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended`""
        }
        if ($script:MissingRust) {
            Write-Line "  winget install --id $WingetRustupId -e --source winget"
        }
    } else {
        Write-Line 'winget is not available. Install manually:'
        if ($script:MissingCurl -or $script:MissingTar) {
            Write-Line '  - Update Windows 10/11 (curl.exe and tar.exe are built in) or install curl and tar.'
        }
        if ($script:MissingMsvc) {
            Write-Line '  - Visual Studio Build Tools ("Desktop development with C++" workload): https://visualstudio.microsoft.com/visual-cpp-build-tools/'
        }
        if ($script:MissingRust) {
            Write-Line '  - Rust (rustup-init.exe): https://rustup.rs'
        }
    }
}

function Test-Interactive {
    if (-not [Environment]::UserInteractive) {
        return $false
    }
    try {
        return (-not [Console]::IsInputRedirected)
    } catch {
        return $true
    }
}

function Read-YesNo {
    param([string]$Prompt)
    $answer = Read-Host "$Prompt [y/N]"
    return ($answer -match '^(y|yes)$')
}

function Install-MissingWithWinget {
    if (-not $script:HasWinget) {
        Invoke-ListMode
        exit 1
    }

    if ($script:MissingCurl) {
        & winget.exe install --id cURL.cURL -e --source winget
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    if ($script:MissingTar) {
        & winget.exe install --id GnuWin32.Tar -e --source winget
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    if ($script:MissingMsvc) {
        & winget.exe install --id $WingetBuildToolsId -e --source winget --override '--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    if ($script:MissingRust) {
        & winget.exe install --id $WingetRustupId -e --source winget
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    $machinePath = [Environment]::GetEnvironmentVariable('PATH', 'Machine')
    $userPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
    $env:PATH = "$userPath;$machinePath"
    Add-CargoToPath
}

function Invoke-BuildAndInstall {
    if ($script:NoAudio) {
        Write-Line 'Building and installing oxysol without audio (--no-default-features)...'
    } else {
        Write-Line 'Building and installing oxysol...'
    }

    $workDir = Join-Path ([IO.Path]::GetTempPath()) ("oxysol-install-" + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $workDir | Out-Null
    try {
        $tarball = Join-Path $workDir 'oxysol-source.tar.gz'
        & curl.exe -fsSL -o $tarball $RepoTarballUrl
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        & tar.exe -xzf $tarball -C $workDir
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        $srcDir = Join-Path $workDir 'oXyFlow.Solitaire-release'
        if (-not (Test-Path -LiteralPath $srcDir)) {
            Write-Line "Expected source directory not found: $srcDir" -ErrorStream
            exit 1
        }

        if ($script:NoAudio) {
            & cargo.exe install --path $srcDir --locked --no-default-features
        } else {
            & cargo.exe install --path $srcDir --locked
        }
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        $cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
        $binPath = Join-Path (Join-Path $cargoHome 'bin') 'oxysol.exe'
        Write-Line "oxysol installed to $binPath"
    } finally {
        if (Test-Path -LiteralPath $workDir) {
            Remove-Item -LiteralPath $workDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

function Invoke-InstallMode {
    if ($script:MissingCount -gt 0) {
        Invoke-ListMode
        if (-not (Test-Interactive)) {
            Write-Line 'Non-interactive session: refusing to install without confirmation.' -ErrorStream
            exit 1
        }
        if (-not (Read-YesNo 'Install the missing dependencies listed above?')) {
            Write-Line 'Cancelled.'
            exit 1
        }

        Install-MissingWithWinget
        Test-Dependencies
        if ($script:MissingCount -gt 0) {
            Write-Line 'Dependencies are still missing after installation:' -ErrorStream
            Invoke-ListMode
            exit 1
        }
    }

    Invoke-BuildAndInstall
}

function Select-InteractiveMode {
    Write-Line 'oXySol dependency check and installer'
    Write-Line '  1) Check dependencies (--check)'
    Write-Line '  2) List missing dependencies and install commands (--list)'
    Write-Line '  3) Install missing dependencies, then build and install (--install)'
    $choice = Read-Host 'Choose [1-3] (default 2)'
    switch ($choice) {
        '1' { $script:Mode = 'check' }
        '' { $script:Mode = 'list' }
        '2' { $script:Mode = 'list' }
        '3' { $script:Mode = 'install' }
        default {
            Write-Line "Invalid choice: $choice" -ErrorStream
            exit 2
        }
    }
}

Read-Arguments -CliArgs $args

if ($script:Mode -eq '') {
    if (Test-Interactive) {
        Select-InteractiveMode
    } else {
        $script:Mode = 'list'
    }
}

Test-Dependencies

switch ($script:Mode) {
    'check' { Invoke-CheckMode }
    'list' { Invoke-ListMode }
    'install' { Invoke-InstallMode }
}
