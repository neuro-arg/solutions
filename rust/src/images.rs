// best run in release mode, debug is slow for image processing
// (this is just my random experiments in image processing)

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
    for (_i, row) in img.chunks_exact_mut(width as usize).enumerate() {
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

