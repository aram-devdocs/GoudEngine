#!/usr/bin/env python3
"""Generate the Kotlin SDK wrapper classes from the schema."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from kotlin_codegen import generate_all

if __name__ == "__main__":
    generate_all()
