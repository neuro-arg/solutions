#![allow(clippy::never_loop)]
#![feature(iter_array_chunks)]
// two small combinator libraries for processing data and numbers
pub mod audio;
pub mod bytes;
pub mod data;
pub mod images;
pub mod math;
pub mod numbers;
pub mod qr;
pub mod schizo;
#[cfg(feature = "video")]
pub mod video;
pub mod curvefit;

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

    pub fn calc(num: BigUint, key: &str) -> String {
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
        for (a, b) in num.to_string().chars().zip(key.chars()) {
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
            &mut base64().chain(decrypt_with(calc(data::NUMBER.into(), "abcdef"))),
        )
        .unwrap()
    }
}

pub mod study {
    use crate::{bytes::*, data, numbers1};

    pub fn phone() -> String {
        process_string(
            data::STUDY_BASE64,
            &mut base64().chain(decrypt_with(numbers1::calc(data::NUMBER.into(), "abcdef"))),
        )
        .unwrap()
    }

    pub fn malbolge_code() -> String {
        process_string(data::STUDY_MALBOLGE, &mut malbolge()).unwrap()
    }
}

pub mod numbers2 {
    use crate::{bytes::*, data, math, split_off_last_word, Noop};

    pub fn stage1() -> (String, String) {
        let mut ret = process_string(
            data::NUMBERS2_BASE64,
            &mut base64().chain(decrypt_with(data::NUMBERS2_NUEROS)),
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
            &mut {
                let mut op = Cipher::chain(Noop, Noop);
                for nums in data::NUMBERS2 {
                    let digits: String = nums.map(|x| x.to_string()).into_iter().collect();
                    op = op.chain(fuzzy_replace(digits.as_bytes(), &[], true));
                }
                op
            },
        )
        .unwrap()[..16];
        let mut ret = process_string(stage1().1, &mut base64().chain(decrypt_with(key))).unwrap();
        let cipher = split_off_last_word(&mut ret);
        (ret, cipher)
    }

    pub fn stage3() -> String {
        process_string(stage2().1.as_bytes(), &mut malbolge()).unwrap()
    }

    pub fn all_stages() -> (String, String, String) {
        (stage1().0, stage2().0, stage3())
    }
}

pub mod candle {
    use crate::{
        bytes::{base64, decrypt_with, process_string, Cipher},
        data,
        math::FibonacciDigits,
        numbers1,
    };

    pub fn numbers_answer() -> String {
        let k0 = data::NUMBER.to_string();
        let mut k = k0.chars();
        let key = k0.clone()
            + &FibonacciDigits::<num::BigUint>::new()
                .skip_while(|x| match k.next() {
                    Some(y) => {
                        if y != *x {
                            k = k0.chars();
                        }
                        true
                    }
                    None => false,
                })
                .take(16)
                .collect::<String>();
        let key = key.parse().unwrap();
        let key = numbers1::calc(key, data::CANDLE_GIST);
        process_string(
            data::CANDLE_NUMBERS_BASE64,
            &mut base64().chain(decrypt_with(key)),
        )
        .unwrap()
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
        process_string(
            data::SOUNDCLOUD_BASE64,
            &mut base64().chain(decrypt_with(key)),
        )
        .unwrap()
    }
}

pub mod filtered {
    use crate::{bytes::*, data};

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

