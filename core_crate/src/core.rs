use crate::chunk::*;
use crate::chunk_node::*;
use errors::ChunkError;

pub mod errors {
    use crate::core::*;
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
                _ => write!(f, "unimplemented error"),
            }
        }
    }
}

pub struct IntervalList<T: ChunkType> {
    pub head: Option<Box<ChunkNode<T>>>,
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
                                        println!("HUI");
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
