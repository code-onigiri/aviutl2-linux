"""Path resolution helpers for Wine prefix, AviUtl2 root, and catalog paths.

Delegates to config.py but provides convenience lookups used across modules.
"""

from . import config
from pathlib import Path
from typing import Optional


def get_aviutl_root() -> Optional[Path]:
    return config.get_aviutl_root()


def get_wine_prefix() -> Optional[Path]:
    return config.get_wine_prefix()


def get_aviutl_data_dir(aviutl_root: Path) -> Path:
    return config.get_aviutl_data_dir(aviutl_root)


def get_plugin_dir(aviutl_root: Path) -> Path:
    return config.get_plugin_dir(aviutl_root)


def get_script_dir(aviutl_root: Path) -> Path:
    return config.get_script_dir(aviutl_root)


def resolve_macro(path_template: str, aviutl_root: Path, tmp_dir: Optional[Path] = None) -> str:
    """Expand path macros in install step paths.

    Macros:
      {appDir}      — AviUtl2 data directory (NOT repo root — avoids overwriting
                       project files like README.md). On Windows this is the same
                       as the AviUtl2 root; on Linux + Proton GE it's the Wine
                       prefix's ProgramData/aviutl2/ path.
      {pluginsDir}  — Plugin directory
      {scriptsDir}  — Script directory
      {dataDir}     — AviUtl2 data directory (same as {appDir} on this platform)
      {tmp}         — Temp directory (per-install)
      {download}    — Download destination (same as {tmp}/download)
    """
    result = path_template
    data_dir = get_aviutl_data_dir(aviutl_root)
    data_str = str(data_dir.resolve())
    # {appDir} → dataDir so ZIP extraction to {appDir} doesn't overwrite
    # repo root files (README.md, etc.)
    result = result.replace("{appDir}", data_str)
    result = result.replace("{dataDir}", data_str)
    result = result.replace("{pluginsDir}", str((data_dir / "Plugin").resolve()))
    result = result.replace("{scriptsDir}", str((data_dir / "Script").resolve()))
    if tmp_dir:
        tmp_str = str(tmp_dir.resolve())
        result = result.replace("{tmp}", tmp_str)
        result = result.replace("{download}", str((tmp_dir / "download").resolve()))
    return result
