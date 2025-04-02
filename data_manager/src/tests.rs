use crate::Chunk;
use core_crate::core::IntervalList;

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_empty_list() {
        let list = IntervalList::<u8>::new();
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
    fn test_near() {
        let mut list = IntervalList::<usize>::new();
        list.add_chunk(Chunk::new(0, 10).unwrap());
        list.add_chunk(Chunk::new(11, 20).unwrap());

        assert_eq!(list.len(), 1);
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

    #[test]
    fn test_adding_overlaping_chunks() {
        let mut list = IntervalList::new();

        assert!(list.add_chunk((10, 80).try_into().unwrap()).is_ok());
        assert!(list.add_chunk((90, 110).try_into().unwrap()).is_ok());
        assert!(list.add_chunk((140, 150).try_into().unwrap()).is_ok());
        assert!(list.add_chunk((60, 120).try_into().unwrap()).is_ok());

        list.iter().for_each(|el| {
            println!("{}", el);
        });

        assert!(list.contains(85));
        assert_eq!(list.len(), 2);

        assert!(list.add_chunk((160, 180).try_into().unwrap()).is_ok());
        assert!(list.add_chunk((40, 155).try_into().unwrap()).is_ok());

        println!("BIG HUI");
        list.iter().for_each(|el| {
            println!("{}", el);
        });

        assert!(list.contains(130));
        assert_eq!(list.len(), 2);
    }
}
