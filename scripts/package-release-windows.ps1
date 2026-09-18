[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [ValidateSet('x64', 'arm64')]
    [string]$Architecture,
    [Parameter(Mandatory = $true)]
    [string]$TargetDirectory,
    [Parameter(Mandatory = $true)]
    [string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'

New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null

$targetBundle = if ($Architecture -eq 'x64') {
    Join-Path $TargetDirectory 'release\bundle'
} else {
    Join-Path $TargetDirectory 'aarch64-pc-windows-msvc\release\bundle'
}

function Get-SingleBundleFile([string]$Directory, [string]$Pattern) {
    $files = @(Get-ChildItem -LiteralPath $Directory -File -Filter $Pattern | Sort-Object Name)
    if ($files.Count -ne 1) {
        throw "Expected exactly one $Pattern in $Directory, found $($files.Count)."
    }
    return $files[0].FullName
}

$installerExe = Get-SingleBundleFile (Join-Path $targetBundle 'nsis') '*.exe'
$msi = Get-SingleBundleFile (Join-Path $targetBundle 'msi') '*.msi'
$releaseRoot = if ($Architecture -eq 'x64') {
    Join-Path $TargetDirectory 'release'
} else {
    Join-Path $TargetDirectory 'aarch64-pc-windows-msvc\release'
}
$portableExe = Join-Path $releaseRoot 'bloody-level.exe'
if (-not (Test-Path -LiteralPath $portableExe -PathType Leaf)) {
    throw "Portable executable was not found at $portableExe. The Tauri release build must produce the standalone executable."
}
$prefix = "bloody-level-$Version-windows-$Architecture"
$exeOutput = Join-Path $OutputDirectory "$prefix.exe"
$msiOutput = Join-Path $OutputDirectory "$prefix.msi"
$zipOutput = Join-Path $OutputDirectory "$prefix.zip"

Copy-Item -LiteralPath $installerExe -Destination $exeOutput -Force
Copy-Item -LiteralPath $msi -Destination $msiOutput -Force

$portableStaging = Join-Path $OutputDirectory ".portable-$Architecture"
if (Test-Path -LiteralPath $portableStaging) {
    Remove-Item -LiteralPath $portableStaging -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $portableStaging | Out-Null
Copy-Item -LiteralPath $portableExe -Destination (Join-Path $portableStaging 'bloody-level.exe') -Force

$resources = Join-Path $releaseRoot 'resources'
if (-not (Test-Path -LiteralPath $resources -PathType Container)) {
    throw "Portable resources directory was not found at $resources. The ZIP must ship the bundled PDFium/ontology resources."
}
Copy-Item -LiteralPath $resources -Destination (Join-Path $portableStaging 'resources') -Recurse -Force
Compress-Archive -Path (Join-Path $portableStaging '*') -DestinationPath $zipOutput -CompressionLevel Optimal -Force
Remove-Item -LiteralPath $portableStaging -Recurse -Force

Write-Host "Packaged installer $prefix.exe, installer $prefix.msi, and portable application $prefix.zip"
