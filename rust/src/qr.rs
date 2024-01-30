use image::DynamicImage;
use itertools::Itertools;

#[allow(unused)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    Numeric = 1,
    Alphanumeric = 2,
    Byte = 4,
    Kanji = 8,
    Eci = 7,
}

fn bits<const N: usize>(x: u8) -> [bool; N] {
    let mut ret = [false; N];
    for (i, n) in ret.iter_mut().rev().enumerate() {
        *n = x & (1 << i) != 0;
    }
    ret
}

impl Mode {
    pub fn bits(self) -> [bool; 4] {
        bits(self as u8)
    }
}

#[allow(unused)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mask {
    M5,
    M4,
    M7,
    M6,
    M1,
    M0,
    M3,
    M2,
}

impl Mask {
    pub fn bits(self) -> [bool; 3] {
        bits(self as u8)
    }
    // may or may not (tm) be correct for all qr code versions
    pub fn flip(self, col: u32, row: u32) -> bool {
        match self {
            Self::M0 => (row + col) % 2 == 0,
            Self::M1 => row % 2 == 0,
            Self::M2 => col % 3 == 0,
            Self::M3 => (row + col) % 3 == 0,
            Self::M4 => (row / 2 + col / 3) % 2 == 0,
            Self::M5 => (row * col) % 2 + (row * col) % 3 == 0,
            Self::M6 => ((row * col) % 2 + (row * col) % 3) % 2 == 0,
            Self::M7 => ((row + col) % 2 + (row * col) % 3) % 2 == 0,
        }
    }
}

fn pix(x: bool) -> image::LumaA<u8> {
    if x {
        [0, 255].into()
    } else {
        [255, 255].into()
    }
}

#[allow(unused)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum EcLevel {
    H,
    Q,
    M,
    L,
}

impl EcLevel {
    pub fn bits(self) -> [bool; 2] {
        bits(self as u8)
    }
}

pub fn create_qr() {
    let mode = Mode::Byte;
    let mask = Mask::M7;
    // https://www.thonky.com/qr-code-tutorial/error-correction-table
    // "Total Number of Data Codewords" * 8
    let bit_capacity = 152;
    let s = "aaaaaaaaaaa";
    let ec_level = EcLevel::L;
    let version = 1;
    let size = 17 + version * 4;
    let mut img = DynamicImage::new_luma_a8(size, size).into_luma_alpha8();
    let empty = image::LumaA::from([127, 255]);
    for pix in img.pixels_mut() {
        *pix = empty;
    }
    // 1. the 3 square thingies
    for (center_x, center_y) in [3, size - 4].into_iter().cartesian_product([3, size - 4]) {
        if center_x != 3 && center_y != 3 {
            continue;
        }
        for (x, y) in (-4i32..=4i32).cartesian_product(-4i32..=4i32) {
            if (center_x as i32 + x).min(center_y as i32 + y) < 0
                || (center_x as i32 + x).max(center_y as i32 + y) >= size as i32
            {
                continue;
            }
            img.put_pixel(
                center_x + (x + 4) as u32 - 4,
                center_y + (y + 4) as u32 - 4,
                pix(x.abs().max(y.abs()) % 2 == 1 || (x == 0 && y == 0)),
            );
        }
    }
    // 2. 1010101 things
    for coord in 7..size - 7 {
        img.put_pixel(6, coord, pix(coord % 2 == 0));
        img.put_pixel(coord, 6, pix(coord % 2 == 0));
    }
    let mut extra_bits = [false; 2 + 3 + 10];
    // 3. error correction level
    extra_bits[..2].copy_from_slice(&ec_level.bits());
    // 4. mask
    extra_bits[2..5].copy_from_slice(&mask.bits());
    // 5. *provisional* ecc info
    extra_bits[5..].copy_from_slice(&[
        false, true, false, true, true, true, false, true, true, false,
    ]);
    for (i, bit) in extra_bits.into_iter().enumerate() {
        let pos = i as u32 + if i > 6 { size - 15 } else { 0 };
        img.put_pixel(pos + if i == 6 { 1 } else { 0 }, 8, pix(bit));
        img.put_pixel(
            8,
            size - 1 - pos + if (7..=8).contains(&i) { 1 } else { 0 },
            pix(bit),
        );
    }
    // 6. useless pixel
    img.put_pixel(8, size - 8, pix(true));
    let put = |img: &mut image::ImageBuffer<image::LumaA<u8>, Vec<u8>>, x, y, w| {
        img.put_pixel(x, y, pix(w != mask.flip(x, y)));
    };
    // 6. mode
    let mut qr_bits = Vec::<Option<bool>>::new();
    qr_bits.extend_from_slice(&mode.bits().map(Option::Some));
    qr_bits.extend_from_slice(&bits::<8>(s.len() as u8).map(Option::Some));
    for ch in s.bytes() {
        qr_bits.extend(
            &bits::<8>(ch)
                .into_iter()
                .enumerate()
                .map(|(i, x)| (i == 0).then_some(x))
                .collect::<Vec<Option<_>>>(),
        );
    }
    // pad
    {
        while qr_bits.len() % 8 != 0 {
            qr_bits.push(Some(false));
        }
        let mut pad = std::iter::repeat([236u8, 17]).flatten();
        while qr_bits.len() < bit_capacity {
            qr_bits.extend_from_slice(&bits::<8>(pad.next().unwrap()).map(Option::Some));
        }
    }
    // get available positions
    let mut x = size - 1;
    let mut y = size - 1;
    let mut up = true;
    let mut positions = vec![];
    let mut last_col = 0;
    while x >= 1 || y > 0 {
        if *img.get_pixel(x, y) == empty {
            last_col = x;
            positions.push((x, y));
            positions.push((x - 1, y));
        }
        if up {
            if y == 0 {
                if last_col == x {
                    if x < 2 {
                        break;
                    }
                    x -= 2;
                } else {
                    x -= 1;
                }
                up = false;
                continue;
            }
            y -= 1;
        } else {
            if y == size - 1 {
                if last_col == x {
                    if x < 2 {
                        break;
                    }
                    x -= 2;
                } else {
                    x -= 1;
                }
                up = true;
                continue;
            }
            y += 1;
        }
    }
    // fill the qr data
    for (bit, (x, y)) in qr_bits.into_iter().zip(positions) {
        if let Some(bit) = bit {
            put(&mut img, x, y, bit);
        }
    }
    image::DynamicImage::from(img)
        .resize(size * 16, size * 16, image::imageops::FilterType::Nearest)
        .save("tmp.png")
        .unwrap();
}
