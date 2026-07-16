# ── Drip Drop: pull cloud DB → local + save timestamped backup ──
# Usage: set $NEON to your Neon connection string, then run:  .\sync-from-cloud.ps1

if (-not $NEON) {
    Write-Host "ERROR: `$NEON is not set. Run this first:" -ForegroundColor Red
    Write-Host '  $NEON = "postgresql://...your neon string..."' -ForegroundColor Yellow
    exit 1
}

$PG = "C:\Program Files\PostgreSQL\18\bin"
$stamp = Get-Date -Format "yyyy-MM-dd_HHmm"
$backupDir = "backups"
if (-not (Test-Path $backupDir)) { New-Item -ItemType Directory -Path $backupDir | Out-Null }
$backupFile = "$backupDir\cloud_$stamp.sql"

Write-Host "1/3  Exporting cloud database from Neon..." -ForegroundColor Cyan
& "$PG\pg_dump.exe" $NEON --no-owner --no-acl -f $backupFile
if ($LASTEXITCODE -ne 0) { Write-Host "Export failed." -ForegroundColor Red; exit 1 }
Write-Host "     Saved backup: $backupFile" -ForegroundColor Green

Write-Host "2/3  Wiping local database..." -ForegroundColor Cyan
& "$PG\psql.exe" -U postgres -d dripdrop -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" | Out-Null

Write-Host "3/3  Loading cloud data into local..." -ForegroundColor Cyan
& "$PG\psql.exe" -U postgres -d dripdrop -f $backupFile | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "Local load failed." -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "Done. Local now mirrors cloud, and a backup is saved at $backupFile" -ForegroundColor Green