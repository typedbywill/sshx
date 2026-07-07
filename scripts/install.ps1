$ErrorActionPreference = 'Stop'

# Repository configuration
$repo = "typedbywill/sshx"

# Determine Architecture
$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -eq "AMD64") {
    $binaryName = "sshx-windows-amd64.exe"
} else {
    Write-Error "Unsupported architecture: $arch"
    exit 1
}

# Fetch latest release version from GitHub API
Write-Host "Checking latest release of sshx..."
$releasesUrl = "https://api.github.com/repos/$repo/releases/latest"
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $response = Invoke-RestMethod -Uri $releasesUrl -UseBasicParsing
    $latestRelease = $response.tag_name
} catch {
    $latestRelease = "v0.1.2"
    Write-Warning "Could not fetch latest release, using default: $latestRelease"
}

$downloadUrl = "https://github.com/$repo/releases/download/$latestRelease/$binaryName"
$installDir = Join-Path $HOME ".sshx\bin"
$installPath = Join-Path $installDir "sshx.exe"

# Create installation directory if it doesn't exist
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

Write-Host "Downloading $binaryName from $downloadUrl..."
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $installPath -UseBasicParsing
} catch {
    Write-Error "Failed to download $binaryName from $downloadUrl."
    exit 1
}

Write-Host "Successfully installed sshx to $installPath"

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host "Adding $installDir to User PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    $env:Path = "$env:Path;$installDir"
    Write-Host "Please restart your terminal to apply the environment changes."
}

Write-Host "Installation complete! Try running 'sshx --help'"
