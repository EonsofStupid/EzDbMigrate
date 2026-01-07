# setup_env.ps1
# Permanently configure User Path for Rust and fnm on Windows

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$NewPath = $UserPath

# Paths to ensure exist
$PathsToAdd = @(
    "$env:LOCALAPPDATA\Programs\fnm",
    "$env:USERPROFILE\.cargo\bin",
    "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin"
)

$MadeChanges = $false

foreach ($Path in $PathsToAdd) {
    if ($NewPath -notlike "*$Path*") {
        Write-Host "Adding to Path: $Path"
        $NewPath += ";$Path"
        $MadeChanges = $true
    } else {
        Write-Host "Found existing: $Path"
    }
}

if ($MadeChanges) {
    Write-Host "Updating User Registry Path..."
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    
    # Update current session too so we don't have to restart immediately
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
    Write-Host "SUCCESS: Environment variables updated permanently."
} else {
    Write-Host "All paths already configured."
}

# Verify
Write-Host "`nVerifying tools:"
Write-Host "fnm: $(Get-Command fnm -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)"
Write-Host "cargo: $(Get-Command cargo -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)"
Write-Host "rustc: $(Get-Command rustc -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)"
