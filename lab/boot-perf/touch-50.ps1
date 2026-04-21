# Criterion 4 helper: touch 50 random .md files in the trial Universe so the
# stat-sweep reconcile path has external changes to detect.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File touch-50.ps1
#
# Assumes the trial Universe lives at ../trial-universe/output/Constellation Discovery/.
# Adjust $universe if yours is elsewhere.

param(
    [string]$universe = "$PSScriptRoot/../trial-universe/output/Constellation Discovery",
    [int]$count = 50
)

if (-not (Test-Path $universe)) {
    Write-Error "Universe not found: $universe"
    exit 1
}

$all = Get-ChildItem -Path $universe -Filter *.md -Recurse -File
if ($all.Count -lt $count) {
    Write-Error "Universe only has $($all.Count) .md files; need at least $count"
    exit 1
}

$picked = $all | Get-Random -Count $count
$now = Get-Date

foreach ($f in $picked) {
    $f.LastWriteTime = $now
}

Write-Host "Touched $count files in $universe"
$picked | Select-Object -First 5 | ForEach-Object { Write-Host "  $($_.FullName)" }
Write-Host "  ..."