    pub fn answer() -> String {
        let code =
            Shift::new(data::FILTERED_TITLE).nth(784).unwrap() + data::FILTERED_GUESSED_CODE_END;
        assert_eq!(
            malbolge().process(code.as_bytes()).unwrap(),
            b"hello world!"
        );
        process_string(
            data::FILTERED_BASE64,
            &mut base64().chain(decrypt_with(code[code.len() - 16..].as_bytes())),
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
        /*for num in schizo::reverse_numbers(&hex()) {
            println!("{num}");
        }*/
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
    #[cfg(feature = "video")]
    ffmpeg::init().unwrap();
    // video::brightness_graph2("f_psv", "brightness/psv1.png", 637, 476);
    // video::brightness_graph2("f_psv", "brightness/psv.png", 0, 0);
    // video::brightness_graph2("f_study", "brightness/study_denoise2.png", 0, 0);
    // video::brightness_graph2("f_helloworld", "brightness/hello.png", 0, 0);
    // video::brightness_graph2("f_unfiltered", "brightness/filtered.png", 100, 0);
    // video::brightness_graph2("f_mol", "brightness/mol.png", 0, 0);
    // video::rate_candles_frames("f_filtered", "f_fd1"); return;
    // images::filtered_denoise_img(image::open("f_fd/0134_023.png").unwrap()).save("out.png").unwrap(); return;
    // image::imageops::rotate270(&images::candles_denoise_img(images::candles_open("f_candles2/2696.png"))).save("out.png").unwrap(); return;
    // images::candles_denoise_img(images::filtered_open("f_filtered/0356.png")).save("out.png").unwrap(); return;
    video::filtered_denoise_dir("f_filtered", "f_unfiltered2"); return;
    // video::candles_denoise_dir("f_candles", "f_uncandles2"); return;
    // images::candles_denoise_graph("f_filtered").save("out.png").unwrap(); return;
    // 12486-12865
    // images::study_denoise_dir("f_study", "f_fstudy"); return;
    // video::rate_candles_frame(image::open("mol.png").unwrap(), true);
    // video::candles_cycles("f_mol", "f_unmol"); return;
    /*for i in 0..1 {
    images::meaning_of_life_reassemble_v2(i);
    } return;
    for i in 60..70 {
        for j in 0..3 {
            let j = j * 19;
            // video::candles_frame("mol.png", format!("mol/{i:03}_{j:03}.png"), i, j);
        }
    }*/
    //video::pattern_search(); return;
    //audio::geiger(); return;
    if std::env::args().count() == 2 {
        video::rate_candles_frame(
            image::open(format!(
                "f_filtered/{}.png",
                std::env::args().nth(1).unwrap()
            ))
            .unwrap(),
            true,
        );
    } else {
        video::why_candles_frame(
            format!("f_filtered/{}.png", std::env::args().nth(1).unwrap()),
            std::env::args().nth(2).unwrap().parse().unwrap(),
        );
        video::candles_frames(
            "f_filtered",
            "tmp",
            std::env::args().nth(1).unwrap(),
            std::env::args().nth(2).unwrap().parse().unwrap(),
        );
        let mut i = 0;
        println!("tmp/{}_{i:03}.png", std::env::args().nth(1).unwrap());
        /*while let Ok(img) = images::filtered_open(format!(
            "tmp/{}_{i:03}.png",
            std::env::args().nth(1).unwrap()
        )) {
            println!("{i}");
            images::candles_denoise_img(img)
                .save(format!(
                    "tmp/{}u_{i:03}.png",
                    std::env::args().nth(1).unwrap()
                ))
                .unwrap();
            i += 1;
        }*/
        images::candles_denoise_img(
            images::filtered_open(format!(
                "f_filtered/{}.png",
                std::env::args().nth(1).unwrap()
            ))
            .unwrap(),
        )
        .save(format!("tmp/{}u.png", std::env::args().nth(1).unwrap()))
        .unwrap();
    }
    // cli tool 2
    /*let mut threads = Vec::new();
    for i in 1..=69 {
        threads.push(std::thread::spawn(move || video::candles_frame(
            std::env::args().nth(1).unwrap(),
            format!("tmp/{i:02}.png"),
            i,
            0,
        )));
    }
    for thread in threads {
        thread.join().unwrap();
    }*/
    // qr::create_qr();
    // println!("{}", candle::numbers_answer());
    return;
    /*video::xor_frames("f_unfiltered", "f_test");
    video::brightness_graph(
        "../../yt/Meaning of life [IRzyqcKljxw].mkv",
        "brightness/mol.png",
    );
    video::brightness_graph("../../yt/Numbers II [giJI-TDbO5k].mkv", "brightness/n2.png");
    video::brightness_graph("../../yt/Numbers [wc-QCoMm4J8].mkv", "brightness/n1.png");
    video::brightness_graph(
        "../../yt/Public Static Void [ymYFqNUt05g].mkv",
        "brightness/psv.png",
    );
    video::brightness_graph("../../yt/Study [zMlH7RH6psw].mkv", "brightness/study.png");
    video::brightness_graph(
        "../../yt/[Filtered] [4j5oDzRiXUA].mkv",
        "brightness/filtered.png",
    );
    video::brightness_graph(
        "../../yt/hello world! [OiKrYrbs3Qs].mkv",
        "brightness/helloworld.png",
    );
    return;*/
    //images::filtered_denoise("o.mkv", "out.ivf");
    //images::idk();
    //images::meaning_of_life_reassemble();
    //images::split_img();
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
    //let hex = meaning_of_life::hex();
    /*let data = bytes::base64()
    .process(data::MEANING_OF_LIFE_BASE64.as_bytes())
    .unwrap();*/
    //let mut cipher = bytes::decrypt_data(data);
    /*for num in schizo::reverse_numbers(&hex) {
        for idk in num.to_string().as_bytes().windows(16) {
            if let Ok(data) = process_string(idk, &mut cipher) {
                println!("{:?}", data);
            }
        }
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
