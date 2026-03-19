#!/usr/bin/env python3
"""Generate Lua bindings for GoudEngine.

Usage:
    cd codegen && python3 gen_lua.py
"""

import sys
from pathlib import Path

# Ensure codegen dir is on the path
sys.path.insert(0, str(Path(__file__).parent))

from lua_codegen.runner import run

if __name__ == "__main__":
    run()
