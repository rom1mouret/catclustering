use crate::data::ClusterSummary;
use crate::dendrogram::Dendrogram;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};

pub(crate) struct Cluster {
    pub(crate) summary: Box<dyn ClusterSummary>,
    pub(crate) merged_into: Option<usize>,
    pub(crate) dendrogram: Option<Dendrogram>,
}

pub(crate) struct Link {
    pub(crate) cluster1_index: usize,
    pub(crate) cluster2_index: usize,
    pub(crate) cluster1_summary_size: usize,
    pub(crate) cluster2_summary_size: usize,
    pub(crate) distance: f32,
}

impl PartialEq for Link {
    fn eq(&self, other: &Link) -> bool {
        return self.distance == other.distance;
    }
}

impl Eq for Link {}

impl PartialOrd for Link {
    fn partial_cmp(&self, other: &Link) -> Option<Ordering> {
        // Reverse the order for a min-heap
        other.distance.partial_cmp(&self.distance)
    }
}

impl Ord for Link {
    fn cmp(&self, other: &Link) -> Ordering {
        // Reverse the order for a min-heap
        other.distance.total_cmp(&self.distance)
    }
}

impl Cluster {
    pub(crate) fn summary_size(&self) -> usize {
        self.summary.summary_size()
    }

    pub(crate) fn distance(&self, other: &Cluster) -> f32 {
        self.summary.distance(&*other.summary)
    }
}
