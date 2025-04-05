pub use crate::chunk::{Chunk, ChunkType, OverlapType, Overlaps};
pub use crate::chunk_node;
pub use errors::ChunkError;

use self::chunk_node::ChunkNode;

pub mod errors {

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    pub enum ChunkError<T: ChunkType> {
        InvalidChunk(Chunk<T>),
        ChunksOverlaps(Chunk<T>, Chunk<T>),
        ChunksDoNotOverlaps(Chunk<T>, Chunk<T>),
        IncorrectChunksOrder(Chunk<T>, Chunk<T>),
    }

    impl<T: ChunkType> std::fmt::Display for ChunkError<T> {
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
            }
        }
    }
}

pub struct IntervalList<T: ChunkType> {
    pub head: Option<Box<ChunkNode<T>>>,
}

impl<T: ChunkType> PartialEq for IntervalList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter()
            .zip(other.iter())
            .try_for_each(|(chunk1, chunk2)| if chunk1 == chunk2 { Ok(()) } else { Err(()) })
            .is_ok()
    }
}

impl<T: ChunkType> IntervalList<T> {
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
                        OverlapType::Left => node.begin = chunk.begin.clone(),
                        OverlapType::Right => node.end = chunk.end.clone(),
                        OverlapType::Inside => node.chunk = chunk.clone(),
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
                                    OverlapType::Left => {
                                        node.end = node.next_chunk.as_ref().unwrap().end.clone()
                                    }
                                    OverlapType::Right => unreachable!(
                                        "{}, {}",
                                        chunk,
                                        node.next_chunk.as_ref().unwrap().chunk
                                    ),
                                    OverlapType::Inside => _break = false,
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
                            let current_node = node;
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
                        Err(e) => {
                            return Err(e);
                        }
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

    pub fn from_intervals(intervals: Vec<Chunk<T>>) -> Result<Self, ChunkError<T>> {
        let mut list = Self::new();

        intervals
            .into_iter()
            .try_for_each(|chunk| list.add_chunk(chunk))?;

        Ok(list)
    }

    pub fn get_complement_intervals(
        &self,
        start_to_end_chunk: Chunk<T>,
    ) -> Result<Self, ChunkError<T>> {
        let mut last_chunk_end = start_to_end_chunk.begin;

        let mut list = IntervalList::new();
        let mut node = &self.head;

        while let Some(current) = node {
            if current.begin > start_to_end_chunk.end {
                last_chunk_end = start_to_end_chunk.end.clone();
                break;
            }
            if let Ok(chunk) = Chunk::new(last_chunk_end.clone(), current.begin.clone()) {
                list.add_chunk(chunk)?;
            };

            last_chunk_end = current.end.clone();
            node = &current.next_chunk;
        }

        if last_chunk_end < start_to_end_chunk.end {
            list.add_chunk(Chunk::new(last_chunk_end, start_to_end_chunk.end)?)?;
        }

        Ok(list)
    }

    pub fn get_interval_by_index(&self, idx: usize) -> Result<&Chunk<T>, std::io::Error> {
        if idx >= self.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "The index went out of list length",
            ));
        }

        let mut current = &self.head;

        let mut cur_idx = 0;

        while let Some(node) = current {
            if cur_idx == idx {
                return Ok(&node.chunk);
            }

            cur_idx += 1;
            current = &node.next_chunk;
        }

        unreachable!("Function: get_inteverl_by_index, the part can not be reached");
    }
}

pub struct IntervalIterator<'a, T: ChunkType> {
    current: Option<&'a ChunkNode<T>>,
}

impl<'a, T: ChunkType> Iterator for IntervalIterator<'a, T> {
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

impl<T: ChunkType> std::fmt::Display for IntervalList<T> {
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

impl<T: ChunkType> std::fmt::Debug for IntervalList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IntervalList{{ ")?;
        let mut first = true;
        for chunk in self.iter() {
            if !first {
                write!(f, ", ")?;
            }
            write!(
                f,
                "Interval {{start: {:?}, end: {:?} }}",
                chunk.begin, chunk.end
            )?;
            first = false;
        }
        write!(f, "}}")
    }
}
