use chunk::Chunk;
use errors::ChunkError;

pub mod errors {
    use crate::core::*;
    #[derive(Debug, PartialEq, Eq)]
    pub enum ChunkError {
        InvalidChunk(chunk::Chunk),
        ChunksOverlaps(chunk::Chunk, chunk::Chunk),
        ChunksDoNotOverlaps(chunk::Chunk, chunk::Chunk),
        IncorrectChunksOrder(chunk::Chunk, chunk::Chunk),
    }

    impl std::fmt::Display for ChunkError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ChunkError::InvalidChunk(chunk) => {
                    write!(f, "The Chunk {} is not valid", chunk)
                }
                ChunkError::ChunksOverlaps(chunk1, chunk2) => {
                    write!(f, "Chunk 1 : {} overlaps with chunk2: {}", chunk1, chunk2)
                }
                ChunkError::ChunksDoNotOverlaps(chunk1, chunk2) => {
                    write!(
                        f,
                        "Chunk 1 : {} does not overlap with chunk2: {}",
                        chunk1, chunk2
                    )
                }
                ChunkError::IncorrectChunksOrder(chunk1, chunk2) => {
                    write!(f, "Chunk2: {} is located before chunk1: {}", chunk2, chunk1)
                }
                _ => write!(f, "unimplemented error"),
            }
        }
    }
}

pub mod chunk {
    use crate::core::errors::ChunkError;

    #[derive(Clone, Debug)]
    pub enum Overlaps {
        Overlaps,
        CanBeOptimized,
        DoNotOverlaps,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Chunk {
        pub begin: u32,
        pub end: u32,
    }

    impl<T> TryFrom<(T, T)> for Chunk
    where
        T: Into<u32>,
    {
        type Error = ChunkError;

        fn try_from(value: (T, T)) -> Result<Self, Self::Error> {
            Chunk::new(value.0.into(), value.1.into())
        }
    }

