# catclustering

This crate implements hierarchical agglomerative clustering optimized for categorical features and (approximated) complete-linkage.

## Initialization Step

The algorithm does not initialize the adjacency matrix by calculating all pairwise distances, as it would be computationally inefficient.
Instead, it identifies neighbors by examining adjecent rows in the sorted dataset.
Since sorting depends on the order of the columns, it sorts the dataset multiple time, each time with a different random column order. This is roughly equivalent to identifying neighbors with random projections.
The number of iterations can be controlled with the 'init_iterations' argument.

This first step is the main reason why this crate is best suited for categorical data.

## Distance Function

To ensure the algorithm works correctly, the distance function must satisfy the following property:

```
d(x1 U x2, y1 U y2) >= d(x1, y1) 
```

for any cluster, x1, x2, y1, y2, including empty clusters.
This property is satisfied by the [original complete-linkage distance](wikipedia).
As complete-linkage is expensive, we recommend this approximation which also exhibits the desired property:

```
|clusterset1| + |clusterset2| - |intersection(clusterset1, clusterset2)|
```

`|clusterset1|` is what is left when you calculate the distance between a cluster and itself (`d(X, X) = |X| + |X| - |intersection(X, X)| = |X|`).
This is analogous to the original complete linkage, in which `d(X, X)` is
equal to X's intra-cluster distance for any cluster that contains two or more non-identical rows.
From this perspective, `|clusterset1| - 1` can be interpreted as an approximation of the intra-cluster distance.

Note that functions like `|union| - |intersection|` do not satisfy `d(x1 U x2, y1 U y2) >= d(x1, y1) ` and would not work correctly in this context.

As we will see with the examples above, the fastest way to implement set operations is by using bitmasks, so long as the number of unique categories fit into ... 

| operation | bitmasks | hashset | [roaringbit]() |   
| cardinality of intersection(x, y) | (x & y).count_ones() ---- |
| cardinality of union(x,y) | (x | y).count_ones()
| cardinality of symmetric difference | (x ^ y).count_ones()
| merge | `x |= y`


## Benchmarks

| rows       | x    |
| ---------- | ---- |
| 10,000 | 61 ms |
| 100,000 | 840 ms
|   1,000,000         |  33.8 s    |
| 10,000,000 |      | 417 s [1] 420s [2]\ = 7 min
| 50,000,000| 2295s = 38 min