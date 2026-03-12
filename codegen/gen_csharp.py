#!/usr/bin/env python3
"""Generates the complete C# SDK from the universal schema."""

from csharp_codegen import generate_all


if __name__ == "__main__":
    print("Generating C# SDK...")
    generate_all()
    print("C# SDK generation complete.")