    impl std::fmt::Display for Chunk {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{} -> {}]", self.begin, self.end)
        }
    }

    impl Chunk {
        pub fn new(begin: u32, end: u32) -> Result<Self, ChunkError> {
            if begin >= end {
                return Err(ChunkError::InvalidChunk(Chunk { begin, end }));
            }

            Ok(Self { begin, end })
        }

        pub fn can_be_followed(&self, chunk: &Chunk) -> bool {
            if self.end <= chunk.begin {
                return true;
            }
            false
        }

        pub fn overlaps(chunk1: &Chunk, chunk2: &Chunk) -> Overlaps {
            if (chunk1.begin > chunk2.begin && chunk1.begin < chunk2.end)
                || (chunk1.end > chunk2.begin && chunk1.end < chunk2.end)
            {
                return Overlaps::Overlaps;
            }

            if chunk1.end == chunk2.begin || chunk1.begin == chunk2.end {
                return Overlaps::CanBeOptimized;
            }

            Overlaps::DoNotOverlaps
        }

        pub fn try_combine(chunk1: Chunk, chunk2: Chunk) -> Result<Chunk, ChunkError> {
            match Chunk::overlaps(&chunk1, &chunk2) {
                Overlaps::Overlaps => Err(ChunkError::ChunksOverlaps(chunk1, chunk2)),
                Overlaps::CanBeOptimized => Chunk::new(chunk1.begin, chunk2.end),
                Overlaps::DoNotOverlaps => Err(ChunkError::ChunksDoNotOverlaps(chunk1, chunk2)),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::core::chunk::{Chunk, Overlaps};
        use crate::core::errors::ChunkError;

        #[test]
        fn test_chunk_creation() {
            // Valid chunks
            let chunk = Chunk::new(10, 20);
            assert!(chunk.is_ok());

            // Invalid chunk (begin >= end)
            let invalid_chunk = Chunk::new(30, 30);
            assert!(invalid_chunk.is_err());
            let invalid_chunk = Chunk::new(40, 30);
            assert!(invalid_chunk.is_err());
        }

        #[test]
        fn test_chunk_overlaps() {
            // Non-overlapping chunks
            let chunk1 = Chunk::new(10, 20).unwrap();
            let chunk2 = Chunk::new(30, 40).unwrap();
            assert!(matches!(
                Chunk::overlaps(&chunk1, &chunk2),
                Overlaps::DoNotOverlaps
            ));

            // Chunks that can be optimized (touching)
            let chunk1 = Chunk::new(10, 20).unwrap();
            let chunk2 = Chunk::new(20, 30).unwrap();
            assert!(matches!(
                Chunk::overlaps(&chunk1, &chunk2),
                Overlaps::CanBeOptimized
            ));

            // Overlapping chunks
            let chunk1 = Chunk::new(10, 25).unwrap();
            let chunk2 = Chunk::new(20, 30).unwrap();
            assert!(matches!(
                Chunk::overlaps(&chunk1, &chunk2),
                Overlaps::Overlaps
            ));
        }

        #[test]
        fn test_try_combine() {
            // Chunks that can be optimized
            let chunk1 = Chunk::new(10, 20).unwrap();
            let chunk2 = Chunk::new(20, 30).unwrap();
            let combined = Chunk::try_combine(chunk1, chunk2);
            assert!(combined.is_ok());
            let combined_chunk = combined.unwrap();
            assert_eq!(combined_chunk.begin, 10);
            assert_eq!(combined_chunk.end, 30);

            // Non-overlapping chunks - should fail
            let chunk1 = Chunk::new(10, 20).unwrap();
            let chunk2 = Chunk::new(30, 40).unwrap();
            let combined = Chunk::try_combine(chunk1, chunk2);
            assert!(combined.is_err());

            // Overlapping chunks - should fail
            let chunk1 = Chunk::new(10, 25).unwrap();
            let chunk2 = Chunk::new(20, 30).unwrap();
            let combined = Chunk::try_combine(chunk1, chunk2);
            assert!(combined.is_err());
        }

        #[test]
        fn test_from_tuple() {
            assert_eq!((10_u32, 29_u32).try_into(), Chunk::new(10, 29));
        }
    }
}

pub mod chunk_node {
    use crate::core::chunk::{Chunk, Overlaps};
    use crate::core::errors::ChunkError;
    use std::ops::{Deref, DerefMut};

    // I will try to implement something like os uses to store pages intervals
    #[derive(Clone, Debug)]
    pub struct ChunkNode {
        pub chunk: Chunk,
        pub next_chunk: Option<Box<ChunkNode>>,
    }

    pub mod impls {
        use super::*;
        impl From<Chunk> for ChunkNode {
            fn from(value: Chunk) -> Self {
                Self {
                    chunk: value,
                    next_chunk: None,
                }
            }
        }

        pub mod derefs {
            use super::*;
            impl Deref for ChunkNode {
                type Target = Chunk;

                fn deref(&self) -> &Self::Target {
                    &self.chunk
                }
            }

            impl DerefMut for ChunkNode {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.chunk
                }
            }
        }
    }

    impl ChunkNode {
        // returns an error if chunks overlaps
        pub fn set_next_chunk(&mut self, chunk: Chunk) -> Result<(), ChunkError> {
            match Chunk::try_combine(self.chunk.clone(), chunk.clone()) {
                Ok(new_chunk) => {
                    self.chunk = new_chunk.into();
                    Ok(())
                }
                Err(err) => match err {
                    ChunkError::ChunksOverlaps(_, _) => Err(err),
                    ChunkError::ChunksDoNotOverlaps(_, _) => {
                        self.next_chunk = Some(Box::new(chunk.into()));
                        Ok(())
                    }
                    ChunkError::IncorrectChunksOrder(_, _) => Err(err),
                    _ => unreachable!("try combine can not return this error"),
                },
            }
        }
    }
}

use crate::core::chunk::Overlaps;
use crate::core::chunk_node::ChunkNode;

pub struct IntervalList {
    pub head: Option<Box<ChunkNode>>,
}

impl IntervalList {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn add_chunk(&mut self, chunk: Chunk) -> Result<(), ChunkError> {
        if self.is_empty() {
            self.head = Some(Box::new(chunk.into()));
            return Ok(());
        }

