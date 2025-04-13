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

## Code & Benchmarks

- You can run benchmarks on a variety of single-threaded, parallel, and data-optimized tree traversal and counting algorithms.
- Go to `code/indextree-method-structural-nav/`
- Run `cargo bench`
- There, you will also find a report in the benchmarks entitled `REPORT_DATA.md`
    - This also discusses future plans and expansion of the project.

## Create a Tree

- To create a new tree from your existing system on Linux, go to `code/linux-atspi-tree/` and simply run `cargo run`
- This will attach to the accessibility bus on your system and create a tree from it.
- Once complete, the tree will be printed to `stdout` in JSON format.
- If you have a web browsewr or email client open, this can take some time as round-trip IPC calls must be made for each node in the tree.
