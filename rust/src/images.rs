// best run in release mode, debug is slow for image processing
// (this is just my random experiments in image processing)

use std::path::Path;

pub fn split_img() {
    let img =
        image::open("../../neuro-arg.github.io/static/images/wdym-enhance-frame2.png").unwrap();
    let img = img.into_luma_alpha8();
    let mut total = 0;
    for i in (0..(img.height() as usize)).skip(1300).take(100) {
        let mut out = img.clone();
        let data = out.as_mut();
        data[0..i * img.width() as usize * 2].fill(0);
        data[(i + 1) * img.width() as usize * 2..].fill(0);
        total += data[i * img.width() as usize * 2..(i + 1) * img.width() as usize * 2]
            .windows(126 * 2)
            .skip(1690)
            .map(|x| {
                let a = *x.first().unwrap() as u64;
                let b = *x.get(x.len() - 2).unwrap() as u64;
                if a > 0xc0 && b > 1 {
                    1i64
                } else if a > 1 {
                    -100i64
                } else {
                    0
                }
            })
            .sum::<i64>();
        println!("{i} {total}");
        total = 0;
        // out.save(format!("out/{i:04}.png")).unwrap();
    }
}

pub fn test() {
    let img =
        image::open("../../neuro-arg.github.io/static/images/wdym-enhance-frame2.png").unwrap();
    let mut img = img.into_luma8();
    let (width, _height) = (img.width(), img.height());
    for row in img.chunks_exact_mut(width as usize) {
        for w in 1..row.len() {
            row[w - 1] = if (row[w] as i32) < (row[w - 1] as i32) - 20 {
                255
            } else {
                0
            };
        }
    }
    img.save("out.png").unwrap();
}

pub fn meaning_of_life_reassemble() {
    // only reassembles the text, the rest is on a best-effort basis
    let mut img = image::open("../../neuro-arg.github.io/static/images/wdym-enhance-frame2.png")
        .unwrap()
        .into_luma8();
    let x_start = 2025;
    let x_end = 3600;
    let (width, height) = (img.width(), img.height());
    let mut old2 = img.chunks_exact(width as usize).next().unwrap()[x_start..x_end].to_vec();
    let mut old = img.chunks_exact(width as usize).next().unwrap()[x_start..x_end].to_vec();
    let mut ofs = vec![0isize; height as usize];
    for row in img.chunks_exact_mut(width as usize) {
        row.rotate_left(425);
        let len = row.len();
        row[(len - 425)..].fill(0);
    }
    for rev in [false, true] {
        let mut chunks = img
            .chunks_exact_mut(width as usize)
            .enumerate()
            .collect::<Vec<_>>();
        if rev {
            chunks.reverse();
        }
        for (i, row) in chunks {
            let mut best_score: i64 = 0;
            let mut best_shift: isize = 0;
            // actually only shifts with mod 60 in {-5,0,5} are allowed
            // but that's too annoying to program
            for shift in ((-425isize / 5)..=(425isize / 5))
                .map(|x| x * 5)
                .filter(|shift| *shift + ofs[i] <= 600)
            {
                let w = &row[((x_start as isize + shift) as usize).min(row.len())
                    ..((x_end as isize + shift) as usize).min(row.len())];
                let score: i64 = old
                    .iter()
                    .copied()
                    .zip(old2.iter().copied())
                    .zip(w.iter().copied())
                    // .take(x_end - x_start - 425)
                    .map(|((a1, a2), b)| {
                        (if (a1 >= 70) && (b >= 70) { 20i64 } else { 0 })
                            + (if (a1 >= 128) && (b >= 128) { 10i64 } else { 0 })
                            + (if (a2 >= 128) && (b >= 128) { 15i64 } else { 0 })
                            + (if (a2 >= 70) && (b >= 70) { 30i64 } else { 0 })
                    })
                    .sum::<i64>()
                    - (shift as i64 + 5) / 50;
                if score >= best_score {
                    best_score = score;
                    best_shift = shift;
                }
            }
            if best_shift < 0 {
                row.rotate_right(-best_shift as usize);
            } else {
                row.rotate_left(best_shift as usize);
                let len = row.len();
                row[(len - best_shift as usize)..].fill(0);
            }
            ofs[i] += best_shift;
            println!("{i}/{height}");
            old2 = old;
            old = row[x_start..x_end].to_vec();
        }
    }
    img.save("out.png").unwrap();
}

pub fn filtered_stripe_analyze(img: image::DynamicImage) -> Vec<u32> {
    let _data = if img.width() == 1920 {
        img.crop_imm(391, 0, 1920 - 391 * 2, img.height())
            .into_luma8()
    } else {
        img.into_luma8()
    };
    todo!()
}

pub fn candles_open(img: &str) -> image::DynamicImage {
    let img = image::open(img).unwrap();
    let img = if img.width() == 1920 {
        img.crop_imm(704, 156, 512, 768).into_luma8()
    } else {
        img.into_luma8()
    };
    image::imageops::rotate90(&img).into()
}

