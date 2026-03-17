#!/usr/bin/env python3
"""TypeScript Node generator entrypoint.

Implementation details live in helper modules under `codegen/`.
"""

import sys
import subprocess
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from ts_node_core import (
    gen_diagnostic,
    gen_entry,
    gen_errors,
    gen_input,
    gen_interface,
    gen_math,
    gen_napi_rust,
    gen_node_wrapper,
    gen_public_entrypoints,
)
from ts_node_wrapper import gen_network_shared_wrapper


def main() -> None:
    print("Generating TypeScript Node SDK...")
    gen_interface()
    gen_input()
    gen_math()
    gen_node_wrapper()
    gen_network_shared_wrapper()
    gen_entry()
    gen_public_entrypoints()
    gen_napi_rust()
    gen_errors()
    gen_diagnostic()
    subprocess.run(
        ["cargo", "fmt", "-p", "goud-engine-node"],
        check=True,
        cwd=Path(__file__).resolve().parent.parent,
    )
    print("TypeScript Node SDK generation complete.")


if __name__ == "__main__":
    main()
