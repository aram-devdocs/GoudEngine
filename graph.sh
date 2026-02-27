cargo modules dependencies --no-externs --no-fns --no-sysroot --no-traits --no-types --no-uses >docs/diagrams/mods.dot
neato -Tpdf docs/diagrams/mods.dot -o docs/diagrams/module_graph.pdf
dot -Tpng docs/diagrams/mods.dot -o docs/diagrams/module_graph.png