pub fn filtered_open(img: impl AsRef<Path>) -> image::DynamicImage {
    let img = image::open(img).unwrap();
    let img = if img.width() == 1920 {
        img.crop_imm(391, 0, 1920 - 391 * 2, 1080).into_luma8()
    } else {
        img.into_luma8()
    };
    img.into()
}

#[derive(Copy, Clone, Debug)]
struct Rect {
    // Image coordinates (can be oob)
    x: i32,
    w: u32,
}

impl Rect {
    fn map(&self, x: u32) -> Option<u32> {
        if x >= self.w {
            None
        } else {
            (self.x + x as i32).try_into().ok()
        }
    }
    fn contains(&self, x: u32) -> bool {
        (self.x..self.x + self.w as i32).contains(&(x as i32))
    }
    fn intersect(&self, rect: &Self) -> Option<Self> {
        let x1 = self.x.max(rect.x);
        let x2 = (self.x + self.w as i32).min(rect.x + rect.w as i32);
        Some(Self {
            x: x1,
            w: (x2 - x1).try_into().ok()?,
        })
    }
}

struct CandlesPixelSource<'a> {
    line: &'a [u8],
    x1: i32,
    x2: i32,
}

// image part: x, intersection: X
// options:
// xXx -> take from left and right x multiplied by 2 by default
//        if some parts occur in both left and right, all the better
//        other than that, take from X, using the leftmost part for the
//        left part of the output and vice versa
// x x -> take from the left multiplied by 2 for the right part and
//        vice versa, intersection is just summed up without mult
impl<'a> CandlesPixelSource<'a> {
    fn sample(&self, x: u32) -> Option<u8> {
        let w = self.line.len() as u32;
        let r1 = Rect { x: self.x1, w };
        let r2 = Rect { x: self.x2, w };
        let inter = r1.intersect(&r2);
        let m1 = r1.map(x);
        let m2 = r2.map(x);
        let mut ms = [m1, m2];
        // sort x'es (we will prefer leftmost coords)
        ms.sort();
        let mut ret = 0u8;
        let mut c = 0;
        if let Some(inter) = inter {
            // if this is in the right half, prefer rightmost coords
            if x >= w / 2 {
                ms.reverse();
            }
            // prefer non-intersecting pixels
            for x in ms.into_iter().flatten() {
                if !inter.contains(x) {
                    if let Some(p) = self.line.get(x as usize) {
                        ret = ret.saturating_add(*p);
                        c += 1;
                    }
                }
            }
            if c >= 1 {
                return Some(ret.saturating_mul(2 / c));
            }
            // if we have to take from the intersection so be it
            for x in ms.into_iter().flatten() {
                if inter.contains(x) {
                    if let Some(p) = self.line.get(x as usize) {
                        return Some(*p);
                    }
                }
            }
        } else {
            // if this is in the left half, prefer rightmost coords
            // LR decomposed into two non-intersecting images is
            // lr lr, and it turns into r l after cropping
            if x < w / 2 {
                ms.reverse();
            }
            for x in ms.into_iter().flatten() {
                if let Some(p) = self.line.get(x as usize) {
                    ret = ret.saturating_add(*p);
                    c += 1;
                }
            }
            if c >= 1 {
                return Some(ret.saturating_mul(2 / c));
            }
        }
        None
    }
}

fn analyze_line(src: &[u8]) -> CandlesPixelSource {
    const WHITE: u8 = 0xA0;
    const BLACK: u8 = 0x30;
    let w = src.len() as u32;
    // analyze, find leftmost and rightmost white
    // and find the longest black
    let w1 = src
        .iter()
        .enumerate()
        .find(|(_, x)| **x >= WHITE)
        .map(|(i, _)| i as u32);
    let w2 = src
        .iter()
        .enumerate()
        .rev()
        .find(|(_, x)| **x >= WHITE)
        .map(|(i, _)| i as u32);
    let mut i = 0usize;
    let mut n = 0usize;
    let mut best = (i, n);
    let inter = w1.zip(w2);
    let mut left = true;
    for (j, p) in src.iter().enumerate() {
        if *p <= BLACK {
            n += 1;
        } else {
            if (i == 0 || inter.is_none()) && n > best.1 {
                best = (i, n);
            }
            i = j + 1;
            n = 0;
        }
    }
    if n > best.1 {
        best = (i, n);
        left = false;
    }
    let (x1, x2) = if let Some((w1, w2)) = w1.zip(w2) {
        // option 1. middle inter, whatever black
        //           w2-w..w2, w1..w1+w
        // option 2. left inter, right black
        //           w2-w..w2, b1-w..b1
        // option 3. right inter, left black
        //           b2..b2+w, w1..w1+w
        // println!("w1 {w1} w2 {w2} best {best:?} left {left:?}");
        if w1 <= 2 && (!left || best.1 < 10) {
            if left {
                best = (w as usize, 0);
            }
            let x1: i32 = w2 as i32 - w as i32 + 1;
            let x2: i32 = best.0 as i32 - w as i32;
            (x1, x2)
        } else if w2 >= w - 2 && (left || best.1 < 10) {
            if !left {
                best = (0, 0);
            }
            let x1: i32 = (best.0 + best.1) as i32;
            let x2: i32 = w1 as i32;
            (x1, x2)
        } else {
            let x1: i32 = w2 as i32 - w as i32 + 1;
            let x2: i32 = w1 as i32;
            (x1, x2)
        }
    } else {
        // worst case is the images are side by side,
        // not much i can do there
        //
        // option 4. no inter, middle black
        //           b2..b2+w, b1-w..b1
        let x1 = (best.0 + best.1) as i32;
        let x2 = best.0 as i32 - w as i32;
        (x1, x2)
    };
    CandlesPixelSource { line: src, x1, x2 }
}

