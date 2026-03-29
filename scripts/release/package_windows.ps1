param(
  [Parameter(Mandatory = $true)]
  [string]$AppName,
  [Parameter(Mandatory = $true)]
  [string]$Target,
  [Parameter(Mandatory = $true)]
  [string]$Version
)

$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$DistDir = Join-Path $RootDir "dist"
$StageDir = Join-Path $DistDir "$AppName-$Version-$Target"
$TargetRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $RootDir "target" }
$BinaryPath = Join-Path $TargetRoot "$Target\release\$AppName.exe"
if (-not (Test-Path $BinaryPath)) {
  $BinaryPath = Join-Path $TargetRoot "release\$AppName.exe"
}
$ArchiveName = "$AppName-$Version-$Target.zip"
$ArchivePath = Join-Path $DistDir $ArchiveName
$ChecksumPath = Join-Path $DistDir "$ArchiveName.sha256"

if (Test-Path $StageDir) {
  Remove-Item -Path $StageDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $StageDir | Out-Null

Copy-Item $BinaryPath (Join-Path $StageDir "$AppName.exe")
Copy-Item (Join-Path $RootDir "README.md") (Join-Path $StageDir "README.md")
Copy-Item (Join-Path $RootDir "migrations") (Join-Path $StageDir "migrations") -Recurse

if (Test-Path $ArchivePath) {
  Remove-Item -Path $ArchivePath -Force
}
Compress-Archive -Path "$StageDir\*" -DestinationPath $ArchivePath

$hash = (Get-FileHash -Algorithm SHA256 -Path $ArchivePath).Hash.ToLower()
"$hash  $ArchiveName" | Set-Content -Path $ChecksumPath -NoNewline

Write-Host "packaged $ArchivePath"
