use chunk::Chunk;
use errors::ChunkError;

pub mod errors {
    use crate::core::*;
    #[derive(Debug, PartialEq, Eq)]
    pub enum ChunkError<T: Clone + std::fmt::Debug + PartialOrd + Ord> {
        InvalidChunk(chunk::Chunk<T>),
        ChunksOverlaps(chunk::Chunk<T>, chunk::Chunk<T>),
        ChunksDoNotOverlaps(chunk::Chunk<T>, chunk::Chunk<T>),
        IncorrectChunksOrder(chunk::Chunk<T>, chunk::Chunk<T>),
    }

    impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> std::fmt::Display for ChunkError<T> {
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

    #[derive(Clone, Debug)]
    pub enum OverlapType {
        Left,
        Right,
        Inside,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Chunk<T: Clone + std::fmt::Debug + PartialOrd + Ord> {
        pub begin: T,
        pub end: T,
    }

    impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> Into<(T, T)> for Chunk<T> {
        fn into(self) -> (T, T) {
            (self.begin, self.end)
        }
    }

    impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> TryFrom<(T, T)> for Chunk<T> {
        type Error = ChunkError<T>;

        fn try_from(value: (T, T)) -> Result<Self, Self::Error> {
            Chunk::new(value.0, value.1)
        }
    }

    impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> std::fmt::Display for Chunk<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{:?} -> {:?}]", self.begin, self.end)
        }
    }

    impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> Chunk<T> {
        pub fn new(begin: T, end: T) -> Result<Self, ChunkError<T>> {
            if begin >= end {
                return Err(ChunkError::InvalidChunk(Chunk { begin, end }));
            }

            Ok(Self { begin, end })
        }

        pub fn can_be_followed(&self, chunk: &Self) -> bool {
            if self.end <= chunk.begin {
                return true;
            }
            false
        }

        pub fn overlaps(chunk1: &Self, chunk2: &Self) -> Overlaps {
            if (chunk1.begin > chunk2.begin && chunk2.end > chunk1.begin)
                || (chunk1.begin < chunk2.begin && chunk1.end > chunk2.begin)
            {
                return Overlaps::Overlaps;
            }

            if chunk1.end == chunk2.begin || chunk1.begin == chunk2.end {
                return Overlaps::CanBeOptimized;
            }

            Overlaps::DoNotOverlaps
        }

        pub fn get_overlap_type(
            chunk1: &Self,
            chunk2: &Self,
        ) -> Result<OverlapType, ChunkError<T>> {
            match Self::overlaps(chunk1, chunk2) {
                Overlaps::Overlaps => {
                    Ok(if chunk2.begin < chunk1.begin && chunk2.end < chunk1.end {
                        OverlapType::Left
                    } else if chunk2.begin < chunk1.begin && chunk2.end > chunk1.end {
                        OverlapType::Inside
                    } else {
                        OverlapType::Right
                    })
                }
                _ => Err(ChunkError::ChunksDoNotOverlaps(
                    chunk1.clone(),
                    chunk2.clone(),
                )),
            }
        }

        pub fn try_combine(chunk1: Self, chunk2: Self) -> Result<Self, ChunkError<T>> {
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
    pub struct ChunkNode<T: Clone + std::fmt::Debug + PartialOrd + Ord> {
        pub chunk: Chunk<T>,
        pub next_chunk: Option<Box<ChunkNode<T>>>,
    }

    pub mod impls {
        use super::*;
        impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> From<Chunk<T>> for ChunkNode<T> {
            fn from(value: Chunk<T>) -> Self {
                Self {
                    chunk: value,
                    next_chunk: None,
                }
            }
        }

        pub mod derefs {
            use super::*;
            impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> Deref for ChunkNode<T> {
                type Target = Chunk<T>;

                fn deref(&self) -> &Self::Target {
                    &self.chunk
                }
            }

            impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> DerefMut for ChunkNode<T> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.chunk
                }
            }
        }
    }

    impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> ChunkNode<T> {
        // returns an error if chunks overlaps
        pub fn set_next_chunk(&mut self, chunk: Chunk<T>) -> Result<(), ChunkError<T>> {
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

pub struct IntervalList<T: Clone + std::fmt::Debug + PartialOrd + Ord> {
    pub head: Option<Box<ChunkNode<T>>>,
}

impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> IntervalList<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn add_chunk(&mut self, chunk: Chunk<T>) -> Result<(), ChunkError<T>> {
        if self.is_empty() {
            self.head = Some(Box::new(chunk.into()));
            return Ok(());
        }

        let mut current = &mut self.head;

        // Plan: move through the list, if a new chunk can be optimized with the current chunk, do it and end. Else check if it fits before the current node paste it and break.
        while let Some(node) = current {
            match Chunk::overlaps(&node.chunk, &chunk) {
                Overlaps::Overlaps => {
                    //change curent chunk
                    match Chunk::get_overlap_type(&node.chunk, &chunk).unwrap() {
                        chunk::OverlapType::Left => node.begin = chunk.begin.clone(),
                        chunk::OverlapType::Right => node.end = chunk.end.clone(),
                        chunk::OverlapType::Inside => node.chunk = chunk.clone(),
                    }

                    loop {
                        if node.next_chunk.is_none() {
                            break;
                        }

                        let mut _break = true;
                        match Chunk::overlaps(&node.next_chunk.as_ref().unwrap().chunk, &chunk) {
                            Overlaps::Overlaps => {
                                match Chunk::get_overlap_type(
                                    &node.next_chunk.as_ref().unwrap().chunk,
                                    &chunk,
                                )
                                .unwrap()
                                {
                                    chunk::OverlapType::Left => {
                                        println!("HUI");
                                        node.end = node.next_chunk.as_ref().unwrap().end.clone()
                                    }
                                    chunk::OverlapType::Right => unreachable!(
                                        "{}, {}",
                                        chunk,
                                        node.next_chunk.as_ref().unwrap().chunk
                                    ),
                                    chunk::OverlapType::Inside => _break = false,
                                }
                            }

                            Overlaps::CanBeOptimized => {}
                            Overlaps::DoNotOverlaps => break,
                        }

                        node.next_chunk = node.next_chunk.as_mut().unwrap().next_chunk.take();

                        if _break {
                            break;
                        }
                    }

                    return Ok(());
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
                        let mut new_node: Box<ChunkNode<T>> = Box::new(chunk.into());
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

    pub fn iter(&self) -> IntervalIterator<T> {
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

    pub fn contains(&self, value: T) -> bool {
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

    pub fn total_range(&self) -> Option<(T, T)> {
        if let Some(chunk) = self.head.as_ref() {
            let left_bound = chunk.begin.clone();

            let mut rigth_bound_el = &(chunk.chunk.clone());

            self.iter().for_each(|el| rigth_bound_el = el);

            let right_bound = rigth_bound_el.end.clone();

            return Some((left_bound, right_bound));
        }
        None
    }
}

pub struct IntervalIterator<'a, T: Clone + std::fmt::Debug + PartialOrd + Ord> {
    current: Option<&'a ChunkNode<T>>,
}

impl<'a, T: Clone + std::fmt::Debug + PartialOrd + Ord> Iterator for IntervalIterator<'a, T> {
    type Item = &'a Chunk<T>;

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

impl<T: Clone + std::fmt::Debug + PartialOrd + Ord> std::fmt::Display for IntervalList<T> {
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
