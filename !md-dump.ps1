param (
    [string]$ProjectRoot = '.',
    [string]$OutputFile = 'project_dump.md'
)

$IncludeExtensions = @('.rs', '.toml', '.md', '.wgsl', '.ron', '.json', '.yaml', '.yml')
$ExcludeDirs = @('target', 'target_tmp', '.vers', '.git', '.idea', '.vscode', 'node_modules')

function Get-LanguageTag([string]$ext) {
    switch ($ext) {
        '.rs'   { 'rust' }
        '.toml' { 'toml' }
        # '.md'   { 'markdown' }
        '.wgsl' { 'wgsl' }
        '.ron'  { 'ron' }
        '.json' { 'json' }
        # '.yaml' { 'yaml' }
        # '.yml'  { 'yaml' }
        default { '' }
    }
}

# Создаём/очищаем output в UTF-8
Set-Content -Path $OutputFile -Value '' -Encoding utf8

$projectName = Split-Path -Leaf (Resolve-Path $ProjectRoot)
Add-Content -Path $OutputFile -Value ("# Project Snapshot: {0}`n" -f $projectName) -Encoding utf8

$rootPath = (Resolve-Path $ProjectRoot).Path

Get-ChildItem -Path $ProjectRoot -Recurse -File | ForEach-Object {

    # пропуск исключённых директорий
    foreach ($dir in $ExcludeDirs) {
        if ($_.FullName -like ("*\{0}\*" -f $dir)) { return }
    }

    # пропуск по расширению
    if ($IncludeExtensions -notcontains $_.Extension) { return }

    $relativePath = $_.FullName.Substring($rootPath.Length).TrimStart('\','/')
    $lang = Get-LanguageTag $_.Extension

    Add-Content -Path $OutputFile -Value ("## {0}" -f $relativePath) -Encoding utf8
    Add-Content -Path $OutputFile -Value ("```{0}" -f $lang) -Encoding utf8

    try {
        Get-Content -Path $_.FullName -Raw -Encoding utf8 -ErrorAction Stop |
            Add-Content -Path $OutputFile -Encoding utf8
    } catch {
        Add-Content -Path $OutputFile -Value '# ERROR: failed to read file as UTF-8' -Encoding utf8
    }

    Add-Content -Path $OutputFile -Value '```' -Encoding utf8
    Add-Content -Path $OutputFile -Value '' -Encoding utf8
}

Write-Host ("Done: {0}" -f $OutputFile)