        let mut current = &mut self.head;

        // Plan: move through the list, if a new chunk can be optimized with the current chunk, do it and end. Else check if it fits before the current node paste it and break.
        while let Some(node) = current {
            match Chunk::overlaps(&node.chunk, &chunk) {
                Overlaps::Overlaps => {
                    return Err(ChunkError::ChunksOverlaps(node.chunk.clone(), chunk));
                }
                Overlaps::CanBeOptimized => {
                    match Chunk::try_combine(node.chunk.clone(), chunk.clone()) {
                        Ok(new_chunk) => {
                            node.chunk = new_chunk;
                            // After merging, we might need to merge with the next chunks as well
                            let mut current_node = node;
                            loop {
                                if current_node.next_chunk.is_none() {
                                    break;
                                }

                                let can_merge = match Chunk::overlaps(
                                    &current_node.chunk,
                                    &current_node.next_chunk.as_ref().unwrap().chunk,
                                ) {
                                    Overlaps::CanBeOptimized => true,
                                    _ => false,
                                };

                                if can_merge {
                                    if let Ok(merged) = Chunk::try_combine(
                                        current_node.chunk.clone(),
                                        current_node.next_chunk.as_ref().unwrap().chunk.clone(),
                                    ) {
                                        current_node.chunk = merged;
                                        // Remove the next node since it's now merged
                                        current_node.next_chunk = current_node
                                            .next_chunk
                                            .as_mut()
                                            .unwrap()
                                            .next_chunk
                                            .take();
                                        // Continue checking with the new next node
                                        continue;
                                    }
                                }

                                // If we can't merge with the next node, we're done
                                break;
                            }
                            return Ok(());
                        }
                        Err(e) => return Err(e),
                    }
                }
                Overlaps::DoNotOverlaps => {
                    if chunk.end <= node.chunk.begin {
                        let mut new_node: Box<ChunkNode> = Box::new(chunk.into());
                        new_node.next_chunk = Some(Box::new(node.as_ref().clone()));
                        *node = new_node;
                        return Ok(());
                    }
                }
            }

            // Move to the next node
            if node.next_chunk.is_none() {
                // If we've reached the end, add the chunk at the end
                node.next_chunk = Some(Box::new(chunk.into()));
                return Ok(());
            }

            current = &mut node.next_chunk;
        }

        Ok(())
    }

    pub fn iter(&self) -> IntervalIterator {
        IntervalIterator {
            current: self.head.as_ref().map(|node| &**node),
        }
    }

    pub fn clear(&mut self) {
        self.head = None;
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        let mut current = self.head.as_ref();

        while let Some(node) = current {
            count += 1;
            current = node.next_chunk.as_ref();
        }

        count
    }

    pub fn contains(&self, value: u32) -> bool {
        let mut current = self.head.as_ref();

        while let Some(node) = current {
            if value >= node.chunk.begin && value < node.chunk.end {
                return true;
            }

            if value < node.chunk.begin {
                return false;
            }

            current = node.next_chunk.as_ref();
        }

        false
    }

    pub fn total_range(&self) -> Option<(u32, u32)> {
        if let Some(chunk) = self.head.as_ref() {
            let left_bound = chunk.begin;

            let mut rigth_bound_el = &(chunk.chunk.clone());

            self.iter().for_each(|el| rigth_bound_el = el);

            let right_bound = rigth_bound_el.end;

            return Some((left_bound, right_bound));
        }
        None
    }
}

pub struct IntervalIterator<'a> {
    current: Option<&'a ChunkNode>,
}

impl<'a> Iterator for IntervalIterator<'a> {
    type Item = &'a Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.current {
            let chunk = &node.chunk;
            self.current = node.next_chunk.as_ref().map(|node| &**node);
            Some(chunk)
        } else {
            None
        }
    }
}

