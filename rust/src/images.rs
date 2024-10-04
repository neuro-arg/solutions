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

pub fn meaning_of_life_reassemble_v2(i: usize) {
    let mut pattern = [
        450, 450, 450, 450, 580, 580, 580, 580, 580, 285, 170, 170, 170, 170, 60, 45, 45, 585, 585,
        710, 715, 715, 715, 715, 390, 45, 45, 45, 45, 45, 405, 405, 405, 705, 575, 575, 575, 335,
        335, 460, 460, 165, 165, 360, 280, 280, 280, 280, 305, 460, 460, 460, 460, 370, 330, 565,
        565, 565, 370, 370, 370, 370, 370, 0, 0,
    ];
    pattern.rotate_left(i);
    // doesnt work obviously
    let mut img = image::open("../../neuro-arg.github.io/static/images/wdym-enhance-frame2.png")
        .unwrap()
        .into_luma8();
    let w = img.width() as usize;
    // let mut graph = image::DynamicImage::new_luma8(img.height(), 3840).into_luma8();
    // let h = 3840 - 8;
    for (i, line) in img.as_mut().chunks_exact_mut(w).enumerate() {
        let Some((mut rightmost_nonblack, _)) =
            line.iter().enumerate().rev().find(|(_, x)| **x >= 0x89)
        else {
            continue;
        };
        // line.rotate_right(w - rightmost_nonblack);
        for x in [
            1, 25, 39, 45, 110, 150, 155, 165, 204, 230, 275, 278, 330, 335, 400, 440, 455, 520,
            566, 691,
        ]
        .into_iter()
        {
            if w - rightmost_nonblack <= x {
                line.rotate_left(691 - x);
                if x == 1 {
                    line.rotate_left(88);
                }
                if x == 45 {
                    line.rotate_left(44);
                }
                if x == 25 {
                    line.rotate_left(26);
                }
                if x == 335 {
                    line.rotate_left(5);
                }
                if x == 155 {
                    line.rotate_left(6);
                }
                break;
            }
        }
        // line.rotate_left(pattern[i % pattern.len()]);
        /*println!("{w} {rightmost_nonblack}");
        let x = w - rightmost_nonblack;
        let pos = h - x as u32;
        graph.put_pixel(i as u32, pos, [0xFF].into());
        if pos + 1 < h {
            graph.put_pixel(i as u32, pos + 1, [0xFF].into());
        }
        if pos > 0 {
            graph.put_pixel(i as u32, pos - 1, [0xFF].into());
        }*/
    }
    println!("wtf");
    img.save(format!("out/{i:02}.png")).unwrap();
    // image::imageops::rotate270(&graph).save("graph.png").unwrap();
}

