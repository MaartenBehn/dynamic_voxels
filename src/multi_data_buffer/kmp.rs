use std::fmt;


pub fn kmp_table<N>(needle: &[N]) -> Vec<usize>
where
    N: PartialEq,
{
    let mut lsp = Vec::with_capacity(needle.len());
    lsp.push(0);

    for needle_char in &needle[1..] {
        let mut distance: usize = *lsp.last().unwrap();

        while distance > 0 && *needle_char != needle[distance] {
            distance = lsp[distance - 1];
        }

        if *needle_char == needle[distance] {
            distance += 1;
        }

        lsp.push(distance);
    }

    lsp
}

pub fn kmp_find<N, H>(needle: &[N], haystack: &[H]) -> Option<usize>
where
    N: PartialEq,
    H: PartialEq<N>,
{
    kmp_find_with_lsp_table(needle, haystack, &kmp_table(needle))
}

pub fn kmp_find_with_lsp_table<N, H>(needle: &[N], haystack: &[H], lsp: &[usize]) -> Option<usize>
where
    N: PartialEq,
    H: PartialEq<N>,
{
    let mut needle_pos: usize = 0;
    for (haystack_pos, haystack_char) in haystack.iter().enumerate() {

        while needle_pos > 0 && *haystack_char != needle[needle_pos] {
            // character mismatch, move backwards in the needle
            needle_pos = lsp[needle_pos - 1];
        }

        if *haystack_char == needle[needle_pos] {
            // char matches, move to next needle character
            needle_pos += 1;

            if needle_pos == needle.len() {
                // we found all needle characters in the haystack, return position in haystack
                return Some(haystack_pos - (needle_pos - 1));
            }
        }
    }

    None
}

pub fn kmp_find_prefix<N, H>(needle: &[N], haystack: &[H], wild_card_size: usize) -> Option<(usize, usize)>
where
    N: PartialEq + fmt::Debug,
    H: PartialEq<N> + fmt::Debug,
{
    kmp_find_prefix_with_lsp_table(needle, haystack, wild_card_size,&kmp_table(needle))
}

pub fn kmp_find_prefix_with_lsp_table<N, H>(needle: &[N], haystack: &[H], wild_card_size: usize,  lsp: &[usize]) -> Option<(usize, usize)>
where
    N: PartialEq + fmt::Debug,
    H: PartialEq<N> + fmt::Debug,
{
    let mut needle_pos: usize = 0;
    let mut best_index = None;

    let start = haystack.len().saturating_sub(needle.len() - 1);
    let wildcard_end = haystack.len() + wild_card_size;
    
    for (haystack_pos, haystack_char) in haystack[start..].iter().enumerate() {

        if (haystack_pos + needle.len() - needle_pos) > wildcard_end {
            return None;
        }  

        while needle_pos > 0 && *haystack_char != needle[needle_pos] {
            needle_pos = lsp[needle_pos - 1];
            best_index = None;
        }

        if *haystack_char == needle[needle_pos] {
            if needle_pos == 0 {
                best_index = Some(haystack_pos);
            }

            needle_pos += 1;
        }
    }

    if best_index.is_none() {
        return None;
    }

    if (needle.len() - needle_pos) > wild_card_size {
        return None;
    }

    Some((best_index.unwrap() + start, needle_pos))
}

#[cfg(test)]

mod tests {
    use proptest::prelude::*;
    use crate::multi_data_buffer::kmp::{kmp_find, kmp_find_prefix};

    #[test]
    fn basic_find() {
        assert_eq!(
            Some(6),
            kmp_find(
                &['a', 'a', 'a', 'b'],
                &['a', 'a', 'a', 'a', 'a', 'a', 'a', 'a', 'a', 'b']
            )
        )
    }

    #[test]
    fn find_empty_haystack() {
        // we have to help with type inference here
        let empty_haystack: &[char; 0] = &[];
        assert_eq!(None, kmp_find(&['a', 'b', 'c'], empty_haystack));

    }

    #[test]
    fn find_needle_longer_haystack() {
        assert_eq!(None, kmp_find(&['a', 'b', 'c'], &['a', 'b']));
    }

    proptest! {
        #[test]
        fn fuzz_find(needle in prop::collection::vec(".*", 1..100), haystack in prop::collection::vec(".*", 0..100)) {
            kmp_find(&needle, &haystack);
        }
    }

    #[test]
    fn basic_prefix() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','l','l','o','3','2'],
                &['l','o','l','h','a','l','l','o'],
                2,
            ),
            Some((3, 5))
        )
    }

    #[test]
    fn failing_prefix() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','a','l','o','3','2'],
                &['l','o','l','h','a','l','l','o'],
                2,
            ),
            None
        )
    }
 
    #[test]
    fn barly_fits_into_wildcard() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','l','l','o'],
                &['l','o','l','h'],
                4,
            ),
            Some((3, 1))
        )
    }

    #[test]
    fn wildcard_to_small() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','l','l','o'],
                &['l','o','l'],
                4,
            ),
            None
        )
    }

    #[test]
    fn wildcard_to_small_overlapping() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','l','l','o'],
                &['l','o','l','h'],
                3,
            ),
            None
        )
    }

    #[test]
    fn will_only_fully_fit() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','l','l','o'],
                &['h','a','l','l','o'],
                10,
            ),
            None
        )
    }

    #[test]
    fn will_only_fully_fit_2() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','l','l','o'],
                &['a', 'b', 'h','a','l','l','o'],
                10,
            ),
            None
        )
    }

    #[test]
    fn fail_2() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','l','l','o'],
                &['a', 'b', 'h','h','h','h','b'],
                10,
            ),
            None
        )
    }

    #[test]
    fn fail_3() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','l','l','o'],
                &['a', 'b', 'h','a','l','l','b'],
                10,
            ),
            None
        )
    }

    #[test]
    fn fits_2() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','l','l','o'],
                &['h', 'a'],
                10,
            ),
            Some((0, 2))
        )
    }

    #[test]
    fn fits_3() {
        assert_eq!(
            kmp_find_prefix(
                &['h','a','l','l','o'],
                &['a','a','h', 'a'],
                10,
            ),
            Some((2, 2))
        )
    }
}