pub fn candles_denoise_graph(src: impl AsRef<Path>) -> image::DynamicImage {
    let count = 2565;
    let mut ret = image::GrayImage::new(count, 1080 * 2);
    for i in 1usize..=(count as usize) {
        let mut path = src.as_ref().to_path_buf();
        path.push(format!("{i:04}.png"));
        let img = filtered_open(path).into_luma8();
        for (src, dst) in img
            .as_ref()
            .chunks_exact(img.width() as usize)
            .zip(ret.as_mut().chunks_exact_mut(2 * count as usize))
        {
            let sampler = analyze_line(src);
            // [-w, w] -> [0, 255]
            let x1 = (sampler.x1 + img.width() as i32) as f64 / (img.width() * 2 / 255) as f64;
            let x2 = (sampler.x2 + img.width() as i32) as f64 / (img.width() * 2 / 255) as f64;
            let x1 = (x1 as i64).clamp(0, 255) as u8;
            let x2 = (x2 as i64).clamp(0, 255) as u8;
            dst[i - 1] = x1;
            dst[i - 1 + count as usize] = x2;
        }
        // ret.as_mut().copy_from_slice(img.as_ref());
        // debug: take very 3rd line
        /*for lines3 in ret.chunks_exact_mut(3 * img.width() as usize) {
            let (a, b) = lines3.split_at_mut(img.width() as usize);
            let (b, c) = b.split_at_mut(img.width() as usize);
            // b.copy_from_slice(c);
            // a.copy_from_slice(c);
        }*/
        /*for lines2 in ret.chunks_exact_mut(2 * img.width() as usize) {
            let (a, b) = lines2.split_at_mut(img.width() as usize);
            // b.copy_from_slice(a);
            // a.copy_from_slice(c);
        }*/
    }
    ret.into()
}

pub fn candles_denoise_img(img: image::DynamicImage) -> image::DynamicImage {
    // for now, consider every line a single stripe
    let mut ret = image::GrayImage::new(img.width(), img.height());
    let img = img.into_luma8();
    for (src, dst) in img
        .as_ref()
        .chunks_exact(img.width() as usize)
        .zip(ret.as_mut().chunks_exact_mut(img.width() as usize))
    {
        let sampler = analyze_line(src);
        for (i, p) in dst.iter_mut().enumerate() {
            *p = sampler.sample(i as u32).unwrap_or_default();
        }
    }
    // ret.as_mut().copy_from_slice(img.as_ref());
    // debug: take very 3rd line
    /*for lines3 in ret.chunks_exact_mut(3 * img.width() as usize) {
        let (a, b) = lines3.split_at_mut(img.width() as usize);
        let (b, c) = b.split_at_mut(img.width() as usize);
        // b.copy_from_slice(c);
        // a.copy_from_slice(c);
    }*/
    /*for lines2 in ret.chunks_exact_mut(2 * img.width() as usize) {
        let (a, b) = lines2.split_at_mut(img.width() as usize);
        // b.copy_from_slice(a);
        // a.copy_from_slice(c);
    }*/
    ret.into()
}

pub fn filtered_denoise_img(img: image::DynamicImage) -> image::DynamicImage {
    let mut data = if img.width() == 1920 {
        img.crop_imm(391, 0, 1920 - 391 * 2, img.height())
            .into_luma8()
    } else {
        img.into_luma8()
    };
    let (width, _height) = (data.width(), data.height());
    // let mut ret = image::DynamicImage::new_luma8(width, height);
    // let ret_data = ret.as_mut_luma8().unwrap();
    for row in data.chunks_exact_mut(width as usize) {
        let rot = row
            .windows(75)
            .map(|v| v.iter().filter(|x| **x > 128).count().abs_diff(15))
            .enumerate()
            .max_by_key(|(_, v)| *v)
            .unwrap()
            .0
            + 75;
        row.rotate_left(rot);
    }
    data.into()
}

#[cfg(test)]
mod tests {
    use crate::images::analyze_line;

    use super::Rect;

    #[test]
    fn test() {
        let x = Rect { x: -192, w: 256 };
        // rect is an image starts at -192 ends at 64
        // if i want to place a pixel at 255 it should return 63
        assert_eq!(x.map(255), Some(63));
        assert_eq!(x.map(192), Some(0));
        assert_eq!(x.map(0), None);
        assert_eq!(x.map(256), None);
        let line = vec![0, 0, 0, 119, 120, 121, 255, 255, 255];
        let sampler = analyze_line(&line);
        assert_eq!(sampler.x2, 6);
        assert_eq!(sampler.x1, 3);
    }
}
