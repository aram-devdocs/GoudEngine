#!/usr/bin/env bash
#
# graph.sh — Render the crate module dependency graph.
# Requires `cargo modules` and Graphviz (neato, dot) on PATH.
set -euo pipefail

cargo modules dependencies --no-externs --no-fns --no-sysroot --no-traits --no-types --no-uses >docs/diagrams/mods.dot
neato -Tpdf docs/diagrams/mods.dot -o docs/diagrams/module_graph.pdf
dot -Tpng docs/diagrams/mods.dot -o docs/diagrams/module_graph.png