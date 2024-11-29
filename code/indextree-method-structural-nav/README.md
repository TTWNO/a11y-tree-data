# Methods for Structural Navigation Traversal

Structural navigation can sometimes be slow, especially on large trees.
While `indextree` helps _a lot_ in referencing nodes, this may not be possible in the real world (since AT-SPI uses `ObjectRef` to reference other items).\

This crate allows us to test tree traversal methods in a "pure" environment, without issues related to actual screen reader usage.

## Running

```bash
$ cargo run -- ../../data/SOME_FILE_HERE.json
```

## Methods

We use various methods to traverse the tree.
Some of them require some pre-calcualtions, of which, we make sure to note at the start of the run.
Methods of storing the `RoleSet` should also be of interest, and we will detail further all methods and all representations of the `RoleSet`.

For now, the only implementation of `RoleSet` is a binflag-like value (136 bits wide) for each `Node`.
Future considerations include:

- A `Vec<Role>`
- A `SmallVec<Role, N>` with various different stack size inlinings.
- A `Vec<(Role, usize)>` that counds the number of each role in the subtree for each node.
- A `SmallVec<(Role, usize), N>`taht counts the number of each role with various different stack size inlinings.

### Method 1

Using a traversal, traverse all nodes until a node with the given role is found.

### Method 2

Uses a stack, and the `roleset` attribute on each node to ignore subtrees which do not have any of the right roles within the subtree.

This method requires a pre-calculation of `RoleSet`s for each node, the timing is checked at the start of the file.

### Method 3

Uses extensions traits on `indextree` to change the standard traversal process; similar to method 2, this removes the overhead of checking subtrees which do not contain the given role.

This method also requires a pre-calculation of `RoleSet`s for each node, the timing is checked at the start of the file.

This method does not allocate, nor does it store an internal stack.

