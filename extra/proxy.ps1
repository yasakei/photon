[CmdletBinding()]
param(
    [string]$version,
    [string]$directory
)

$proxy = (Join-Path $directory 'photon.exe')

$PhotonProcesses = (Get-Process -Name 'photon' -EA SilentlyContinue).Count
if ($PhotonProcesses -ne 0) {
    Write-Host 'Proxy currently in use. Aborting installation'
    exit
}

if (Test-Path $proxy) {
    Write-Host 'Proxy already installed'
    exit
}

switch ($env:PROCESSOR_ARCHITECTURE) {
    # Only x86_64 is supported currently
    'AMD64' {
        $arch = 'x86_64'
    }
}

$url = "https://github.com/lapce/lapce/releases/download/${version}/photon-proxy-windows-${arch}.gz"
$gzip = Join-Path "${env:TMP}" "photon-proxy-windows-${arch}.gz"

$webclient = [System.Net.WebClient]::new()
$webclient.DownloadFile($url, $gzip)
$webclient.Dispose()

[System.IO.Directory]::CreateDirectory($directory)

$archive = [System.IO.File]::Open($gzip, [System.IO.FileMode]::Open)
$proxy_file = [System.IO.File]::Create($proxy)
$compressor = [System.IO.Compression.GZipStream]::new($archive, [System.IO.Compression.CompressionMode]::Decompress)
$compressor.CopyTo($proxy_file)
Start-Sleep -Seconds 3
$compressor.close()
$proxy_file.close()
$archive.close()

[System.IO.File]::Delete($gzip)

Write-Host 'photon-proxy installed'