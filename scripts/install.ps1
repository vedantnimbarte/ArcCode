# wingman installer for Windows — downloads a prebuilt `wingman.exe` from the
# latest GitHub Release and installs it onto your PATH.
#
#   irm https://raw.githubusercontent.com/vedantnimbarte/Wingman/main/scripts/install.ps1 | iex
#
# Environment overrides:
#   $env:WINGMAN_INSTALL_DIR   install location  (default: %LOCALAPPDATA%\Programs\wingman)
#   $env:VERSION               pin a release tag (default: latest, e.g. v0.0.1)

$ErrorActionPreference = "Stop"

$Repo = "vedantnimbarte/Wingman"
$Bin  = "wingman"

# Only x86_64 Windows binaries are published today.
$arch = (Get-CimInstance Win32_Processor).Architecture
if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
  throw "No prebuilt Windows arm64 binary yet. Use: cargo install --git https://github.com/$Repo wingman-cli"
}
$target = "x86_64-pc-windows-msvc"
$asset  = "$Bin-$target.zip"

if ($env:VERSION) {
  $url = "https://github.com/$Repo/releases/download/$($env:VERSION)/$asset"
} else {
  $url = "https://github.com/$Repo/releases/latest/download/$asset"
}

$installDir = if ($env:WINGMAN_INSTALL_DIR) { $env:WINGMAN_INSTALL_DIR } `
              else { Join-Path $env:LOCALAPPDATA "Programs\wingman" }

$tmp = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ([System.Guid]::NewGuid()))
try {
  Write-Host "Downloading $asset ..."
  $zip = Join-Path $tmp $asset
  Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing
  Expand-Archive -Path $zip -DestinationPath $tmp -Force

  New-Item -ItemType Directory -Path $installDir -Force | Out-Null
  $exe = Get-ChildItem -Path $tmp -Recurse -Filter "$Bin.exe" | Select-Object -First 1
  if (-not $exe) { throw "$Bin.exe not found inside the archive." }
  Copy-Item $exe.FullName (Join-Path $installDir "$Bin.exe") -Force
  Write-Host "Installed $Bin to $installDir\$Bin.exe"

  # Add to the user PATH if it isn't already there.
  $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
  if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    Write-Host "Added $installDir to your user PATH — restart your terminal to pick it up."
  }
  Write-Host "Run: $Bin --help"
} finally {
  Remove-Item -Recurse -Force $tmp
}
