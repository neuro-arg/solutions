use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    mem::swap,
    ops::{AddAssign, Div, DivAssign, Mul, MulAssign, Sub},
};

use num::{Integer, Num};

pub fn apply_num_op<N: Num, W: NumOp<N>>(num: N, mut op: W) -> N {
    op.apply(num)
}

pub trait NumOp<N> {
    fn apply(&mut self, num: N) -> N;
    fn chain(self, other: impl 'static + NumOp<N>) -> Chain<N>
    where
        Self: 'static + Sized,
    {
        Chain(vec![Box::new(self), Box::new(other)])
    }
}

impl<N> NumOp<N> for Noop {
    fn apply(&mut self, num: N) -> N {
        num
    }
}

pub struct Multiply<N>(N);

impl<N: Clone + Num + Mul> NumOp<N> for Multiply<N> {
    fn apply(&mut self, num: N) -> N {
        num * self.0.clone()
    }
}

pub fn multiply<N: Clone + Num + Mul>(x: N) -> Multiply<N> {
    Multiply(x)
}

pub struct DivideBy<N>(N);

impl<N: Clone + Num + Div> NumOp<N> for DivideBy<N> {
    fn apply(&mut self, num: N) -> N {
        num / self.0.clone()
    }
}

pub fn divide_by<N: Clone + Num + Div>(x: N) -> DivideBy<N> {
    DivideBy(x)
}

pub struct Add<N>(N);

impl<N: Clone + Num + std::ops::Add> NumOp<N> for Add<N> {
    fn apply(&mut self, num: N) -> N {
        num + self.0.clone()
    }
}

pub fn add<N: Clone + Num + std::ops::Add>(x: N) -> Add<N> {
    Add(x)
}

pub struct SubtractBy<N>(N);

impl<N: Clone + Num + Sub> NumOp<N> for SubtractBy<N> {
    fn apply(&mut self, num: N) -> N {
        num - self.0.clone()
    }
}

pub fn subtract_by<N: Clone + Num + Sub>(x: N) -> SubtractBy<N> {
    SubtractBy(x)
}

pub struct Chain<N>(Vec<Box<dyn NumOp<N>>>);
impl<N: Num> NumOp<N> for Chain<N> {
    fn apply(&mut self, mut num: N) -> N {
        for op in &mut self.0 {
            num = op.apply(num);
        }
        num
    }
    fn chain(mut self, other: impl 'static + NumOp<N>) -> Chain<N>
    where
        Self: 'static + Sized,
    {
        self.0.push(Box::new(other));
        self
    }
}

pub struct ConcatDigits<N> {
    base: u8,
    digits: N,
    after: bool,
}

impl<N: Clone + DivAssign + From<u8> + Integer + MulAssign> NumOp<N> for ConcatDigits<N> {
    fn apply(&mut self, mut b: N) -> N {
        let mut a = self.digits.clone();
        if self.after {
            swap(&mut a, &mut b);
        }
        assert!(!a.is_zero());
        let base = N::from(self.base);
        a *= base.clone();
        let mut pow = b.clone();
        while pow >= base {
            a *= base.clone();
            pow /= base.clone();
        }
        a + b
    }
}

pub struct FlipDigits {
    base: u8,
}

impl<N: Clone + From<u8> + Integer + AddAssign + MulAssign> NumOp<N> for FlipDigits {
    fn apply(&mut self, mut num: N) -> N {
        let mut ret = N::from(0u8);
        let base = N::from(self.base);
        while !num.is_zero() {
            ret *= base.clone();
            let (div, rem) = num::integer::div_rem(num, base.clone());
            num = div;
            ret += rem;
        }
        ret
    }
}

pub struct ReplaceDigits {
    base: u8,
    a: Vec<u8>,
    b: Vec<u8>,
    ignore_order: bool,
}

impl<N: Clone + From<u8> + Hash + Integer + AddAssign + MulAssign> NumOp<N> for ReplaceDigits {
    fn apply(&mut self, num: N) -> N {
        let mut ret = N::from(0u8);
        let base = N::from(self.base);
        let a_deq = self.a.iter().copied().map(N::from).collect::<VecDeque<N>>();
        let a_set = self.ignore_order.then(|| {
            let mut map = HashMap::new();
            for val in a_deq.iter() {
                *map.entry(val.clone()).or_default() += 1usize;
            }
            for digit in 0..self.base {
                map.entry(N::from(digit)).or_default();
            }
            map
        });
        let b = self.b.iter().copied().map(N::from).collect::<VecDeque<N>>();
        let mut c_deq = VecDeque::<N>::new();
        let mut c_set = self.ignore_order.then(|| {
            (0..self.base)
                .map(|x| (N::from(x), 0usize))
                .collect::<HashMap<N, usize>>()
        });
        let mut num = FlipDigits { base: self.base }.apply(num);
        let mut first = true;
        while first || !num.is_zero() {
            first = false;
            let (div, rem) = num::integer::div_rem(num, base.clone());
            if let Some(cnt) = c_set.as_mut().and_then(|x| x.get_mut(&rem)) {
                *cnt += 1;
            }
            c_deq.push_back(rem);
            if c_deq.len() > a_deq.len() {
                if let Some(digit) = c_deq.pop_front() {
                    if let Some(cnt) = c_set.as_mut().and_then(|x| x.get_mut(&digit)) {
                        *cnt -= 1;
                    }
                    ret *= base.clone();
                    ret += digit;
                }
            }
            if !a_deq.is_empty()
                && if self.ignore_order {
                    a_deq.len() == c_deq.len() && a_set == c_set
                } else {
                    a_deq == c_deq
                }
            {
                c_deq = b.clone();
                if let Some(c_set) = &mut c_set {
                    c_set.clear();
                    for val in c_deq.iter() {
                        *c_set.entry(val.clone()).or_default() += 1usize;
                    }
                    for digit in 0..self.base {
                        c_set.entry(N::from(digit)).or_default();
                    }
                }
            }
            num = div;
        }
        for digit in c_deq {
            ret *= base.clone();
            ret += digit;
        }
        ret
    }
}

pub mod decimal {
    use super::*;

    pub fn append_digits<N>(digits: N) -> ConcatDigits<N> {
        ConcatDigits {
            base: 10,
            digits,
            after: true,
        }
    }

    pub fn prepend_digits<N>(digits: N) -> ConcatDigits<N> {
        ConcatDigits {
            base: 10,
            digits,
            after: false,
        }
    }

    pub fn flip_digits() -> FlipDigits {
        FlipDigits { base: 10 }
    }

    pub fn replace_digit(a: u8, b: u8) -> ReplaceDigits {
        ReplaceDigits {
            base: 10,
            a: vec![a],
            b: vec![b],
            ignore_order: false,
        }
    }
}

pub use decimal::*;

use crate::Noop;

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        use super::*;
        for x in 1..=100 {
            for y in 0..=100 {
                assert_eq!(
                    prepend_digits(x).apply(y).to_string(),
                    x.to_string() + &y.to_string()
                );
                assert_eq!(
                    append_digits(y).apply(x).to_string(),
                    x.to_string() + &y.to_string()
                );
                assert_eq!(multiply(x).apply(y), y * x);
                assert_eq!(divide_by(x).apply(y), y / x);
                assert_eq!(add(x).apply(y), y + x);
                assert_eq!(subtract_by(x).apply(y), y - x);
            }
        }
        assert_eq!(flip_digits().apply(12345), 54321);
        assert_eq!(replace_digit(1, 2).apply(12345), 22345);
    }
}
