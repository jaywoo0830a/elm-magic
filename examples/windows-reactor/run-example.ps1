<#
.SYNOPSIS
    windows-reactor 예제 **하나만** 컴파일해서 실제 WinUI 창으로 띄운다.

.DESCRIPTION
    `run-tests.ps1` 은 20개 전부를 **테스트**한다. 이 스크립트는 반대로 고른
    **하나만** 빌드하고 실행한다 — 창을 닫으면 끝난다.

    하는 일은 이 한 줄과 같다(저장소 루트에서):
        cargo run --example 18_shell_layout --features winui --manifest-path examples\windows-reactor\Cargo.toml

    Windows 전용이다(WinUI 3 / Windows App SDK 런타임 필요).

.PARAMETER Example
    예제 번호 또는 이름(접두어 일치). 예: `01`, `18`, `20_app_skeleton`.
    생략하면 목록을 보여주고 번호를 물어본다.

.PARAMETER Release
    `--release` 로 빌드한다(첫 빌드는 느리지만 화면이 더 부드럽다).

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File examples\windows-reactor\run-example.ps1 01
    powershell -ExecutionPolicy Bypass -File examples\windows-reactor\run-example.ps1 -Example 18
    powershell -ExecutionPolicy Bypass -File examples\windows-reactor\run-example.ps1 20 -Release
#>
[CmdletBinding()]
param(
    [Parameter(Position = 0)][string] $Example,
    [switch] $Release
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# cargo/rustc의 한글 출력이 콘솔에서 깨지지 않게 맞춘다.
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
    throw 'windows-reactor 예제는 Windows 전용이다 (WinUI 3 / Windows App SDK).'
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

function Get-ExampleTitle {
    param([string] $Name)
    $first = Get-Content -LiteralPath (Join-Path $here "$Name.rs") -TotalCount 1 -Encoding UTF8
    # `//! windows-reactor 예제 09 — 제목` → 제목
    if ($first -match '—\s*(.+)$') { return $Matches[1].Trim() }
    return ($first -replace '^//!\s*', '').Trim()
}

if ([string]::IsNullOrEmpty($Example)) {
    Write-Host "예제 $($registered.Count)개:" -ForegroundColor Green
    foreach ($name in $registered) {
        Write-Host ('  {0,-28} {1}' -f $name, (Get-ExampleTitle -Name $name))
    }
    $Example = Read-Host '번호 또는 이름 (예: 01)'
    if ([string]::IsNullOrWhiteSpace($Example)) { throw '예제를 고르지 않았다.' }
}

$hits = @($registered | Where-Object { $_ -eq $Example -or $_ -like "$Example*" })
if ($hits.Count -eq 0) { throw "그런 예제가 없다: $Example  (인자 없이 실행하면 목록을 본다)" }
if ($hits.Count -gt 1) { throw "여러 예제가 걸린다: $($hits -join ', ') — 더 길게 지정한다" }
$target = $hits[0]

Write-Host ''
Write-Host "예제 $target — $(Get-ExampleTitle -Name $target)" -ForegroundColor Green
Write-Host '컴파일 후 WinUI 창이 뜬다. 창을 닫으면 끝난다.' -ForegroundColor DarkGray

$cargoArgs = @('run', '--example', $target, '--features', 'winui', '--manifest-path', $manifest)
if ($Release) { $cargoArgs += '--release' }
Write-Host ('> cargo ' + ($cargoArgs -join ' ')) -ForegroundColor Cyan
& cargo @cargoArgs
exit $LASTEXITCODE
