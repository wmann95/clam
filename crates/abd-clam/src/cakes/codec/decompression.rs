//! Traits and an implementation for decompressing datasets.

use std::collections::HashMap;

use distances::Number;

use crate::{dataset::ParDataset, Dataset};

/// A trait that defines how a value can be decoded in terms of a reference.
pub trait Decodable {
    /// Decodes the value from a byte array.
    fn from_bytes(bytes: &[u8]) -> Self;

    /// Decodes the value in terms of a reference.
    fn decode(reference: &Self, bytes: &[u8]) -> Self;
}

/// A trait that defines how a dataset can be decompressed.
pub trait Decompressible<I: Decodable, U: Number>: Dataset<I, U> + Sized {
    /// Returns the centers of the clusters in the tree associated with this
    /// dataset.
    fn centers(&self) -> &HashMap<usize, I>;

    /// Returns the bytes slice representing all compressed leaves.
    fn leaf_bytes(&self) -> &[u8];

    /// Returns the offsets of the leaves' encodings in the bytes.
    fn leaf_offsets(&self) -> &[usize];

    /// Finds the offset of the leaf's instances in the compressed form, given
    /// the offset of the leaf in decompressed form.
    fn find_compressed_offset(&self, decompressed_offset: usize) -> usize;

    /// Decodes all the instances of a leaf cluster in terms of its center.
    fn decode_leaf(&self, mut offset: usize) -> Vec<I> {
        let mut instances = Vec::new();
        let bytes = self.leaf_bytes();

        let arg_center = super::read_usize(bytes, &mut offset);
        let center = &self.centers()[&arg_center];

        let cardinality = super::read_usize(bytes, &mut offset);

        for _ in 0..cardinality {
            let encoding = super::read_encoding(bytes, &mut offset);
            let instance = I::decode(center, &encoding);
            instances.push(instance);
        }

        instances
    }
}

/// Parallel version of the `Decompressible` trait.
pub trait ParDecompressible<I: Decodable + Send + Sync, U: Number>: Decompressible<I, U> + ParDataset<I, U> {
    /// Parallel version of the `decode_leaf` method.
    fn par_decode_leaf(&self, offset: usize) -> Vec<I> {
        self.decode_leaf(offset)
    }
}
