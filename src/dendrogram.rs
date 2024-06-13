pub enum Dendrogram {
    Leaf(usize),  // row index
    Node(Box<Dendrogram>, Box<Dendrogram>, u16, usize), // cluster1, cluster2, distance, size
}

impl Dendrogram {
    pub fn size(&self) -> usize {
        match self {
            Dendrogram::Leaf(_) => 1,
            Dendrogram::Node(_, _, _, s) => *s,
        }
    }
}


fn assign_rows_to<'a>(cluster_n: usize, dendrogram: &'a Dendrogram, assigments: &mut Vec<usize>, stack: &mut Vec<&'a Dendrogram>) {
    stack.clear();
    stack.push(dendrogram);

    while let Some(current) = stack.pop() {
        match current {
            Dendrogram::Leaf(row_index) => {
                assigments[*row_index] = cluster_n;
            }
            Dendrogram::Node(cluster1, cluster2, _, size) => {
                stack.push(cluster1);
                stack.push(cluster2);
            }
        }
    }
}

// returns the number of clusters
pub fn assign_rows_to_clusters(dendrogram: &Dendrogram, assignments: &mut Vec<usize>, max_cluster_size: usize) -> usize {
    let mut cluster_n = 0;
    let mut stack = Vec::new();
    let mut same_cluster_stack = Vec::new();
    stack.push(dendrogram);

    while let Some(current) = stack.pop() {
        match current {
            Dendrogram::Leaf(row_index) => {
                assignments[*row_index] = cluster_n;
                cluster_n += 1;
            }
            Dendrogram::Node(cluster1, cluster2, _, size) => {
                if *size > max_cluster_size {
                    // dive deeper
                    stack.push(cluster1);
                    stack.push(cluster2);
                } else {
                    assign_rows_to(cluster_n, &current, assignments, &mut same_cluster_stack);
                    cluster_n += 1;
                }
            }
        }
    }
    cluster_n
}

pub fn find_clusters(dendrogram: &Dendrogram, n_rows: usize, max_cluster_size: usize) -> Vec<Vec<usize>> {
    let mut assignments = vec![usize::MAX as usize; n_rows];
    let num_clusters = assign_rows_to_clusters(&dendrogram, &mut assignments, max_cluster_size);
    let mut clusters: Vec<Vec<usize>> = (0..num_clusters).map(|_| Vec::new()).collect();

    for (row_idx, assignment) in assignments.iter().enumerate() {
        clusters[*assignment].push(row_idx)
    }
    return clusters
}