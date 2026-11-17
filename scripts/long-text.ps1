$ErrorActionPreference = 'Stop'
$project = (Resolve-Path "$PSScriptRoot/..").Path
$data = "$project/.test-data/long-text"
New-Item -ItemType Directory -Path $data -Force | Out-Null
$file = "$data/library.json"
if (-not (Test-Path -LiteralPath $file)) {
    $paragraph = "薄雾越过河岸，远处的屋顶渐渐隐入晨光。林舟放下茶杯，听见檐下的雨水沿着铜铃滴落。时间并不催促任何人，只是安静地把每一个片刻放在它应在的位置。`n`n"
    $docs = foreach ($count in @(2000, 100000, 300000)) {
        $builder = [System.Text.StringBuilder]::new($count + $paragraph.Length)
        $sequence = 0
        while ($builder.Length -lt $count) {
            $sequence++
            [void]$builder.Append(('第{0:D5}段　' -f $sequence)).Append($paragraph)
        }
        @{ title = ('雾河行记 · {0:N0} 字' -f $count); date = '2026-11-18'; body = $builder.ToString(0, $count); font = 0 }
    }
    $json = @{ docs = @($docs); active = 2; last = 0 } | ConvertTo-Json -Depth 10 -Compress
    [System.IO.File]::WriteAllText($file, $json, [System.Text.UTF8Encoding]::new($false))
}
$exe = "$project/dist/Mistpaper/mistpaper.exe"
if (-not (Test-Path -LiteralPath $exe)) { throw 'Run scripts/build.ps1 first' }
$oldData = $env:MISTPAPER_DATA_DIR
$oldFps = $env:MISTPAPER_FPS
$oldLog = $env:RUST_LOG
try {
    $env:MISTPAPER_DATA_DIR = $data
    $env:MISTPAPER_FPS = '1'
    $env:RUST_LOG = 'warn,nsh::fps=info'
    New-Item -ItemType Directory -Path "$project/logs" -Force | Out-Null
    $stamp = Get-Date -Format yyyyMMdd-HHmmss
    Start-Process -FilePath $exe -WorkingDirectory (Split-Path $exe) -WindowStyle Normal -PassThru -RedirectStandardError "$project/logs/fps-$stamp.log"
} finally {
    $env:MISTPAPER_DATA_DIR = $oldData
    $env:MISTPAPER_FPS = $oldFps
    $env:RUST_LOG = $oldLog
}
