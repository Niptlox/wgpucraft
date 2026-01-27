# $ProjectName = "wgpucraft"
$ProjectRoot = Get-Location
$ProjectName = Split-Path $ProjectRoot -Leaf   # имя текущей папки


$VersionDir  = ".vers"
$SevenZip    = "C:\Program Files\7-Zip\7z.exe"

$ExcludeFolders = @(
    ".vers",
    "target",
    "target_tmp",
    "node_modules",
    ".idea",
    ".vscode",
    "dist"
)

$ProjectRoot = Get-Location

if (!(Test-Path $VersionDir)) {
    New-Item -ItemType Directory -Path $VersionDir | Out-Null
}

# --- Надёжный поиск всех версий ---
$lastVersion = 0
Get-ChildItem $VersionDir -File | ForEach-Object {
    if ($_.Name -match "^$ProjectName \((\d+)\)\.7z$") {
        $v = [int]$matches[1]
        if ($v -gt $lastVersion) { $lastVersion = $v }
    }
}

$newVersion = $lastVersion + 1

# --- Гарантия что файл не существует ---
do {
    $archiveName = "$ProjectName ($newVersion).7z"
    $archivePath = Join-Path $VersionDir $archiveName
    $newVersion++
} while (Test-Path $archivePath)

Write-Host "Последняя версия: $lastVersion"
Write-Host "Создаю архив: $archiveName"

# --- Исключения ---
$excludeArgs = @()
foreach ($folder in $ExcludeFolders) {
    $excludeArgs += "-xr!$folder"
}

# --- ВАЖНО: ключ 'a' (add) без обновления ---
& $SevenZip a -t7z "`"$archivePath`"" "`"$ProjectRoot\*`"" @excludeArgs

Write-Host "? Архив создан: $archivePath"
