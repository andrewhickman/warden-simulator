# Renumbers "Tile N" groups (N >= 14 shifted down by one), moves each tile into
# "Row floor(N / 16) + 1" (creating rows as needed), and sorts rows and tiles.
# Not idempotent: running it twice shifts the tiles twice.
param([string]$Path = (Join-Path $PSScriptRoot 'walls.svg'))

$ErrorActionPreference = 'Stop'

$text = [IO.File]::ReadAllText($Path)
$nl = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }
$lines = $text -split '\r?\n'

# Returns the index of the line closing the start tag opened at $start.
function Get-TagEnd([int]$start) {
    $i = $start
    while (-not $lines[$i].TrimEnd().EndsWith('>')) { $i++ }
    $i
}

function Get-Label([int]$start, [int]$end) {
    foreach ($l in $lines[$start..$end]) {
        if ($l -match 'inkscape:label="([^"]*)"') { return $Matches[1] }
    }
}

$rows = @{}
$tiles = @{}
$rowTemplate = $null
$firstRow = -1
$lastRow = -1
$i = 0
while ($i -lt $lines.Count) {
    if ($lines[$i] -ne '  <g') { $i++; continue }
    $headerEnd = Get-TagEnd $i
    if ((Get-Label $i $headerEnd) -notmatch '^Row (\d+)$') { $i++; continue }
    $rowNum = [int]$Matches[1]

    if ($firstRow -lt 0) { $firstRow = $i }
    elseif ($i -ne $lastRow + 1) { throw "Unexpected content between rows at line $($lastRow + 2)" }

    $header = $lines[$i..$headerEnd]
    if (-not $rowTemplate) { $rowTemplate = $header }
    $rows[$rowNum] = $header

    $j = $headerEnd + 1
    while ($lines[$j] -ne '  </g>') {
        if ($lines[$j] -ne '    <g') { throw "Unexpected non-tile content in Row $rowNum at line $($j + 1)" }
        $tileStart = $j
        $tileHeaderEnd = Get-TagEnd $j
        if ((Get-Label $tileStart $tileHeaderEnd) -notmatch '^Tile (\d+)$') { throw "Unexpected group in Row $rowNum at line $($j + 1)" }
        $n = [int]$Matches[1]
        while ($lines[$j] -ne '    </g>') { $j++ }
        $tile = $lines[$tileStart..$j]
        $j++

        if ($n -ge 14) { $n-- }
        for ($k = 0; $k -le $tileHeaderEnd - $tileStart; $k++) {
            if ($tile[$k] -match 'inkscape:label="Tile \d+"') {
                $tile[$k] = $tile[$k] -replace 'inkscape:label="Tile \d+"', "inkscape:label=`"Tile $n`""
                break
            }
        }

        if ($tiles.ContainsKey($n)) { throw "Duplicate Tile $n after renumbering" }
        $tiles[$n] = $tile
    }
    $lastRow = $j
    $i = $j + 1
}
if ($firstRow -lt 0) { throw 'No rows found' }

$rowTiles = @{}
foreach ($n in $tiles.Keys) {
    $m = [int][math]::Floor($n / 16) + 1
    if (-not $rowTiles[$m]) { $rowTiles[$m] = [Collections.Generic.List[int]]::new() }
    $rowTiles[$m].Add($n)
}

foreach ($m in $rowTiles.Keys) {
    if ($rows.ContainsKey($m)) { continue }
    $id = "layer$m"
    $suffix = 0
    while ($text.Contains("id=`"$id`"")) { $id = "layer$m-$(++$suffix)" }
    $rows[$m] = $rowTemplate | ForEach-Object {
        $_ -replace 'inkscape:label="Row \d+"', "inkscape:label=`"Row $m`"" -replace '\bid="[^"]*"', "id=`"$id`""
    }
}

$out = [Collections.Generic.List[string]]::new()
$out.AddRange([string[]]$lines[0..($firstRow - 1)])
foreach ($m in $rows.Keys | Sort-Object) {
    $out.AddRange([string[]]$rows[$m])
    if ($rowTiles[$m]) {
        foreach ($n in $rowTiles[$m] | Sort-Object) { $out.AddRange([string[]]$tiles[$n]) }
    }
    $out.Add('  </g>')
}
$out.AddRange([string[]]$lines[($lastRow + 1)..($lines.Count - 1)])

[IO.File]::WriteAllText($Path, ($out -join $nl), [Text.UTF8Encoding]::new($false))
