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

pub fn kmp_find_prefix_with_lsp_table<N, H>(needle: &[N], haystack: &[H], wild_card_size: usize,  lsp: &[usize]) -> Option<(usize, usize)>
where
    N: PartialEq + fmt::Debug,
    H: PartialEq<N> + fmt::Debug,
{
    let mut needle_pos: usize = 0;
    let mut best_index= None;

    let start = haystack.len().saturating_sub(needle.len());
    let end = (start + wild_card_size).min(haystack.len());
    for (haystack_pos, haystack_char) in haystack[start..end].iter().enumerate() {

        while needle_pos > 0 && *haystack_char != needle[needle_pos] {
            // character mismatch, move backwards in the needle
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

    if let Some(best_index) = best_index {
        if (best_index + needle.len()) > (haystack.len() + wild_card_size) {
            None
        } else {
            Some((best_index + start, needle_pos))
        }
    } else {
        if needle.len() <= wild_card_size {
            Some((end, 0))
        } else {
            None
        }
    }
}
