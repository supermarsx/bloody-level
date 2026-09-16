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

$exe = Get-SingleBundleFile (Join-Path $targetBundle 'nsis') '*.exe'
$msi = Get-SingleBundleFile (Join-Path $targetBundle 'msi') '*.msi'
$prefix = "bloody-level-$Version-windows-$Architecture"
$exeOutput = Join-Path $OutputDirectory "$prefix.exe"
$msiOutput = Join-Path $OutputDirectory "$prefix.msi"
$zipOutput = Join-Path $OutputDirectory "$prefix.zip"

Copy-Item -LiteralPath $exe -Destination $exeOutput -Force
Copy-Item -LiteralPath $msi -Destination $msiOutput -Force
Compress-Archive -Path $exeOutput, $msiOutput -DestinationPath $zipOutput -CompressionLevel Optimal -Force

Write-Host "Packaged $prefix.exe, $prefix.msi, and $prefix.zip"
