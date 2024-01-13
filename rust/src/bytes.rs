use std::collections::VecDeque;

use aes::{
    cipher::{BlockDecrypt, KeyInit},
    Aes128, Aes192, Aes256,
};
use base64_crate::Engine;

use crate::Noop;

pub fn process_string<X: AsRef<[u8]>, W: Cipher>(data: X, mut cipher: W) -> String {
    String::from_utf8(cipher.process(data.as_ref())).unwrap()
}

pub trait Cipher {
    fn process(&mut self, data: &[u8]) -> Vec<u8>;
    fn chain(self, other: impl 'static + Cipher) -> Chain
    where
        Self: 'static + Sized,
    {
        Chain(vec![Box::new(self), Box::new(other)])
    }
}

impl Cipher for Noop {
    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }
}

impl<T: for<'a> FnMut(&'a [u8]) -> Vec<u8>> Cipher for T {
    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        self(data)
    }
}

pub struct Chain(Vec<Box<dyn Cipher>>);
impl Cipher for Chain {
    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut data = data.to_vec();
        for cipher in &mut self.0 {
            data = cipher.process(&data);
        }
        data
    }
    fn chain(mut self, other: impl 'static + Cipher) -> Chain
    where
        Self: 'static + Sized,
    {
        self.0.push(Box::new(other));
        self
    }
}

pub fn decrypt1<C: BlockDecrypt>(crypt: C, data: &mut [u8]) {
    assert_eq!(data.len() % 16, 0);
    for chunk in data.chunks_mut(16) {
        crypt.decrypt_block(chunk.into());
    }
}

pub mod pkcs7 {
    pub fn unpad(data: &mut Vec<u8>) {
        if let Some(count) = data.last().copied() {
            if count == 0
                || data[data.len() - usize::from(count)..]
                    .iter()
                    .copied()
                    .any(|x| x != count)
            {
                panic!("invalid pkcs7 padding");
            }
            data.truncate(data.len() - usize::from(count));
        }
    }
}

pub fn decrypt_aes(data: &[u8], key: &[u8]) -> Vec<u8> {
    let mut data = data.to_vec();
    eprintln!("{:?}", String::from_utf8(key.to_vec()));
    match key.len() * 8 {
        128 => decrypt1(Aes128::new_from_slice(key).unwrap(), &mut data),
        192 => decrypt1(Aes192::new_from_slice(key).unwrap(), &mut data),
        256 => decrypt1(Aes256::new_from_slice(key).unwrap(), &mut data),
        _ => panic!("invalid aes key len: {} (key: {:?})", key.len() * 8, key),
    }
    let padded_len = data.len();
    pkcs7::unpad(&mut data);
    assert!((1..=16).contains(&(padded_len - data.len())));
    data
}

pub fn decrypt_data<D: AsRef<[u8]>>(data: D) -> DecryptData {
    DecryptData(data.as_ref().to_vec())
}
pub struct DecryptData(Vec<u8>);
impl Cipher for DecryptData {
    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        decrypt_aes(&self.0, data)
    }
}

pub fn decrypt_with<K: AsRef<[u8]>>(key: K) -> DecryptDataWith {
    DecryptDataWith(key.as_ref().to_vec())
}
pub struct DecryptDataWith(Vec<u8>);
impl Cipher for DecryptDataWith {
    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        decrypt_aes(data, &self.0)
    }
}

pub struct Base64Dec;
impl Cipher for Base64Dec {
    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let data = data
            .iter()
            .copied()
            .filter(|x| x.is_ascii_alphanumeric() || matches!(x, b'=' | b'/' | b'+' | b'_' | b'-'))
            .map(|x| match x {
                b'_' => b'/',
                b'-' => b'+',
                x => x,
            })
            .collect::<Vec<_>>();
        base64_crate::engine::general_purpose::STANDARD
            .decode(data)
            .expect("invalid base64")
    }
}

pub fn base64() -> Base64Dec {
    Base64Dec
}

const POW3: [u16; 11] = [1, 3, 9, 27, 81, 243, 729, 2187, 6561, 19683, 59049];

fn crazy(mut a: u16, mut b: u16) -> u16 {
    let mut ret = 0;
    for pow in &POW3[..10] {
        let val = match (a % 3, b % 3) {
            (3.., _) | (_, 3..) => unreachable!(),
            (0, 1 | 2) | (1, 1) => 0,
            (0 | 1, 0) | (2, 2) => 1,
            (1, 2) | (2, 0 | 1) => 2,
        };
        ret += pow * val;
        a /= 3;
        b /= 3;
    }
    ret
}

