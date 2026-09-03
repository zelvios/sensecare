param([Parameter(Position=0)][string]$Cmd = "up")

switch ($Cmd) {
  "up"      { docker compose up --build -d }
  "down"    { docker compose down }
  "logs"    { docker compose logs -f }
  "rebuild" { docker compose build --no-cache }
  "psql"    { docker compose exec db psql -U sensecare -d sensecare }
  "clean"   { docker compose down -v }
  default   { Write-Host "Usage: .\scripts\dev.ps1 [up|down|logs|rebuild|psql|clean]" }
}

# If powershell refuse to run script: "Set-ExecutionPolicy -Scope CurrentUser RemoteSigned"
