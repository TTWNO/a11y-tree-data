# Report on Findings of Parallel Algorithms

## Goals

This project was created to assess different methods of traversing and storing tree data for use in assistive technologies like screen readers.
Since screen readers often have to jump to the next in-order node with a given role, we wanted to explore methods to make this faster, given a directed asynclic graph (tree) with accessibility inforamtion (role) and its children.
While this _does_ have general applicability to other role-based traversals, this is the applied reasoning for the project.

## Experiment

Here is a list of methods that were benchmarked to understand the performance implications of each implementaion.
We ran on real data from a large web page—the one-page HTML specification—and _much_ larger synthetic data to understand the performance characteristics of various approaches with increasingly large sizes.
Additionally, there are two implementations for all functions, one which only stores a role-based bitset, and another that stores both a bitset, and _the number of nodes with that role in its descendants_.
These can be thought of as `TreeNode { bitset }` and `TreeCountNode { bitset, Vec<(Role, uszie)> }`

Here, we will list each function, along with links to its documentation, performance results with both real and synthetic data (x) and performance results across the two styles of storing bitset-propogation (y).
Additionally, notes will be given when a function performs particularily poorly or well, and additional details as needed.

One interesting result to note before reading the details: the performance uplift is relatively independent of the size; the non-synthetic data appears to be large enough to get proportional benefits.

- [`iter_leafs`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.iter_leafs)
    - Iterate through all leaf nodes.
    - `O(n)`
- [`par_iter_leafs`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.par_iter_leafs)
    - Compared to `iter_leafs`, about an 80% improvement to performance and 50% decrease in standard deviation.
    - `O(n/p)` where `p` is number of processors.
- [`how_many(role)`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.how_many)
    - Counts the number of nodes with a given role.
    - `O(n)`
- [`par_how_many(role)`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.par_how_many)
    - Compared to `how_many`, increase in performance of 95%, reduction in standard deviation by 80%.
    - `O(n/p)` where `p` is number of processors.
- [`how_many_roleset(role)`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.how_many_roleset)
    - For the variety of tree that stores both the role and the count for each of them in all subtrees, this was extemely fast; constant in about 7 nanoseconds. `O(1)`
    - For the tree that doesn't store this extra data: there is still a speedup of 2 orders of magnitude (99%). Still `O(n)` worst case.
- [`par_how_many_roleset(role)`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.par_how_many_roleset)
    - Compared to `how_many_roleset` (non-counting nodes only), about a 60% performance increase. `O(n/p)`
- [`max_depth`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.max_depth)
    - Find the depth of the deepest node. `O(n)`
- [`par_max_depth`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.par_max_depth)
    - Compared to `max_depth`, 75% performance increase. `O(n/p)`
- [`unique_roles`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.unique_roles)
    - Get a list of all unique roles in the tree. `O(n)`
- [`par_unique_roles`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.par_unique_roles)
    - Compared to `unique_roles`, 90% performance increase. `O(n/p)`
- [`unique_roles_roleset`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.unique_roles_roleset)
    - For both trees this is instant: `O(1)` in about 10 nanoseconds.
- [`find_first`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.find_first)
    - `O(n)`
    - Synthetic data had _much_ shorter processing times.
    - However, this is just a coincidence; a larger variety of roles are closer to the root, and the benchmarks only meassure from the root node.
- [`par_find_first`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.par_find_first)
    - Compared to `find_first`, there was a fairly consistent 50% performance improvement, and 50% decrease in standard deviation. `O(n)`
- [`find_first_roleset`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.find_first_roleset)
    - Compared to `find_first`, pruning subtress without the searched-for role increases performance by about 2 orders of magnitude (99%; `O(n)`)
    - And shrinks the standard deviation by one order of magnitude (90%)
- [`par_find_first_roleset`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.par_find_first_roleset)
    - Compared to `find_first_roleset`, consistent performance improvements of about 50%.
    - And a standard deviation decrease of about 70%.
    - `O(n/p)`
- [`find_first_stack`](./target/doc/indextree_method_structural_nav/trait.TreeTraversal.html#tymethod.find_first_stack)
    - Since this uses a stack-based push/pop algorithm, it's inherently sequential. It uses the `roleset` advantage.
    - Compared to `find_first_roleset`, it increases performance by about 15%, and improves standard deviation by about the same amount.
    - `O(n)`

You can also access [detailed performance reports here](./target/criterion/report/index.html).

Interestingly, despite almost all algorithms performing similarily in asymptotic time, there are very signficant gains to be made by both parallelizing and structuring the data to compute things faster at runtime.
This shows that while asymtotic time is real and a useful lense to use on _massive_ datasets, crunching through hundreds of mega (or giga-) bytes is completely reasonable with some clever, non-asymtotic speedups.
This conclusion is pretty similar to Quicksort's; the asymptotic time is still `O(n^2)`, but in practice it is close to `O(n log n)` for nearly all real data.

## Further Work

- Screen readers need to write to their tree _much_ more often than they need to read from it. There should be additional benchmarks for:
    - Removing items (and propogating the role changes through counting and non-counting trees)
    - Adding items (and propogating the role changes though counting and non-counting trees)
- And equivelant benchmarks for parallel/concurrent access and modification; try using two strategies for keeping the tree in tact
    - [`RwLock<Tree>`](https://doc.rust-lang.org/std/sync/struct.RwLock.html)
    - [`ReadHandle<Tree>`](https://docs.rs/evmap/latest/evmap/struct.ReadHandle.html) and [`WriteHandle<Tree>`](https://docs.rs/evmap/latest/evmap/struct.WriteHandle.html) from the `evmap` project.
        - This allows for `ev`entual consistency while allowing _multiple reads and multiple write to proceeed in parallel_, with explicit synchronization.
- Test against other traditional "caching" mechanisms.
- Integrate into the [Odilia screen reader](https://odilia.app/)