impl std::fmt::Display for IntervalList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IntervalList[")?;
        let mut first = true;
        for chunk in self.iter() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", chunk)?;
            first = false;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::chunk::Chunk;

    #[test]
    fn test_empty_list() {
        let list = IntervalList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.total_range(), None);
    }

    #[test]
    fn test_add_single_chunk() {
        let mut list = IntervalList::new();
        let chunk = Chunk::new(10, 20).unwrap();

        assert!(list.add_chunk(chunk).is_ok());
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert_eq!(list.total_range(), Some((10, 20)));
    }

    #[test]
    fn test_add_non_overlapping_chunks() {
        let mut list = IntervalList::new();

        assert!(list.add_chunk(Chunk::new(10, 20).unwrap()).is_ok());
        assert!(list.add_chunk(Chunk::new(30, 40).unwrap()).is_ok());
        assert!(list.add_chunk(Chunk::new(50, 60).unwrap()).is_ok());

        assert_eq!(list.len(), 3);
        assert_eq!(list.total_range(), Some((10, 60)));
    }

    #[test]
    fn test_add_optimizable_chunks() {
        let mut list = IntervalList::new();

        assert!(list.add_chunk(Chunk::new(10, 20).unwrap()).is_ok());
        assert!(list.add_chunk(Chunk::new(20, 30).unwrap()).is_ok()); // Should optimize with previous

        assert_eq!(list.len(), 1);
        assert_eq!(list.total_range(), Some((10, 30)));
    }

    #[test]
    fn test_add_overlapping_chunks() {
        let mut list = IntervalList::new();

        assert!(list.add_chunk(Chunk::new(10, 25).unwrap()).is_ok());

        let result = list.add_chunk(Chunk::new(20, 30).unwrap());
        assert!(result.is_err());

        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_contains() {
        let mut list = IntervalList::new();

        assert!(list.add_chunk(Chunk::new(10, 20).unwrap()).is_ok());
        assert!(list.add_chunk(Chunk::new(30, 40).unwrap()).is_ok());

        assert!(list.contains(15));
        assert!(list.contains(35));
        assert!(!list.contains(25));
        assert!(!list.contains(5));
        assert!(!list.contains(45));
    }

    #[test]
    fn test_hard_merge() {
        let mut list = IntervalList::new();

        assert!(list.add_chunk(Chunk::new(0, 20).unwrap()).is_ok());
        assert!(list.add_chunk(Chunk::new(40, 50).unwrap()).is_ok());

        assert_eq!(list.len(), 2);

        assert!(list.add_chunk(Chunk::new(20, 40).unwrap()).is_ok());

        assert_eq!(list.len(), 1);

        assert_eq!(list.total_range(), Some((0, 50)));
    }

    #[test]
    fn test_clear() {
        let mut list = IntervalList::new();

        assert!(list.add_chunk(Chunk::new(10, 20).unwrap()).is_ok());
        assert!(list.add_chunk(Chunk::new(30, 40).unwrap()).is_ok());

        assert_eq!(list.len(), 2);

        list.clear();

        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_add_in_middle() {
        let mut list = IntervalList::new();

        assert!(list.add_chunk(Chunk::new(10, 20).unwrap()).is_ok());
        assert!(list.add_chunk(Chunk::new(40, 50).unwrap()).is_ok());
        assert!(list.add_chunk(Chunk::new(25, 35).unwrap()).is_ok()); // Add in the middle

        assert_eq!(list.len(), 3);
        assert!(list.contains(15));
        assert!(list.contains(30));
        assert!(list.contains(45));
        assert!(!list.contains(22));
        assert!(!list.contains(38));
    }

    #[test]
    fn test_tobeggining() {
        let mut list = IntervalList::new();

        assert!(list.add_chunk((20_u32, 40_u32).try_into().unwrap()).is_ok());
        assert!(list.add_chunk((10_u32, 14_u32).try_into().unwrap()).is_ok());
        assert!(list.add_chunk((50_u32, 55_u32).try_into().unwrap()).is_ok());

        assert!(list.contains(10));
        assert!(!list.contains(5));

        let mut len = 0;

        list.iter().for_each(|el| {
            println!("{}", el);
            len += 1;
        });

        assert_eq!(len, 3);
    }
}
