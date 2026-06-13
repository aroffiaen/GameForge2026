#!/usr/bin/env bash
# Lance GameForge2026 sous WSLg avec accélération GPU (NVIDIA via D3D12).
#
# Contexte WSL2/WSLg :
#  - wgpu (moteur de Bevy) utilise Vulkan par défaut, mais Ubuntu ne fournit
#    AUCUN driver Vulkan matériel sous WSL (seulement lavapipe = logiciel,
#    très lent → fenêtre qui semble figée).
#  - Le seul chemin GPU dispo est OpenGL via le driver Mesa "d3d12"
#    (d3d12_dri.so), qui passe par /dev/dxg → ton GPU. Et ce backend GL ne
#    s'initialise proprement que sur X11 (sur Wayland il se fige).
#
# Pré-requis (UNE fois) : la lib d'entrée X11 de xkbcommon.
#    sudo apt install -y libxkbcommon-x11-0
set -e
cd "$(dirname "$0")"

# Forcer X11 (Xwayland) : on n'expose PAS le socket Wayland à winit.
unset WAYLAND_DISPLAY
export DISPLAY="${DISPLAY:-:0}"

# Backend GL de wgpu + driver Mesa d3d12 (accélération GPU).
export WGPU_BACKEND=gl
export LIBGL_ALWAYS_SOFTWARE=0
export GALLIUM_DRIVER=d3d12
export MESA_LOADER_DRIVER_OVERRIDE=d3d12

exec cargo run --release "$@"
