// two small combinator libraries for processing data and numbers
pub mod bytes;
pub mod data;
pub mod images;
pub mod math;
pub mod numbers;

pub struct Noop;

fn split_off_last_word(s: &mut String) -> String {
    let pos = s
        .bytes()
        .enumerate()
        .rev()
        .find(|(_, x)| x.is_ascii_whitespace())
        .unwrap()
        .0;
    let ret = s.split_off(pos + 1);
    s.pop();
    ret
}

pub mod numbers1 {
    use std::collections::HashMap;

    use crate::{bytes::*, data, numbers::*};

    pub const NUMBER: u128 = 572943;
    pub fn calc(num: u128) -> String {
        let ret = apply_num_op(
            num,
            prepend_digits(2u128)
                .chain(append_digits(9))
                .chain(append_digits(1))
                .chain(multiply(5))
                .chain(append_digits(6))
                .chain(flip_digits())
                .chain(replace_digit(2, 3))
                .chain(multiply(9))
                .chain(append_digits(24))
                .chain(prepend_digits(17)),
        );
        let mut map = HashMap::new();
        for (a, b) in num
            .to_string()
            .chars()
            .zip(std::iter::successors(Some('a'), |x| {
                char::from_u32(*x as u32 + 1)
            }))
        {
            map.entry(a).or_insert(b);
        }
        ret.to_string()
            .chars()
            .map(|x| map.get(&x).copied().unwrap_or(x))
            .collect()
    }

    pub fn answer() -> String {
        process_string(
            data::NUMBERS_BASE64,
            base64().chain(decrypt_with(calc(NUMBER))),
        )
    }
}

pub mod study {
    use crate::{bytes::*, data, numbers1};

    pub fn phone() -> String {
        process_string(
            data::STUDY_BASE64,
            base64().chain(decrypt_with(numbers1::calc(numbers1::NUMBER))),
        )
    }

    pub fn malbolge_code() -> String {
        process_string(data::STUDY_MALBOLGE, malbolge())
    }
}

pub mod numbers2 {
    use crate::{bytes::*, data, math, split_off_last_word, Noop};

    pub fn stage1() -> (String, String) {
        let mut ret = process_string(
            data::NUMBERS2_BASE64,
            base64().chain(decrypt_with(data::NUMBERS2_NUEROS)),
        );
        let cipher = split_off_last_word(&mut ret);
        (ret, cipher)
    }

    pub fn stage2() -> (String, String) {
        let key = &process_string(
            math::FibonacciDigits::<num::BigUint>::new()
                .take(500)
                .collect::<String>()
                .as_bytes(),
            {
                let mut op = Cipher::chain(Noop, Noop);
                for nums in data::NUMBERS2 {
                    let digits: String = nums.map(|x| x.to_string()).into_iter().collect();
                    op = op.chain(fuzzy_replace(digits.as_bytes(), &[], true));
                }
                op
            },
        )[..16];
        let mut ret = process_string(stage1().1, base64().chain(decrypt_with(key)));
        let cipher = split_off_last_word(&mut ret);
        (ret, cipher)
    }

    pub fn stage3() -> String {
        process_string(stage2().1.as_bytes(), malbolge())
    }

    pub fn all_stages() -> (String, String, String) {
        (stage1().0, stage2().0, stage3())
    }
}

pub mod soundcloud {
    use crate::{bytes::*, data};

    pub fn answer() -> String {
        let key: String = data::SOUNDCLOUD_IMAGE
            .split('Y')
            .next()
            .unwrap()
            .split('X')
            .last()
            .unwrap()
            .chars()
            .filter(|x| !x.is_ascii_digit())
            .collect();
        process_string(data::SOUNDCLOUD_BASE64, base64().chain(decrypt_with(key)))
    }
}

pub mod filtered {
    use crate::{bytes::*, data, images};

    #[derive(Clone, Debug)]
    pub struct Shift(String, u32);
    impl From<String> for Shift {
        fn from(value: String) -> Self {
            Self(value, 0)
        }
    }
    impl Shift {
        pub fn new<S: AsRef<str>>(s: S) -> Self {
            Self::from(s.as_ref().to_owned())
        }
    }
    impl Iterator for Shift {
        type Item = String;
        fn next(&mut self) -> Option<Self::Item> {
            let mut valid = false;
            let ret = self
                .0
                .chars()
                .filter_map(|x| {
                    let val = char::from_u32((x as u32).saturating_sub(self.1).max(b' ' as u32));
                    if matches!(val, Some(x) if x != ' ') {
                        valid = true;
                    }
                    val
                })
                .collect();
            self.1 += 1;
            valid.then_some(ret)
        }
    }
    impl std::iter::FusedIterator for Shift {}

    pub fn denoise_image() {
        images::filtered_denoise("noise.png", "out.png");
    }

    pub fn answer() -> String {
        let code =
            Shift::new(data::FILTERED_TITLE).nth(784).unwrap() + data::FILTERED_GUESSED_CODE_END;
        assert_eq!(malbolge().process(code.as_bytes()), b"hello world!");
        process_string(
            data::FILTERED_BASE64,
            base64().chain(decrypt_with(code[code.len() - 16..].as_bytes())),
        )
    }
}

/// KEY = 128bit
mod meaning_of_life {
    use crate::{bytes::*, data, images};

    pub fn hex() -> String {
        data::MEANING_OF_LIFE_BINARY
            .split_whitespace()
            .map(|x| u8::from_str_radix(x, 2).unwrap() as char)
            .collect()
    }

    pub fn hex_to_bytes(s: &str) -> Vec<u8> {
        s.as_bytes()
            .chunks_exact(2)
            .map(|x| std::str::from_utf8(x).unwrap())
            .map(|x| u8::from_str_radix(x, 16).unwrap())
            .collect()
    }

    #[allow(unused)]
    pub fn reassemble_image() {
        images::meaning_of_life_reassemble();
    }

    #[allow(unused)]
    pub fn answer() -> Vec<u8> {
        let cipher = data::MEANING_OF_LIFE_BASE64;
        let hex = hex();
        let hex_bytes = hex_to_bytes(&hex);
        let num = data::MEANING_OF_LIFE_NUM;
        let str = data::MEANING_OF_LIFE_STR;
        panic!(
            "{:?}",
            base64()
                .chain(decrypt_with(&hex_bytes[16..]))
                .process(cipher.as_bytes())
        )
    }
}

fn main() {
    filtered::denoise_image();
    // println!("{:?}", meaning_of_life::answer());
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test() {
        numbers1::answer();
        study::phone();
        study::malbolge_code();
        numbers2::all_stages();
        soundcloud::answer();
        filtered::answer();
    }
}
