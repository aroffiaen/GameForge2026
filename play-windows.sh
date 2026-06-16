#!/usr/bin/env bash
# Compile et lance GameForge2026 NATIVEMENT sous Windows, depuis WSL.
#
# Pourquoi : sous WSL/WSLg, Bevy ne trouve pas de compositeur/GPU exploitable
# (`cargo run` plante avec « WaylandError(NoCompositor) » ou tombe en rendu
# logiciel inutilisable). On compile donc avec la toolchain Windows et on lance
# l'exe dans une vraie fenêtre Windows (GPU NVIDIA, Vulkan).
#
# Usage : depuis ~/GameForge2026, lance simplement :  ./play-windows.sh
set -e

SRC="$HOME/GameForge2026"
DST="/mnt/c/GameForge2026"

echo ">> [1/4] Sync sources + assets vers C:\\GameForge2026 ..."
mkdir -p "$DST/src"
rm -rf "$DST/src"/*
cp -r "$SRC/src/." "$DST/src/"
cp "$SRC/Cargo.toml" "$DST/"
cp "$SRC/Cargo.lock" "$DST/" 2>/dev/null || true
rm -rf "$DST/assets"
cp -r "$SRC/assets" "$DST/"

echo ">> [2/4] Build release (toolchain Windows)..."
powershell.exe -NoProfile -Command 'cd C:\GameForge2026; & "$env:USERPROFILE\.cargo\bin\cargo.exe" build --release; exit $LASTEXITCODE'

echo ">> [3/4] Copie des assets a cote de l'exe (resolution par defaut de Bevy)..."
rm -rf "$DST/target/release/assets"
cp -r "$DST/assets" "$DST/target/release/"

echo ">> [4/4] Lancement du jeu (fenetre Windows)..."
powershell.exe -NoProfile -Command 'Start-Process "C:\GameForge2026\target\release\GameForge2026.exe"'

echo ">> OK — le jeu s'ouvre dans une fenetre Windows."
