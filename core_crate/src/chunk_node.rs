use crate::chunk::*;
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
