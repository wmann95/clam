//! A `Cluster` is a collection of "similar" instances in a dataset.

pub mod adapter;
mod balanced_ball;
mod ball;
mod lfd;
pub mod partition;

use std::collections::HashMap;

use distances::Number;

use super::{dataset::ParDataset, Dataset, MetricSpace};

pub use balanced_ball::BalancedBall;
pub use ball::Ball;
pub use lfd::LFD;
pub use partition::Partition;

/// A `Cluster` is a collection of "similar" instances in a dataset.
///
/// # Type Parameters
///
/// - `I`: The type of the instances in the dataset.
/// - `U`: The type of the distance values between instances.
/// - `D`: The type of the dataset.
///
/// # Remarks
///
/// A `Cluster` must have certain properties to be useful in CLAM. These are:
///
/// - `depth`: The depth of the `Cluster` in the tree.
/// - `cardinality`: The number of instances in the `Cluster`.
/// - `indices`: The indices of the instances in the `Cluster`.
/// - `arg_center`: The index of the geometric median of the instances in the
///   `Cluster`. This may be computed exactly, using all instances in the
///   `Cluster`, or approximately, using a subset of the instances.
/// - `radius`: The distance from the center to the farthest instance in the
///   `Cluster`.
/// - `arg_radial`: The index of the instance that is farthest from the center.
/// - `lfd`: The Local Fractional Dimension of the `Cluster`.
///
/// A `Cluster` may have two or more children, which are `Cluster`s of the same
/// type. The children should be stored as a tuple with:
///
/// - The index of the extremal instance in the `Cluster` that was used to
///   create the child.
/// - The distance from that extremal instance to the farthest instance that was
///   assigned to the child. We refer to this as the "extent" of the child.
/// - The child `Cluster`.
pub trait Cluster<I, U: Number, D: Dataset<I, U>>: Ord + core::hash::Hash + Sized {
    /// Returns the depth os the `Cluster` in the tree.
    fn depth(&self) -> usize;

    /// Returns the cardinality of the `Cluster`.
    fn cardinality(&self) -> usize;

    /// Returns the index of the center instance in the `Cluster`.
    fn arg_center(&self) -> usize;

    /// Sets the index of the center instance in the `Cluster`.
    ///
    /// This is used to find the center instance after permutation.
    fn set_arg_center(&mut self, arg_center: usize);

    /// Returns the radius of the `Cluster`.
    fn radius(&self) -> U;

    /// Returns the index of the radial instance in the `Cluster`.
    fn arg_radial(&self) -> usize;

    /// Sets the index of the radial instance in the `Cluster`.
    ///
    /// This is used to find the radial instance after permutation.
    fn set_arg_radial(&mut self, arg_radial: usize);

    /// Returns the Local Fractional Dimension (LFD) of the `Cluster`.
    fn lfd(&self) -> f32;

    /// Gets the indices of the instances in the `Cluster`.
    fn indices(&self) -> impl Iterator<Item = usize> + '_;

    /// Sets the indices of the instances in the `Cluster`.
    fn set_indices(&mut self, indices: Vec<usize>);

    /// Returns the children of the `Cluster`.
    #[must_use]
    fn children(&self) -> &[(usize, U, Box<Self>)];

    /// Returns the children of the `Cluster` as mutable references.
    #[must_use]
    fn children_mut(&mut self) -> &mut [(usize, U, Box<Self>)];

    /// Sets the children of the `Cluster`.
    fn set_children(&mut self, children: Vec<(usize, U, Box<Self>)>);

    /// Returns the owned children and sets the cluster's children to an empty vector.
    fn take_children(&mut self) -> Vec<(usize, U, Box<Self>)>;

    /// Computes the distances from the `query` to all instances in the `Cluster`.
    fn distances_to_query(&self, data: &D, query: &I) -> Vec<(usize, U)>;

    /// Returns whether the `Cluster` is a descendant of another `Cluster`.
    fn is_descendant_of(&self, other: &Self) -> bool;

