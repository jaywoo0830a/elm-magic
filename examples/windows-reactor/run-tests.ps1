<#
.SYNOPSIS
    windows-reactor 참고 예제 20개를 Windows에서 테스트하거나 실행한다.

.DESCRIPTION
    `examples/windows-reactor/*.rs` 는 `fn main()` 을 가진 **독립 프로그램**이라
    루트 워크스페이스의 Cargo 타겟이 아니었다. 이 스크립트는 같은 디렉터리의
    `Cargo.toml`(예제 전용 패키지)을 통해 각 파일을
    `cargo test --example <이름>` / `cargo run --example <이름>` 으로 돌린다.

    예제 타겟은 `winui` 기능으로 게이트되어 있어 **Windows에서만** 컴파일된다
    (WinUI 3 / Windows App SDK 런타임 필요). 리눅스·macOS 에서는 이 스크립트가
    아무것도 하지 않고 안내만 출력한다 — 그쪽에서는 루트 `cargo test --workspace`.

    스크립트 없이 같은 일을 하는 한 줄(저장소 루트에서):
        cargo test --examples --features winui --manifest-path examples\windows-reactor\Cargo.toml
        cargo run  --example 09_async_effects --features winui --manifest-path examples\windows-reactor\Cargo.toml

    **하나만 창으로 보고 싶으면** 같은 디렉터리의 `run-example.ps1` 을 쓴다 —
    `-Run` 과 같지만 목록/테스트 타겟을 거치지 않고 고른 예제만 빌드·실행한다:
        .\run-example.ps1 18

.PARAMETER Example
    예제 번호 또는 이름(접두어 일치). 예: `-Example 09`, `-Example 20_app_skeleton`.
    생략하면 20개 전부.

.PARAMETER Filter
    테스트 이름 필터. 하네스에 `-- <필터>` 로 전달된다(예: `-Filter counter`).

.PARAMETER Run
    테스트 대신 **실제 WinUI 창**을 띄운다(`-Example` 필요).

.PARAMETER Check
    테스트하지 않고 컴파일만 확인한다(`cargo check --examples`).

.PARAMETER List
    예제 목록과 각 파일의 첫 `//!` 줄(제목)을 출력한다. 매니페스트에 빠진 파일이
    있으면 경고한다.

.PARAMETER Fmt
    예제 파일들에 `cargo fmt` 를 **적용**한다. 기본 예제 서식은 손으로 맞춘
    정렬 주석을 보존하므로 rustfmt 대상이 아니다 — 필요할 때만 쓴다.
    (검사만: `cargo fmt --check --manifest-path examples\windows-reactor\Cargo.toml`)

.PARAMETER Release
    `--release` 로 빌드한다.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File examples\windows-reactor\run-tests.ps1
    powershell -ExecutionPolicy Bypass -File examples\windows-reactor\run-tests.ps1 -Example 09
    powershell -ExecutionPolicy Bypass -File examples\windows-reactor\run-tests.ps1 -Example 20 -Run
    powershell -ExecutionPolicy Bypass -File examples\windows-reactor\run-tests.ps1 -List
#>
[CmdletBinding()]
param(
    [string] $Example,
    [string] $Filter,
    [switch] $Run,
    [switch] $Check,
    [switch] $List,
    [switch] $Fmt,
    [switch] $Release
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# cargo/rustc의 한글 출력(테스트 이름 등)이 콘솔에서 깨지지 않게 맞춘다.
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    $OutputEncoding = [System.Text.Encoding]::UTF8
} catch {
    # 콘솔이 UTF-8을 거부하면(드묾) 그냥 진행한다.
}

$here = $PSScriptRoot
if ([string]::IsNullOrEmpty($here)) {
    $here = Split-Path -Parent $MyInvocation.MyCommand.Path
}
$manifest = Join-Path $here 'Cargo.toml'
if (-not (Test-Path -LiteralPath $manifest)) {
    throw "예제 패키지 매니페스트를 찾을 수 없다: $manifest"
}

$onWindows = $true
if ($PSVersionTable.PSEdition -eq 'Core') { $onWindows = $IsWindows }
if (-not $onWindows) {
    Write-Warning 'windows-reactor 예제는 Windows 전용이다 (WinUI 3 / Windows App SDK).'
    Write-Warning '리눅스/macOS 에서는 예제가 컴파일되지 않는다 — 루트 `cargo test --workspace` 를 쓴다.'
    exit 2
}

if (-not (Get-Command -Name cargo -ErrorAction SilentlyContinue)) {
    throw 'cargo 를 찾을 수 없다 (rustup 설치 후 새 셸에서 다시 실행).'
}

# 매니페스트의 `[[example]] path = "NN_*.rs"` 가 목록의 정본이다.
$registered = @(
    Select-String -LiteralPath $manifest -Pattern '^\s*path\s*=\s*"(.+?)\.rs"\s*$' |
        ForEach-Object { $_.Matches[0].Groups[1].Value } |
        Sort-Object
)
$files = @(
    Get-ChildItem -LiteralPath $here -Filter '*.rs' -File |
        ForEach-Object { $_.BaseName } |
        Sort-Object
)

