#[cfg(test)]
pub mod tests {
    use crate::core::IntervalList;

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
        list.add_chunk(Chunk::new(0, 11).unwrap());
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

    #[cfg(test)]
    mod tests_interval {
        use super::*;

        #[test]
        fn test_complement_intervals() {
            let mut list = IntervalList::new();

            list.add_chunk((10, 20).try_into().unwrap());
            list.add_chunk((40, 50).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((0, 100).try_into().unwrap())
                .unwrap();

            println!("{}", comp_list);
        }

        #[test]
        fn test_empty_interval_list() {
            let list = IntervalList::new();
            let range = (0, 100).try_into().unwrap();

            let comp_list = list.get_complement_intervals(range).unwrap();

            let expected =
                IntervalList::from_intervals(vec![(0, 100).try_into().unwrap()]).unwrap();
            assert_eq!(
                comp_list, expected,
                "Empty interval list should return the full range"
            );
        }

        #[test]
        fn test_single_interval() {
            let mut list = IntervalList::new();
            list.add_chunk((10, 20).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((0, 30).try_into().unwrap())
                .unwrap();

            let expected_chunks = vec![(0, 10).try_into().unwrap(), (20, 30).try_into().unwrap()];
            let expected = IntervalList::from_intervals(expected_chunks).unwrap();
            assert_eq!(
                comp_list, expected,
                "Complement should include ranges before and after interval"
            );
        }

        #[test]
        fn test_multiple_intervals() {
            let mut list = IntervalList::new();
            list.add_chunk((10, 20).try_into().unwrap());
            list.add_chunk((40, 50).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((0, 100).try_into().unwrap())
                .unwrap();

            let expected_chunks = vec![
                (0, 10).try_into().unwrap(),
                (20, 40).try_into().unwrap(),
                (50, 100).try_into().unwrap(),
            ];
            let expected = IntervalList::from_intervals(expected_chunks).unwrap();
            assert_eq!(
                comp_list, expected,
                "Complement should include all gaps between intervals"
            );
        }

        #[test]
        fn test_interval_at_start() {
            let mut list = IntervalList::new();
            list.add_chunk((0, 10).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((0, 20).try_into().unwrap())
                .unwrap();

            let expected =
                IntervalList::from_intervals(vec![(10, 20).try_into().unwrap()]).unwrap();
            assert_eq!(
                comp_list, expected,
                "Complement should handle interval at start of range"
            );
        }

        #[test]
        fn test_interval_at_end() {
            let mut list = IntervalList::new();
            list.add_chunk((90, 100).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((0, 100).try_into().unwrap())
                .unwrap();

            let expected = IntervalList::from_intervals(vec![(0, 90).try_into().unwrap()]).unwrap();
            assert_eq!(
                comp_list, expected,
                "Complement should handle interval at end of range"
            );
        }

        #[test]
        fn test_adjacent_intervals() {
            let mut list = IntervalList::new();
            list.add_chunk((10, 20).try_into().unwrap());
            list.add_chunk((20, 30).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((0, 40).try_into().unwrap())
                .unwrap();

            let expected_chunks = vec![(0, 10).try_into().unwrap(), (30, 40).try_into().unwrap()];
            let expected = IntervalList::from_intervals(expected_chunks).unwrap();
            assert_eq!(
                comp_list, expected,
                "Complement should handle adjacent intervals"
            );
        }

        #[test]
        fn test_interval_covering_full_range() {
            let mut list = IntervalList::new();
            list.add_chunk((0, 100).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((0, 100).try_into().unwrap())
                .unwrap();

            let expected = IntervalList::new();
            assert_eq!(
                comp_list, expected,
                "Complement should be empty when interval covers full range"
            );
        }

        #[test]
        fn test_interval_exceeding_range() {
            let mut list = IntervalList::new();
            list.add_chunk((-10, 110).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((0, 100).try_into().unwrap())
                .unwrap();

            let expected = IntervalList::new();
            assert_eq!(
                comp_list, expected,
                "Complement should be empty when interval exceeds range"
            );
        }

        #[test]
        fn test_exact_interval_bounds() {
            let mut list = IntervalList::new();
            list.add_chunk((10, 20).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((10, 20).try_into().unwrap())
                .unwrap();

            let expected = IntervalList::new();
            assert_eq!(
                comp_list, expected,
                "Complement should be empty when interval exactly matches range"
            );
        }

        #[test]
        fn test_overlapping_intervals() {
            let mut list = IntervalList::new();
            list.add_chunk((10, 30).try_into().unwrap());
            list.add_chunk((20, 40).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((0, 50).try_into().unwrap())
                .unwrap();

            let expected_chunks = vec![(0, 10).try_into().unwrap(), (40, 50).try_into().unwrap()];
            let expected = IntervalList::from_intervals(expected_chunks).unwrap();
            assert_eq!(
                comp_list, expected,
                "Complement should handle overlapping intervals correctly"
            );
        }

        #[test]
        fn last_test() {
            let mut list = IntervalList::new();
            list.add_chunk((10, 27).try_into().unwrap());

            let comp_list = list
                .get_complement_intervals((0, 100).try_into().unwrap())
                .unwrap();

            let expected = IntervalList::from_intervals(vec![
                (0, 10).try_into().unwrap(),
                (27, 100).try_into().unwrap(),
            ])
            .unwrap();

            assert_eq!(comp_list, expected);
        }

        #[test]
        fn test_bug() {
            let mut list = IntervalList::new();
            list.add_chunk((10, 20).try_into().unwrap()).unwrap();

            list.add_chunk((0, 10).try_into().unwrap()).unwrap();

            assert_eq!(list.len(), 1);
        }
    }
}
