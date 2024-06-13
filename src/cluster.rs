use std::cmp::{PartialEq, PartialOrd, Ord, Ordering};
use std::any::Any;
use crate::data::CategoryMatrix;
use crate::dendrogram::Dendrogram;

pub(crate) struct Cluster {
    pub(crate) categories: Box<dyn CategoryMatrix>,
    pub(crate) merged_into: Option<usize>,
    pub(crate) dendrogram: Option<Dendrogram>,
}

pub(crate) struct Link {
    pub(crate) cluster1_index: usize,
    pub(crate) cluster2_index: usize,
    pub(crate) cluster1_num_categories: u16,
    pub(crate) cluster2_num_categories: u16,
    pub(crate) distance: u16,
}


impl PartialEq for Link {
    fn eq(&self, other: &Link) -> bool {
        return self.distance == other.distance
    }
}

impl Eq for Link {
   
}

impl PartialOrd for Link {
    fn partial_cmp(&self, other: &Link) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Link {
    fn cmp(&self, other: &Link) -> Ordering {
        // Reverse the order for a min-heap
        other.distance.cmp(&self.distance)
    }
}

impl Cluster {
    pub(crate) fn num_categories(&self) -> u16 {
        self.categories.num_categories()
    }

    pub(crate) fn symmetric_distance(&self, other: &Cluster) -> u16 {
        self.categories.symmetric_distance(&*other.categories)
    }
}