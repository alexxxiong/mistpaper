param([string]$Cargo = 'cargo')
$ErrorActionPreference = 'Stop'
$project = (Resolve-Path "$PSScriptRoot/..").Path
Push-Location $project
try {
    $env:RUST_MIN_STACK = '268435456'
    if (-not $env:CARGO_BUILD_JOBS) { $env:CARGO_BUILD_JOBS = '2' }
    & $Cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Rust release build failed' }
    $output = "$project/dist/Mistpaper"
    New-Item -ItemType Directory -Path $output -Force | Out-Null
    Copy-Item -LiteralPath "$project/target/release/mistpaper.exe" -Destination $output -Force
    Copy-Item -LiteralPath "$project/assets" -Destination $output -Recurse -Force
    Copy-Item -LiteralPath "$project/LICENSE" -Destination $output -Force
    Copy-Item -LiteralPath "$project/THIRD_PARTY_NOTICES.md" -Destination $output -Force
    Write-Host "Client: $output/mistpaper.exe"
} finally { Pop-Location }
