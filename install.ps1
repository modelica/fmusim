# ==========================================
# CONFIGURATION
# ==========================================
$repoUser   = "modelica"
$repoName   = "fmusim"
$appName    = "fmusim" # The name of the folder where it will live
# The name of your zip file asset as it appears on GitHub:
$zipName    = "fmusim-x86_64-windows.zip"

# ==========================================
# INSTALLATION LOGIC
# ==========================================
$ErrorActionPreference = "Stop"

# 1. Define paths (Installs to C:\Users\<User>\AppData\Local\<AppName>)
$installDir = Join-Path $env:LOCALAPPDATA $appName
$tempZip    = Join-Path $env:TEMP "$zipName"
$downloadUrl = "https://github.com/$repoUser/$repoName/releases/latest/download/$zipName"

Write-Host "Installing $appName..." -ForegroundColor Cyan

# 2. Clean old installation if it exists
if (Test-Path $installDir) {
    Write-Host "Removing older version..." -ForegroundColor Gray
    Remove-Item -Recurse -Force $installDir
}

# 3. Download the zip artifact
Write-Host "Downloading latest release..." -ForegroundColor Gray
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip -UseBasicParsing

# 4. Extract the ZIP file
Write-Host "Extracting files to $installDir..." -ForegroundColor Gray
Expand-Archive -Path $tempZip -DestinationPath $installDir -Force

# 5. Add to User PATH permanently (if not already there)
$userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($userPath -notlike "*$installDir*") {
    Write-Host "Adding $appName to User PATH..." -ForegroundColor Gray
    $newPath = "$userPath;$installDir"
    [Environment]::SetEnvironmentVariable("Path", $newPath, [EnvironmentVariableTarget]::User)
    
    # Update current session PATH so they can use it immediately
    $env:Path += ";$installDir"
}

# 6. Cleanup the downloaded zip
Remove-Item -Force $tempZip

Write-Host "`nSuccessfully installed $appName!" -ForegroundColor Green
Write-Host "You may need to restart your terminal/IDE for the PATH changes to take effect." -ForegroundColor Yellow

# ==========================================
# 7. AUTOMATIC POWERSHELL AUTO-COMPLETE SETUP
# ==========================================
Write-Host "Setting up PowerShell auto-completion..." -ForegroundColor Gray

# Define where the completion script will live
$completionFile = Join-Path $installDir "_$appName.ps1"

try {
    # Generate the completion file by running the newly installed app
    # (Adjust the arguments below if your completion flag is named differently)
    & (Join-Path $installDir "$appName.exe") completion power-shell | Out-File -FilePath $completionFile -Encoding utf8

    # Check if the user has a PowerShell profile, create it if it doesn't exist
    if (!(Test-Path $PROFILE)) {
        New-Item -Type File -Path $PROFILE -Force | Out-Null
    }

    # Line to add to the profile
    $profileLine = ". `"$completionFile`""

    # Read the current profile content to avoid duplicating the line
    $profileContent = Get-Content $PROFILE -ErrorAction SilentlyContinue
    if ($profileContent -notcontains $profileLine) {
        Add-Content -Path $PROFILE -Value "`n$profileLine"
        Write-Host "Auto-completion added to your PowerShell `$PROFILE." -ForegroundColor Gray
    }

    # Dot-source it immediately so it works in the current active session
    . $completionFile
    Write-Host "Auto-completion activated for this session!" -ForegroundColor Green
}
catch {
    Write-Warning "Could not configure auto-completion: $_"
}