pub struct Malbolge(VecDeque<u8>);
impl Cipher for Malbolge {
    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut ret = vec![];
        let mut mem = vec![0u16; 3usize.pow(10)];
        let mut filled = 0usize;
        for (mem, byte) in mem
            .iter_mut()
            .zip(data.iter().copied().filter(|x| !x.is_ascii_whitespace()))
        {
            let opcode = (filled + usize::from(byte)) % 94;
            filled += 1;
            if matches!(opcode, 4 | 5 | 23 | 39 | 40 | 62 | 68 | 81) {
                *mem = byte.into();
            } else {
                panic!(
                    "invalid malbolge opcode: {opcode}/{:?}",
                    char::from_u32(opcode as u32)
                );
            }
        }
        {
            let (mut a, mut b) = (mem[filled - 2], mem[filled - 1]);
            for target in &mut mem[filled..] {
                a = crazy(a, b);
                *target = a;
                std::mem::swap(&mut a, &mut b);
            }
        }
        let mut a = 0u16;
        let mut c = 0usize;
        let mut d = 0usize;
        while (33..128).contains(&mem[c]) {
            match (c + usize::from(mem[c])) % 94 {
                4 => c = mem[d].into(),
                5 => ret.push(a as u8),
                23 => a = self.0.pop_front().unwrap().into(),
                39 => {
                    a = mem[d] % 3 * POW3[9] + mem[d] / 3;
                    mem[d] = a;
                }
                40 => d = mem[d].into(),
                62 => {
                    a = crazy(mem[d], a);
                    mem[d] = a;
                }
                81 => break,
                _ => {}
            }
            mem[c] %= 94;
            mem[c] = b"9m<.TVac`uY*MK'X~xDl}REokN:#?G\"i@5z]&gqtyfr$(we4{WP)H-Zn,[%\\3dL+Q;>U!pJS72FhOA1CB6v^=I_0/8|jsb"[usize::from(mem[c])].into();
            c += 1;
            d += 1;
            c %= usize::from(POW3[10]);
            d %= usize::from(POW3[10]);
        }
        ret
    }
}

pub fn malbolge() -> Malbolge {
    Malbolge(VecDeque::new())
}

pub struct FuzzyReplace(Vec<u8>, Vec<u8>, bool);

impl Cipher for FuzzyReplace {
    fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut ret = vec![];
        let mut target0 = [0usize; 256];
        for x in self.0.iter().copied() {
            target0[usize::from(x)] = target0[usize::from(x)].checked_add(1).unwrap();
        }
        let mut target = target0;
        let mut buf = VecDeque::new();
        let mut done = false;
        for val in data.iter().copied() {
            target[usize::from(val)] = target[usize::from(val)].wrapping_sub(1);
            buf.push_back(val);
            if buf.len() > self.0.len() {
                let val = buf.pop_front().unwrap();
                target[usize::from(val)] = target[usize::from(val)].wrapping_add(1);
                ret.push(val);
            }
            if !done && buf.len() == self.0.len() && target == [0usize; 256] {
                buf.clear();
                buf.extend(self.1.iter().copied());
                target = target0;
                if self.2 {
                    done = true;
                }
            }
        }
        if self.2 && !done {
            panic!("fuzzy match not found for single replace");
        }
        let (a, b) = buf.as_slices();
        ret.extend_from_slice(a);
        ret.extend_from_slice(b);
        ret
    }
}

pub fn fuzzy_replace(a: &[u8], b: &[u8], single: bool) -> FuzzyReplace {
    FuzzyReplace(a.to_vec(), b.to_vec(), single)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test() {
        assert_eq!(
            crazy(
                POW3[6] + POW3[5] + POW3[4] + 2 * POW3[3] + 2 * POW3[2] + 2 * POW3[1],
                POW3[8] + 2 * POW3[7] + POW3[5] + 2 * POW3[4] + POW3[2] + 2 * POW3[1]
            ),
            POW3[9] + POW3[6] + 2 * POW3[4] + 2 * POW3[3] + 2 * POW3[2] + POW3[1] + POW3[0],
        );
        assert_eq!(malbolge().process(b"(=<`#9]~6ZY327Uv4-QsqpMn&+Ij\"'E%e{Ab~w=_:]Kw%o44Uqp0/Q?xNvL:`H%c#DD2^WV>gY;dts76qKJImZkj"), b"Hello, world.");
        assert_eq!(
            fuzzy_replace(b"12345", b"abc", true).process(b"b42315"),
            b"babc"
        );
    }
}
