# `a11y-tree-data`

A repository for sharing accessibility trees for analysis of their shape, size, and properties.

## Licenses

- Code: MIT or Apache-2.0
- Data: CC-BY-SA-4.0

## Scripts

You can use the `code/scripts/get_stats.sh` on the JSON output of the `linux-atspi-tree` tool.

That script retrieves the following information about the tree:

- Number of leaf nodes
- Number of total nodes
- Unique roles (and # of nodes for each)
- Max depth of tree
- Max children of single node

