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

//TODO delete
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

    pub fn get_overlap_type(chunk1: &Self, chunk2: &Self) -> Result<OverlapType, ChunkError<T>> {
        match Self::overlaps(chunk1, chunk2) {
            Overlaps::Overlaps => Ok(if chunk2.begin < chunk1.begin && chunk2.end < chunk1.end {
                OverlapType::Left
            } else if chunk2.begin < chunk1.begin && chunk2.end > chunk1.end {
                OverlapType::Inside
            } else {
                OverlapType::Right
            }),
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

    pub fn convert<U: Clone + std::fmt::Debug + PartialOrd + Ord + TryFrom<T>>(
        self,
    ) -> Result<Chunk<U>, U::Error> {
        Ok(Chunk::<U> {
            begin: self.begin.try_into()?,
            end: self.end.try_into()?,
        })
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