function Get-ExampleTitle {
    param([string] $Name)
    $first = Get-Content -LiteralPath (Join-Path $here "$Name.rs") -TotalCount 1 -Encoding UTF8
    # `//! windows-reactor 예제 09 — 제목` → 제목
    if ($first -match '—\s*(.+)$') { return $Matches[1].Trim() }
    return ($first -replace '^//!\s*', '').Trim()
}

function Resolve-ExampleName {
    param([string] $Needle)
    $hits = @($registered | Where-Object { $_ -eq $Needle -or $_ -like "$Needle*" })
    if ($hits.Count -eq 0) { throw "그런 예제가 없다: $Needle  (-List 로 목록 확인)" }
    if ($hits.Count -gt 1) { throw "여러 예제가 걸린다: $($hits -join ', ') — 더 길게 지정한다" }
    return $hits[0]
}

function Invoke-CargoStep {
    param(
        [string[]] $CargoArgs,
        # `--` 뒤(= 테스트 하네스)로 넘길 인자. `--manifest-path`는 **반드시**
        # `--` 앞에 있어야 한다 — 뒤에 두면 cargo가 아니라 libtest가 받아서
        # "Unrecognized option"으로 죽는다.
        [string[]] $HarnessArgs
    )
    $argv = @($CargoArgs) + @('--manifest-path', $manifest)
    if ($HarnessArgs -and $HarnessArgs.Count -gt 0) {
        $argv += @('--') + $HarnessArgs
    }
    Write-Host ('> cargo ' + ($argv -join ' ')) -ForegroundColor Cyan
    & cargo @argv
    if ($LASTEXITCODE -ne 0) {
        Write-Host "cargo 실패 (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

if ($List) {
    Write-Host "windows-reactor 예제 $($registered.Count)개 — $here" -ForegroundColor Green
    foreach ($name in $registered) {
        Write-Host ('  {0,-28} {1}' -f $name, (Get-ExampleTitle -Name $name))
    }
    $unregistered = @($files | Where-Object { $registered -notcontains $_ })
    $stale = @($registered | Where-Object { $files -notcontains $_ })
    if ($unregistered.Count -gt 0) {
        Write-Warning ('매니페스트에 없다: {0} — Cargo.toml 에 [[example]] 추가 필요' -f ($unregistered -join ', '))
    }
    if ($stale.Count -gt 0) {
        Write-Warning ('파일이 없는 등록 항목: {0}' -f ($stale -join ', '))
    }
    Write-Host ''
    Write-Host '전체 테스트 : powershell -ExecutionPolicy Bypass -File .\run-tests.ps1'
    Write-Host '하나만      : powershell -ExecutionPolicy Bypass -File .\run-tests.ps1 -Example 09'
    Write-Host '창 띄우기   : powershell -ExecutionPolicy Bypass -File .\run-tests.ps1 -Example 09 -Run'
    Write-Host '창 하나만   : .\run-example.ps1 09   (그 예제만 빌드·실행 — run-example.ps1)'
    exit 0
}

if ($Fmt) {
    Invoke-CargoStep -CargoArgs @('fmt')
    Write-Host 'rustfmt 적용 완료 (예제 파일 서식)' -ForegroundColor Green
    exit 0
}

if ($Check) {
    Invoke-CargoStep -CargoArgs @('check', '--examples', '--features', 'winui')
    Write-Host 'cargo check: 예제 컴파일 OK' -ForegroundColor Green
    exit 0
}

if ($Run) {
    if ([string]::IsNullOrEmpty($Example)) {
        throw '-Run 은 -Example 과 함께 쓴다 (예: -Example 09 -Run)'
    }
    $target = Resolve-ExampleName -Needle $Example
    $cargoArgs = @('run', '--example', $target, '--features', 'winui')
    if ($Release) { $cargoArgs += '--release' }
    Invoke-CargoStep -CargoArgs $cargoArgs
    exit 0
}

$cargoArgs = @('test', '--features', 'winui')
if ([string]::IsNullOrEmpty($Example)) {
    $cargoArgs += '--examples'
    Write-Host "windows-reactor 예제 $($registered.Count)개 — 테스트 실행" -ForegroundColor Green
} else {
    $target = Resolve-ExampleName -Needle $Example
    $cargoArgs += @('--example', $target)
    Write-Host "예제 $target — 테스트 실행" -ForegroundColor Green
}
if ($Release) { $cargoArgs += '--release' }

if ([string]::IsNullOrEmpty($Filter)) {
    Invoke-CargoStep -CargoArgs $cargoArgs
} else {
    Invoke-CargoStep -CargoArgs $cargoArgs -HarnessArgs @($Filter)
}
Write-Host '통과 — 예제 테스트 성공' -ForegroundColor Green
exit 0
