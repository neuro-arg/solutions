use std::ops::AddAssign;

use num::{Integer, Num};

pub struct Fibonacci<N: Num>(N, N, bool);

impl<N: Num> Default for Fibonacci<N> {
    fn default() -> Self {
        Self(N::one(), N::zero(), true)
    }
}

impl<N: Num> Fibonacci<N> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<N: AddAssign + Clone + Num> Iterator for Fibonacci<N> {
    type Item = N;
    fn next(&mut self) -> Option<Self::Item> {
        if self.2 {
            self.2 = false;
            return Some(self.1.clone());
        }
        self.0 += self.1.clone();
        std::mem::swap(&mut self.0, &mut self.1);
        Some(self.1.clone())
    }
}

pub struct FibonacciDigits<N: Num>(Fibonacci<N>, String);

impl<N: Num> Default for FibonacciDigits<N> {
    fn default() -> Self {
        Self(Fibonacci::default(), String::new())
    }
}

impl<N: Num> FibonacciDigits<N> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<N: AddAssign + Clone + From<u8> + Integer + ToString> Iterator for FibonacciDigits<N> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(char) = self.1.pop() {
                return Some(char);
            }
            if let Some(mut val) = self.0.next() {
                if val.is_zero() {
                    return Some('0');
                }
                let ten = N::from(10u8);
                while !val.is_zero() {
                    let (div, rem) = num::integer::div_rem(val, ten.clone());
                    val = div;
                    self.1.push_str(&rem.to_string());
                }
            } else {
                return None;
            }
        }
    }
}

pub fn digits<N: Clone + From<u8> + Integer + TryInto<u8>>(mut val: N) -> Vec<u8> {
    if val.is_zero() {
        return vec![0];
    }
    let ten = N::from(10u8);
    let mut ret = vec![];
    while !val.is_zero() {
        let (div, rem) = num::integer::div_rem(val, ten.clone());
        val = div;
        ret.push(rem.try_into().map_err(|_| ()).unwrap());
    }
    ret.reverse();
    ret
}