    /// Clears the indices stored with every cluster in the tree.
    fn clear_indices(&mut self) {
        if !self.is_leaf() {
            self.child_clusters_mut().for_each(Self::clear_indices);
        }
        self.set_indices(Vec::new());
    }

    /// Trims the tree at the given depth. Returns the trimmed roots in the same
    /// order as the leaves of the trimmed tree at that depth.
    fn trim_at_depth(&mut self, depth: usize) -> Vec<Vec<(usize, U, Box<Self>)>> {
        let mut queue = vec![self];
        let mut stack = Vec::new();

        while let Some(c) = queue.pop() {
            if c.depth() == depth {
                stack.push(c);
            } else {
                queue.extend(c.child_clusters_mut());
            }
        }

        stack.into_iter().map(Self::take_children).collect()
    }

    /// Inverts the `trim_at_depth` method.
    fn graft_at_depth(&mut self, depth: usize, trimmings: Vec<Vec<(usize, U, Box<Self>)>>) {
        let mut queue = vec![self];
        let mut stack = Vec::new();

        while let Some(c) = queue.pop() {
            if c.depth() == depth {
                stack.push(c);
            } else {
                queue.extend(c.child_clusters_mut());
            }
        }

        stack
            .into_iter()
            .zip(trimmings)
            .for_each(|(c, children)| c.set_children(children));
    }

