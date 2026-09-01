@echo off
REM Un comando: levanta inZKtagram con Docker (cmd.exe / doble clic).
cd /d "%~dp0"
where docker >nul 2>&1
if errorlevel 1 (
  echo No se encontro Docker. Instala Docker Desktop y reintenta.
  exit /b 1
)
docker info >nul 2>&1
if errorlevel 1 (
  echo Docker esta instalado pero el motor no responde. Abri Docker Desktop y espera a que arranque.
  exit /b 1
)
echo inZKtagram - http://localhost:8080
echo La primera vez baja la toolchain RISC Zero y puede tardar varios minutos.
echo Ctrl+C detiene el contenedor.
echo.
docker compose version >nul 2>&1
if errorlevel 1 (
  docker-compose up --build
) else (
  docker compose up --build
)
