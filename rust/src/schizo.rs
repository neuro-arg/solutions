use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use num::{BigUint, Num, Zero};

// experimental/potentially useless algorithms go here

pub fn hex_to_bytes(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|x| std::str::from_utf8(x).unwrap())
        .map(|x| u8::from_str_radix(x, 16).unwrap())
        .collect()
}

#[derive(Clone, Debug)]
pub struct RevShift(String, u32);
impl From<String> for RevShift {
    fn from(value: String) -> Self {
        Self(value, 0)
    }
}
impl RevShift {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self::from(s.as_ref().to_owned())
    }
}
impl Iterator for RevShift {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self
            .0
            .chars()
            .filter_map(|x| {
                if x == ' ' {
                    Some(x)
                } else {
                    char::from_u32((x as u32 + self.1).max(b' ' as u32))
                }
            })
            .collect();
        self.1 += 1;
        Some(ret)
    }
}
impl std::iter::FusedIterator for RevShift {}

// for meaning of life...
pub fn reverse_numbers(s: &str) -> Vec<BigUint> {
    let set = HashSet::<char>::from_iter(s.chars());
    // sanity check: up to 10 digits
    assert!(set.len() <= 10);
    // sanity check: all are valid hex digits
    assert!(set.iter().copied().all(|x| x.is_ascii_hexdigit()));
    // sanity check: the start is 17 or 24 (may be partially replaced with hex)
    let mut map = (0..10).map(|i| (b'0' + i, i)).collect::<HashMap<_, _>>();
    let mut s = s.to_owned();
    assert!(s[..2]
        .to_owned()
        .bytes()
        .chain(s[s.len() - 2..].to_owned().bytes())
        .zip([1, 7, 2, 4])
        .all(|(x, y)| {
            if let Some(w) = map.get(&x) {
                *w == y
            } else {
                s = s.replace(x as char, std::str::from_utf8(&[b'0' + y]).unwrap());
                map.insert(x, y);
                true
            }
        }));
    let set = HashSet::<char>::from_iter(s.chars());
    // missing decimal digits
    let missing_digits: Vec<_> = (0u8..10u8)
        .filter(|x| !set.contains(&((b'0' + x) as char)))
        .collect();
    // all hex digits
    let hex: Vec<_> = set
        .iter()
        .filter(|x| x.is_ascii_alphabetic())
        .map(|x| *x as u8 - b'a' + 10)
        .collect();
    // currently this function isn't designed for other cases
    assert_eq!(missing_digits.len(), hex.len());
    let mut ret = vec![];
    for ks in hex.iter().copied().permutations(hex.len()) {
        let mut s = s.clone();
        for (k, v) in ks.into_iter().zip(missing_digits.iter().copied()) {
            s = s.replace(
                (k - 10 + b'a') as char,
                std::str::from_utf8(&[v + b'0']).unwrap(),
            );
        }
        if let Some(s) = s
            .strip_prefix("17")
            .and_then(|s| s.strip_suffix("24"))
            .and_then(|s| {
                let b = BigUint::from_str_radix(s, 10).unwrap();
                let (div, rem) = num::integer::div_rem(b, 9u8.into());
                rem.is_zero().then_some(div)
            })
            .map(|num| num.to_string().chars().rev().collect::<String>())
            .and_then(|s| s.strip_suffix('6').map(ToOwned::to_owned))
        {
            // any 3 may or may not come from a 2
            for s in s
                .chars()
                .map(|x| match x {
                    '3' => Some('2').into_iter().chain(Some('3')),
                    '2' => None.into_iter().chain(None),
                    x => Some(x).into_iter().chain(None),
                })
                .multi_cartesian_product()
                .map(|x| x.into_iter().collect::<String>())
            {
                let b = BigUint::from_str_radix(&s, 10).unwrap();
                let (div, rem) = num::integer::div_rem(b, 5u8.into());
                if rem.is_zero() {
                    if let Some(s) = div
                        .to_string()
                        .strip_suffix("91")
                        .and_then(|s| s.strip_prefix('2'))
                    {
                        ret.push(BigUint::from_str_radix(s, 10).unwrap());
                    }
                }
            }
        }
    }
    ret
}
