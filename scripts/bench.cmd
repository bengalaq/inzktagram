@echo off
cd /d "%~dp0.."
if not exist benchmarks mkdir benchmarks
echo inZKtagram zkbench - writes benchmarks\results.csv
echo This takes several minutes per row. Ctrl+C to stop; the CSV is flushed after each row.
docker compose --profile bench run --rm zkbench
exit /b %ERRORLEVEL%
