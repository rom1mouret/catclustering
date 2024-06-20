# catclustering

Crate link: [crates.io/crates/catclustering](https://crates.io/crates/catclustering)

This crate implements hierarchical agglomerative clustering optimized for categorical features and (approximated) complete linkage.

## Initialization Step

To avoid the computational inefficiency of calculating all pairwise distances for the adjacency matrix, the algorithm identifies neighbors by examining adjacent rows in the sorted dataset. 

Since the order of the columns affects the outcome, the dataset is sorted multiple times, each time with a different random column order.
This approach is roughly equivalent to identifying neighbors through random projections onto a one-dimensional line.
The number of iterations can be controlled with the 'init_iterations' parameter.

This initial step makes this crate particularly well-suited for categorical data.

## Distance Function

To ensure the algorithm works correctly, the distance function must satisfy the following property:

```
d(x1 U x2, y1 U y2) >= d(x1, y1) 
```

for any cluster, *x1*, *x2*, *y1*, *y2*, including empty clusters.

This property is satisfied by the [original complete-linkage distance](https://en.wikipedia.org/wiki/Complete-linkage_clustering).

However, because complete linkage is computationally expensive, we recommend the following approximation, which also exhibits the desired property:

```
|clusterset1| + |clusterset2| - |intersection(clusterset1, clusterset2)|
```

`|clusterset1|` is what remains when you calculate the distance between a cluster and itself  (`d(X, X) = |X| + |X| - |intersection(X, X)| = |X|`).

This is analogous to the original complete linkage, where `d(X, X)` equals X's intra-cluster distance when the cluster contains at least two distinct rows. 
From this perspective, `|clusterset1| - 1` can be interpreted as an approximation of the intra-cluster distance.

Note that functions like `|union(clusterset1, clusterset2)| - |intersection(clusterset1, clusterset2)|` do not satisfy `d(x1 U x2, y1 U y2) >= d(x1, y1) ` and would not work correctly in this context.

### Distance Implementation

The fastest way to implement set operations is by using bit masks, provided the number of unique categories fits into the masks.
You can combine bit masks and hashsets or [roaring bitmaps](https://docs.rs/roaring/latest/roaring/) to fit your needs.


| operation \ data structure          | bitmasks            | hashsets                           | roaring bitmaps                |
|-------------------------------------|---------------------|------------------------------------|--------------------------------|
| cardinality of intersection         | (x&y).count_ones()  | x.intersection(&y).count()         | x.intersection_len(&y)         |
| cardinality of union                | (x\|y).count_ones() | x.union(&y).count()                | x.union_len(&y)                |
| cardinality of symmetric difference | (x^y).count_ones()  | x.symmetric_difference(&y).count() | x.symmetric_difference_len(&y) |
| merge                               | x \|= y             | x.extend(&y)                       | x.extend(&y)                   |

## Example

Here is an example for a dataset that comprises 4 low-cardinality columns and 1 high-cardinality column.

```rust
use std::vec::Vec;
use std::collections::HashSet;
use std::any::Any;
use rand::Rng;

struct SimpleMatrix {
    col1to4: u32,
    col5: HashSet<u16>
}

struct MyData {
    vecs: Vec<Vec<i32>>
}

impl catclustering::ClusterSummary for SimpleMatrix {
    fn summary_size(&self) -> usize {
        self.col1to4.count_ones() as usize + self.col5.len() 
    }
    fn distance(&self, other: &dyn catclustering::ClusterSummary) -> f32 {
        let o = other.as_any().downcast_ref::<SimpleMatrix>().unwrap();
        let intersection = (self.col1to4 & o.col1to4).count_ones();

        (self.summary_size() + other.summary_size()) as f32 - intersection as f32
    }
    
    fn extend(&mut self, other: &dyn catclustering::ClusterSummary) {
        let o = other.as_any().downcast_ref::<SimpleMatrix>().unwrap();

        self.col1to4 |= o.col1to4;
        self.col5.extend(&o.col5);
    }
    fn clear(&mut self) {
        self.col5.clear();
        self.col5.shrink_to_fit();
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}   

impl catclustering::IndexableData for MyData {
    fn get_value(&self, row_index: usize, column_index: usize) -> f32 {
        self.vecs[row_index][column_index] as f32
    }

    fn get_num_columns(&self) -> usize {
        self.vecs[0].len()
    }

    fn get_num_rows(&self) -> usize {
        self.vecs.len()
    }

    fn create_cluster_summary(&self, row_index: usize) -> Box<dyn catclustering::ClusterSummary> {
        let row = &self.vecs[row_index];
        Box::new(SimpleMatrix {
            col1to4: (1 << row[0])
                | (1 << (row[1] + 8))
                | (1 << (row[2] + 16))
                | (1 << (row[3] + 24)),
            col5: HashSet::from_iter(vec![row[4] as u16]),
        })
    }
}

/// Just a utility function to create a random data set
fn create_random_matrix(rows: usize, cardinality: [i32; 5]) -> Vec<Vec<i32>> {
    let mut rng = rand::thread_rng();
    let mut matrix = Vec::with_capacity(rows);

    for _ in 0..rows {
        let row: Vec<i32> = (0..cardinality.len())
            .map(|k| rng.gen_range(0..1000) % cardinality[k])
            .collect();
        matrix.push(row);
    }

    matrix
}

fn main() {
    let matrix = MyData{vecs: create_random_matrix(10_000, [8, 8, 8, 8, 400])};

    // main algorithm
    let mut rng = rand::thread_rng(); 
    let mut dendro = catclustering::create_dendrogram(&matrix, None, &mut rng);
   
    // more interpretable results
    let clusters = catclustering::find_clusters(&dendro, 100);
    
    // or, if you just want the assignments:
    let mut assignments = Vec::new(); // feel free to reuse this vector 
    let num_clusters = catclustering::assign_rows_to_clusters(&dendro, &mut assignments, 100);
}
```

## Benchmarks

The benchmarks are run from the example above.

| rows       | time   |
| ---------- | ------ |
| 10,000     | 61 ms  |
| 100,000    | 840 ms |
| 1,000,000  | 33.8 s |
| 10,000,000 | 14 min |
| 50,000,000 | 76 min |

Specs:

- i7-6700 @ 3.40GHz
- Linux pop-os 6.5.4
- rustc 1.76.0