pub fn meaning_of_life_reassemble() {
    // only reassembles the text, the rest is on a best-effort basis
    let mut img = image::open("tmp.png") // "../../neuro-arg.github.io/static/images/wdym-enhance-frame2.png")
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
    println!("{ofs:?}");
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

pub fn candles_open(img: impl AsRef<Path>) -> image::DynamicImage {
    let img = image::open(img).unwrap();
    let img = if img.width() == 1920 {
        img.crop_imm(704, 156, 512, 768).into_luma8()
    } else {
        img.into_luma8()
    };
    image::imageops::rotate90(&img).into()
}

pub fn filtered_open(img: impl AsRef<Path>) -> Result<image::DynamicImage, image::ImageError> {
    let img = image::open(img)?;
    let img = if img.width() == 1920 {
        img.crop_imm(391, 0, 1920 - 391 * 2, 1080).into_luma8()
    } else {
        img.into_luma8()
    };
    Ok(img.into())
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
    fn sample(&self, x: u32, old_line: Option<&[u8]> /*, nice: &mut bool*/) -> Option<u8> {
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
            /*if x >= w / 2 {
                ms.reverse();
            }*/
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
                // *nice = ret >= 0x80 && ret < 0xF0;
                return Some(ret.saturating_mul(2 / c));
            }
            /*if *nice {
                if let Some(val) = old_line.and_then(|w| w.get(x as usize).copied()) {
                    return Some(val);
                }
            }*/
            // if we have to take from the intersection so be it
            for x in ms.into_iter().flatten() {
                if inter.contains(x) {
                    if let Some(p) = self.line.get(x as usize) {
                        // *nice = false;
                        return Some(*p);
                    }
                }
            }
        } else {
            // if this is in the left half, prefer rightmost coords
            // LR decomposed into two non-intersecting images is
            // lr lr, and it turns into r l after cropping
            /*if x < w / 2 {
                ms.reverse();
            }*/
            for x in ms.into_iter().flatten() {
                if let Some(p) = self.line.get(x as usize) {
                    ret = ret.saturating_add(*p);
                    c += 1;
                }
            }
            if c >= 1 {
                // *nice = ret >= 0x80 && ret < 0xF0;
                return Some(ret.saturating_mul(2 / c));
            }
        }
        // *nice = false;
        old_line.and_then(|w| w.get(x as usize).copied())
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

fn take_column(img: &[u8], w: u32, col: u32) -> image::GrayImage {
    let mut ret = image::GrayImage::new(1, img.len() as u32 / w);
    for (src, mut dst) in img
        .iter()
        .skip(col as usize)
        .step_by(w as usize)
        .zip(ret.rows_mut())
    {
        *dst.next().unwrap() = image::Luma([*src]);
    }
    ret
}

pub fn candles_denoise_graph(src: impl AsRef<Path>) -> image::DynamicImage {
    let count = 2565;
    let mut ret = image::GrayImage::new(count, 1080);
    let mut threads = vec![];
    let (red_tx, red_rx) = std::sync::mpsc::channel();
    for th in 0..15 {
        let path = src.as_ref().to_path_buf();
        let ret_len = ret.len();
        // fuck safety all my homies hate safety (this code has/had data races at times but it's fine because
        // i don't need full reproducibility)
        let ret = ret.as_mut_ptr() as usize;
        let red_tx = red_tx.clone();
        threads.push(std::thread::spawn(move || {
            let ret = unsafe { std::slice::from_raw_parts_mut(ret as *mut u8, ret_len) };
            for i in 0usize..(count as usize / 15) {
                let i = i * 15 + th;
                let j = i + 1;
                let div = if i < 451 {
                    1. // (450. - i as f64) / 240. + 1.
                }
                /*else if (558..923).contains(&i) {
                    ((923. - 558.) - (i - 558) as f64) / 350. + 1.
                }*/ /*else if (2305..2565).contains(&i) {
                    ((2565. - 2305.) - (i - 2305) as f64) / 2000. + 1.
                } */
                else {
                    1.
                };
                // println!("{div}");
                let mut path = path.clone();
                path.push(format!("{j:04}.png"));
                let img = filtered_open(path).unwrap().into_luma8();
                for (k, src) in img.as_ref().chunks_exact(img.width() as usize).enumerate() {
                    // let dst = ret.as_mut()[2 * (count as usize) * k];
                    let sampler = analyze_line(&src[..img.width() as usize]);
                    // [-w, w] -> [0, 255]
                    /*let x1 =
                        (sampler.x1 + img.width() as i32) as f64 / (img.width() * 2 / 255) as f64;
                    let x2 =
                        (sampler.x2 + img.width() as i32) as f64 / (img.width() * 2 / 255) as f64;
                    let x1 = (x1 as i64).clamp(0, 255) as u8;
                    let x2 = (x2 as i64).clamp(0, 255) as u8;*/
                    let xavg = ((sampler.x1 + sampler.x2) / 2 + img.width() as i32) as f64
                        / (img.width() * 2 / 255) as f64;
                    let xavg = (xavg as i64).clamp(0, 255) as u8;
                    ret[((/*2 **/k) as f64 / div) as usize * (count as usize) + i] = xavg;
                    // ret[((2 * k) as f64 / div) as usize * (count as usize) + count as usize + i] = x2;
                }
                let col = take_column(ret, count, i as u32);
                if let Some(rating) = crate::video::rate_candles_frame(col.into(), false) {
                    red_tx.send((i as u32, rating)).unwrap();
                }
            }
        }));
    }
    drop(red_tx);
    for t in threads {
        t.join().unwrap();
    }

    // now plot functions
    #[allow(dead_code)]
    fn plot(img: &mut image::RgbImage, mut f: impl FnMut(f64) -> f64, mut x: u32, x2: u32) {
        let h: i64 = img.height().into();
        let mut old = -1i64;
        let mut first = true;
        while x < img.width().min(x2) {
            let v = f(x as f64) as i64;
            if v < 0 || v >= h {
                if !first {
                    break;
                }
                x += 1;
                continue;
            }
            first = false;
            if old >= 0 {
                while old != v {
                    img.put_pixel(x, old as u32, image::Rgb([0xFF, 0x00, 0x00]));
                    if old > v {
                        old -= 1;
                    } else {
                        old += 1;
                    }
                }
            } else {
                old = v;
            }
            img.put_pixel(x, v as u32, image::Rgb([0xFF, 0x00, 0x00]));
            x += 1;
        }
    }
    let ret: image::DynamicImage = ret.into();
    let mut ret = ret.into_rgb8();

    //plot(&mut ret, |coord| coord * (-1.111) + 900., 558, 923);
    //plot(&mut ret, |coord| coord * (-2.222) + 2000., 558, 923);
    //plot(&mut ret, |coord| coord * (-3.333) + 3060., 558, 923);
    /*plot(&mut ret, |coord| coord * -0.14 + coord * (-0.5) + 360., 0, 451);
    plot(&mut ret, |coord| coord * -0.14 + coord * (-1.0) + 720., 0, 451);
    plot(&mut ret, |coord| coord * -0.14 + coord * (-1.5) + 1080., 0, 451);
    plot(&mut ret, |coord| coord * -0.14 + coord * (-2.0) + 1440., 0, 451);
    plot(&mut ret, |coord| coord * -0.14 + coord * (-2.5) + 1800., 0, 451);*/
    /*let data: Vec<(u32, u32)> = std::fs::read_to_string("patterns.txt")
        .unwrap()
        .split('\n')
        .filter(|x| !x.is_empty())
        .map(|x| x.split_once(' ').unwrap())
        .map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap()))
        .collect();
    for (mut a, b) in data {
        a -= 1;*/
    while let Ok((a, b)) = red_rx.recv() {
        let mut x = b;
        let mut c = [0xff, 0, 0];
        while x < ret.height() {
            ret.put_pixel(a, x, image::Rgb(c));
            c.rotate_right(1);
            x += b;
        }
    }
    ret.into()
}

pub fn candles_denoise_img(img: image::DynamicImage) -> image::DynamicImage {
    // for now, consider every line a single stripe
    let mut ret = image::GrayImage::new(img.width(), img.height());
    let img = img.into_luma8();
    let mut old_p = std::ptr::null::<u8>();
    let mut old_len = 0usize;
    // let mut nices = vec![false; img.width() as usize];
    for (src, dst) in img
        .as_ref()
        .chunks_exact(img.width() as usize)
        .zip(ret.as_mut().chunks_exact_mut(img.width() as usize))
    {
        let sampler = analyze_line(src);
        for (i, p) in dst.iter_mut().enumerate()
        /*.zip(nices.iter_mut())*/
        {
            *p = sampler
                .sample(
                    i as u32,
                    (!old_p.is_null())
                        .then_some(unsafe { std::slice::from_raw_parts(old_p, old_len) }),
                    // nice,
                )
                .unwrap_or_default();
        }
        old_p = dst.as_ptr();
        old_len = dst.len();
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

fn shift_step(x: &mut [u8], step: usize, offset: usize, shift_by: isize, buf: &mut Vec<u8>) {
    let shift_by = -shift_by;
    buf.clear();
    assert!(buf.capacity() >= x.len());
    for (i, q) in x.iter().copied().enumerate() {
        if (i + x.len() - offset) % step == 0 {
            buf.push(x[((i + x.len()) as isize + step as isize * shift_by) as usize % x.len()])
        } else {
            buf.push(q);
        }
    }
    x.copy_from_slice(buf);
}

pub fn study_denoise(img: image::DynamicImage) -> image::DynamicImage {
    let mut img = img.into_rgb8();
    let w = img.width();
    let mut buf = Vec::with_capacity(w as usize);
    for row in img.as_mut().chunks_exact_mut(w as usize) {
        for chunk in row.chunks_exact_mut(3) {
            let mut chunks = [chunk[0], chunk[1], chunk[2]];
            chunks.sort();
            let median = chunks[1];
            for pix in chunk {
                *pix = pix.saturating_sub(median);
            }
        }
        shift_step(row, 3, 0, 2, &mut buf);
        shift_step(row, 3, 2, -2, &mut buf);
    }
    img.into()
}

pub fn study_denoise_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    let mut threads = vec![];
    for i in 0..16 {
        let src = src.as_ref().to_path_buf();
        let dst = dst.as_ref().to_path_buf();
        threads.push(std::thread::spawn(move || {
            for j in 0..24 {
                let i = j * 16 + i + 12486;
                let mut src = src.clone();
                let mut dst = dst.clone();
                src.push(format!("{i}.png"));
                dst.push(format!("{:04}.png", i - 12485));
                study_denoise(image::open(src).unwrap()).save(dst).unwrap();
            }
        }));
    }
    for th in threads {
        th.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::images::{analyze_line, shift_step};

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

        let mut w = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        let mut buf = Vec::with_capacity(w.len());
        shift_step(&mut w, 3, 0, 2, &mut buf);
        assert_eq!(w, [3, 1, 2, 6, 4, 5, 0, 7, 8]);
        shift_step(&mut w, 3, 1, -2, &mut buf);
        assert_eq!(w, [3, 7, 2, 6, 1, 5, 0, 4, 8]);
        shift_step(&mut w, 3, 2, 1, &mut buf);
        assert_eq!(w, [3, 7, 8, 6, 1, 2, 0, 4, 5]);
    }
}
