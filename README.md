# catclustering

This crate implements hierarchical agglomerative clustering optimized for categorical features and (approximated) complete linkage.

## Initialization Step

To avoid the computational inefficiency of calculating all pairwise distances for the adjacency matrix, the algorithm identifies neighbors by examining adjacent rows in the sorted dataset. 

Since the order of the columns affects the outcome, the dataset is sorted multiple times, each time with a different random column order.

Since sorting depends on the order of the columns, it sorts the dataset multiple time, each time with a different random column order.
This approach is roughly equivalent to identifying neighbors through random projections onto a one-dimensional line.
The number of iterations can be controlled with the 'init_iterations' parameter.

This initial step makes this crate particularly well-suited for categorical data.

## Distance Function

To ensure the algorithm works correctly, the distance function must satisfy the following property:

```
d(x1 U x2, y1 U y2) >= d(x1, y1) 
```

for any cluster, *x1*, *x2*, *y1, *y2*, including empty clusters.

This property is satisfied by the [original complete-linkage distance](https://en.wikipedia.org/wiki/Complete-linkage_clustering).

However, because complete linkage is computationally expensive, we recommend the following approximation, which also exhibits the desired property:

```
|clusterset1| + |clusterset2| - |intersection(clusterset1, clusterset2)|
```

`|clusterset1|` is what remains when you calculate the distance between a cluster and itself  (`d(X, X) = |X| + |X| - |intersection(X, X)| = |X|`).

This is analogous to the original complete linkage, where `d(X, X)`` equals X's intra-cluster distance for any cluster containing two or more non-identical rows. 
From this perspective, `|clusterset1| - 1` can be interpreted as an approximation of the intra-cluster distance.

Note that functions like `|union(clusterset1, clusterset2)| - |intersection(clusterset1, clusterset2)|` do not satisfy `d(x1 U x2, y1 U y2) >= d(x1, y1) ` and would not work correctly in this context.

### Distance Implementation

The fastest way to implement set operations is by using bitmasks, provided the number of unique categories fits into the masks.
You can mix bitmasks and hashset or [roaring bitmaps](https://docs.rs/roaring/latest/roaring/) for high-cardinality colums.

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
| 10,000,000 |      | 854 s \ = 14 min
| 50,000,000| 2295s = 76 min