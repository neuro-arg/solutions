use crate::bytes::{process_string, Cipher};

// two small combinator libraries for processing data and numbers
pub mod bytes;
pub mod data;
pub mod images;
pub mod math;
pub mod numbers;
pub mod schizo;

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

    use num::BigUint;

    use crate::{bytes::*, data, numbers::*};

    pub fn calc(num: BigUint) -> String {
        let ret = apply_num_op(
            num.clone(),
            prepend_digits(BigUint::from(2u8))
                .chain(append_digits(9u8.into()))
                .chain(append_digits(1u8.into()))
                .chain(multiply(5u8.into()))
                .chain(append_digits(6u8.into()))
                .chain(flip_digits())
                .chain(replace_digit(2, 3))
                .chain(multiply(9u8.into()))
                .chain(append_digits(24u8.into()))
                .chain(prepend_digits(17u8.into())),
        );
        let mut map = HashMap::new();
        for (a, b) in num
            .to_string()
            .chars()
            .zip(std::iter::successors(Some('a'), |x| {
                char::from_u32(*x as u32 + 1)
            }))
            .take(6)
        {
            map.insert(a, b);
            // map.entry(a).or_insert(b);
        }
        // println!("{}", ret.clone().to_string());
        ret.to_string()
            .chars()
            .map(|x| map.get(&x).copied().unwrap_or(x))
            .collect()
    }

    pub fn answer() -> String {
        process_string(
            data::NUMBERS_BASE64,
            base64().chain(decrypt_with(calc(data::NUMBER.into()))),
        )
        .unwrap()
    }
}

pub mod study {
    use crate::{bytes::*, data, numbers1};

    pub fn phone() -> String {
        process_string(
            data::STUDY_BASE64,
            base64().chain(decrypt_with(numbers1::calc(data::NUMBER.into()))),
        )
        .unwrap()
    }

    pub fn malbolge_code() -> String {
        process_string(data::STUDY_MALBOLGE, malbolge()).unwrap()
    }
}

pub mod numbers2 {
    use crate::{bytes::*, data, math, split_off_last_word, Noop};

    pub fn stage1() -> (String, String) {
        let mut ret = process_string(
            data::NUMBERS2_BASE64,
            base64().chain(decrypt_with(data::NUMBERS2_NUEROS)),
        )
        .unwrap();
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
        )
        .unwrap()[..16];
        let mut ret = process_string(stage1().1, base64().chain(decrypt_with(key))).unwrap();
        let cipher = split_off_last_word(&mut ret);
        (ret, cipher)
    }

    pub fn stage3() -> String {
        process_string(stage2().1.as_bytes(), malbolge()).unwrap()
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
        process_string(data::SOUNDCLOUD_BASE64, base64().chain(decrypt_with(key))).unwrap()
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
        assert_eq!(
            malbolge().process(code.as_bytes()).unwrap(),
            b"hello world!"
        );
        process_string(
            data::FILTERED_BASE64,
            base64().chain(decrypt_with(code[code.len() - 16..].as_bytes())),
        )
        .unwrap()
    }
}

/// KEY = 128bit
mod meaning_of_life {
    use crate::{bytes::*, data, images, schizo};

    pub fn hex() -> String {
        data::MEANING_OF_LIFE_BINARY
            .split_whitespace()
            .map(|x| u8::from_str_radix(x, 2).unwrap() as char)
            .collect()
    }

    #[allow(unused)]
    pub fn reassemble_image() {
        images::meaning_of_life_reassemble();
    }

    #[allow(unused)]
    pub fn schizo() {
        // !!!
        for num in schizo::reverse_numbers(&hex()) {
            println!("{num}");
        }
    }

    #[allow(unused)]
    pub fn answer() -> Vec<u8> {
        let cipher = data::MEANING_OF_LIFE_BASE64;
        let hex = hex();
        let hex_bytes = schizo::hex_to_bytes(&hex);
        let num = data::MEANING_OF_LIFE_NUM;
        let str = data::MEANING_OF_LIFE_STR;
        /* 692048501258949201
         * 99aa671fce19251
         * 46325147077470311121
         * 692048501258949201
         * c75e0fb05eec877fc8522e550df55ded5b50508fbbe88bb7d82e28d8f2df0e2b
         * 201 1321 5831 723 743 4913 879 875 2145 716 906 4156
         * bb ce td ht eft ggd sgfi dqj ie br vtye b sbs
         * 22 23 83 48 338 443 7434 375 43 27 8893 2 727
         * bbc etd hte ftg gds gfi dqj ieb rvt yeb sbs
         * 34558 6109 419 177
         * 1416938177140W6W13234W0996W8W54119196W770W7949581998XW
         */
        for w in super::filtered::Shift::new("bb ce td ht eft ggd sgfi dqj ie br vtye b sbs") {
            println!("{w}");
        }
        panic!(
            "{:?}",
            base64()
                .chain(decrypt_with("1416938177140585"))
                .process(cipher.as_bytes())
        )
    }
}

fn main() {
    println!(
        "{}",
        process_string(
            math::FibonacciDigits::<num::BigUint>::new()
                .take(10000)
                .collect::<String>()
                .as_bytes(),
            {
                let mut op = Cipher::chain(Noop, Noop);
                for nums in data::NUMBERS2 {
                    let digits: String = nums.map(|x| x.to_string()).into_iter().collect();
                    op = op.chain(bytes::fuzzy_replace(digits.as_bytes(), &[], true));
                }
                for digits in ["692048501", "258949201"] {
                    op = op.chain(bytes::fuzzy_replace(digits.as_bytes(), &[], true));
                }
                op
            },
        )
        .unwrap()
    );
    //println!("{}", numbers1::calc(data::MEANING_OF_LIFE_NUM.into()));
    /*for n in data::NUMBERS2.into_iter().flatten() {
        println!("{n} {}", numbers1::calc(n.into()));
    }*/
    //return;
    //println!("{}", numbers1::calc(1416938177140558u64.into()));
    //panic!();
    // a758eeb757d82e28d8f2df0e2b
    // filtered::denoise_image();
    // 175fbfbf757f7bce
    /*let hex = "1bad0fcabc1ebdce";
    for num in schizo::reverse_numbers(hex) {
        let a = numbers1::calc(num.clone());
        // let b = process_string(&a, bytes::fuzzy_replace(b"692048501258949201", b"", false));
        // if a != b {
        println!("{num} {a}");
        // }
    }*/
    //println!("{}", numbers1::calc(692048u32.into()));
    // println!("{:?}", meaning_of_life::answer());
    // println!("{:?}", meaning_of_life::reassemble_image());
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
