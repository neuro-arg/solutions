// best run in release mode, debug is slow for image processing
// (this is just my random experiments in image processing)

use std::path::Path;

pub fn meaning_of_life_reassemble() {
    // doesn't really work well, oh well
    let mut img =
        image::open("../../neuro-arg.github.io/static/images/wdym-enhance-frame2.png").unwrap();
    let x_start = 3300;
    let x_end = 3600;
    let y_start = 800;
    let (width, height) = (img.width(), img.height());
    let data = img.as_mut_rgb8().unwrap();
    let mut old2 = data.chunks_exact(width as usize * 3).nth(y_start).unwrap()
        [x_start * 3..x_end * 3]
        .to_vec();
    let mut old = data.chunks_exact(width as usize * 3).nth(y_start).unwrap()
        [x_start * 3..x_end * 3]
        .to_vec();
    for (i, row) in data
        .chunks_exact_mut(width as usize * 3)
        .skip(y_start)
        .enumerate()
    {
        let mut best_score: u64 = 0;
        let mut best_shift: isize = 0;
        // for mut shift in -((width - x_end as u32) as isize)..=(width - x_end as u32) as isize {
        for mut shift in [0, 128, 234, 288, 352, 418] {
            shift *= 3;
            let w = &row[((x_start as isize * 3 + shift) as usize).min(row.len())
                ..((x_end as isize * 3 + shift) as usize).min(row.len())];
            let score: u64 = old
                .iter()
                .copied()
                .zip(old2.iter().copied())
                .zip(w.iter().copied())
                .map(|((a1, a2), b)| {
                    (if (a1 >= 80) && (b >= 80) { 2u64 } else { 0 })
                        + (if (a1 >= 128) && (b >= 128) { 2u64 } else { 0 })
                        + (if (a2 >= 80) && (b >= 80) { 1u64 } else { 0 })
                })
                .sum();
            if score > best_score {
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
        println!("{i}/{height}");
        old2 = old;
        old = row[x_start * 3..x_end * 3].to_vec();
        //row[..x_start * 3].fill(0);
        //row[x_end * 3..].fill(0);
    }
    img.save("out.png").unwrap();
}

pub fn filtered_denoise(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    let data = image::open(src.as_ref()).unwrap();
    let data = if data.width() == 1920 {
        data.crop_imm(391, 0, 1920 - 391 * 2, data.height())
            .into_luma8()
    } else {
        data.into_luma8()
    };
    let (width, height) = (data.width(), data.height());
    let mut ret = image::DynamicImage::new_luma8(width, height);
    let ret_data = ret.as_mut_luma8().unwrap();
    for (row, ret_row) in data
        .chunks_exact(width as usize)
        .zip(ret_data.chunks_exact_mut(width as usize))
    {
        let rot = row
            .windows(75)
            .map(|v| v.iter().filter(|x| **x > 128).count().abs_diff(15))
            .enumerate()
            .max_by_key(|(_, v)| *v)
            .unwrap()
            .0
            + 75;
        ret_row.copy_from_slice(row);
        ret_row.rotate_left(rot);
    }
    ret.save(dst.as_ref()).unwrap();
}