    /// Gets the child `Cluster`s.
    fn child_clusters<'a>(&'a self) -> impl Iterator<Item = &Self>
    where
        U: 'a,
    {
        self.children().iter().map(|(_, _, child)| child.as_ref())
    }

    /// Gets the child `Cluster`s as mutable references.
    fn child_clusters_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut Self>
    where
        U: 'a,
    {
        self.children_mut().iter_mut().map(|(_, _, child)| child.as_mut())
    }

    /// Returns all `Cluster`s in the subtree of this `Cluster`, in depth-first order.
    fn subtree<'a>(&'a self) -> Vec<&'a Self>
    where
        U: 'a,
    {
        let mut clusters = vec![self];
        self.child_clusters().for_each(|child| clusters.extend(child.subtree()));
        clusters
    }

    /// Returns the subtree, with unique integers to reference the parents and children of each `Cluster` in a `HashMap`.
    ///
    /// The Vec contains tuples of:
    ///
    /// - The `Cluster` itself.
    /// - The index of the `Cluster` in the Vec.
    /// - The position of the `Cluster` among its siblings.
    /// - A Vec of tuples of:
    ///  - The index of the parent `Cluster` in the Vec.
    ///  - A tuple of:
    ///     - The index of the parent `Cluster` in the Vec.
    ///    - The extent of the child.
    #[allow(clippy::type_complexity)]
    fn take_subtree(mut self) -> Vec<(Self, usize, usize, Vec<(usize, (usize, U))>)> {
        let children = self.take_children();
        let mut clusters = vec![(self, 0, 0, vec![])];

        for (e, d, children) in children.into_iter().map(|(e, d, c)| (e, d, c.take_subtree())) {
            let offset = clusters.len();

            for (ci, (child, parent_index, _, children_indices)) in children.into_iter().enumerate() {
                let parent_index = parent_index + offset;
                let children_indices = children_indices.into_iter().map(|(pi, ed)| (pi + offset, ed)).collect();
                clusters.push((child, parent_index, ci, children_indices));
            }

            clusters[0].3.push((offset, (e, d)));
        }

        clusters
    }

    /// Returns the subtree as a list of `Cluster`s, with the indices required
    /// to go from a parent to a child and vice versa.
    ///
    /// The Vec contains tuples of:
    ///
    /// - The `Cluster` itself.
    /// - The position of the `Cluster` among its siblings.
    /// - A Vec of tuples of:
    ///  - The index of the parent `Cluster` in the Vec.
    ///  - A tuple of:
    ///     - The index of the parent `Cluster` in the Vec.
    ///    - The extent of the child.
    #[allow(clippy::type_complexity)]
    fn unstack_tree(self) -> Vec<(Self, usize, Vec<(usize, (usize, U))>)> {
        let mut subtree = self.take_subtree();
        subtree.sort_by_key(|(_, i, _, _)| *i);
        subtree
            .into_iter()
            .map(|(c, _, ci, children)| (c, ci, children))
            .collect()
    }

    /// Inverts the `unstack_tree` method.
    ///
    /// The Vec contains tuples of:
    ///
    /// - The `Cluster` itself.
    /// - The position of the `Cluster` among its siblings.
    /// - A Vec of tuples of:
    ///  - The index of the parent `Cluster` in the Vec.
    ///  - A tuple of:
    ///     - The index of the parent `Cluster` in the Vec.
    ///    - The extent of the child.
    #[allow(clippy::type_complexity)]
    fn restack_tree(list: Vec<(Self, usize, Vec<(usize, (usize, U))>)>) -> Self {
        let mut list = list.into_iter().enumerate().collect::<HashMap<_, _>>();
        let mut leaves;

        while list.len() > 1 {
            (leaves, list) = list.into_iter().partition(|(_, (_, _, children))| children.is_empty());

            let mut grouped_leaves = HashMap::new();
            for (pi, (leaf, ci, _)) in leaves {
                let entry = grouped_leaves.entry(pi).or_insert_with(Vec::new);
                entry.push((ci, leaf));
            }

            for (pi, mut children) in grouped_leaves {
                children.sort_by_key(|(ci, _)| *ci);
                let (parent, _, eds) = list.get_mut(&pi).unwrap_or_else(|| unreachable!("Parent not found"));

                let children = children.into_iter().map(|(_, c)| Box::new(c));
                let eds = eds.iter().map(|(_, (e, d))| (*e, *d));
                let children = children.zip(eds).map(|(c, (e, d))| (e, d, c)).collect();
                parent.set_children(children);
            }
        }

        list.into_iter()
            .next()
            .unwrap_or_else(|| unreachable!("Root not found"))
            .1
             .0
    }

    /// Returns all leaf `Cluster`s in the subtree of this `Cluster`, in depth-first order.
    fn leaves<'a>(&'a self) -> Vec<&'a Self>
    where
        U: 'a,
    {
        let mut queue = vec![self];
        let mut stack = vec![];

        while let Some(cluster) = queue.pop() {
            if cluster.is_leaf() {
                stack.push(cluster);
            } else {
                queue.extend(cluster.child_clusters());
            }
        }

        stack
    }

    /// Returns mutable references to all leaf `Cluster`s in the subtree of this `Cluster`, in depth-first order.
    fn leaves_mut<'a>(&'a mut self) -> Vec<&'a mut Self>
    where
        U: 'a,
    {
        let mut queue = vec![self];
        let mut stack = vec![];

        while let Some(cluster) = queue.pop() {
            if cluster.is_leaf() {
                stack.push(cluster);
            } else {
                queue.extend(cluster.child_clusters_mut());
            }
        }

        stack
    }

    /// Whether the `Cluster` is a leaf in the tree.
    fn is_leaf(&self) -> bool {
        self.children().is_empty()
    }

    /// Whether the `Cluster` is a singleton.
    fn is_singleton(&self) -> bool {
        self.cardinality() == 1 || self.radius() < U::EPSILON
    }

    /// Computes the distance from the `Cluster`'s center to a given `query`.
    fn distance_to_center(&self, data: &D, query: &I) -> U {
        let center = data.get(self.arg_center());
        MetricSpace::one_to_one(data, center, query)
    }

    /// Computes the distance from the `Cluster`'s center to another `Cluster`'s center.
    fn distance_to_other(&self, data: &D, other: &Self) -> U {
        Dataset::one_to_one(data, self.arg_center(), other.arg_center())
    }
}

/// A parallelized version of the `Cluster` trait.
#[allow(clippy::module_name_repetitions)]
pub trait ParCluster<I: Send + Sync, U: Number, D: ParDataset<I, U>>: Cluster<I, U, D> + Send + Sync {
    /// Parallelized version of the `distances_to_query` method.
    fn par_distances_to_query(&self, data: &D, query: &I) -> Vec<(usize, U)>;
}
