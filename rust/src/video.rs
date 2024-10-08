use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::mpsc,
};

use ffmpeg::{
    format::{input, Pixel},
    frame::Video,
    media::Type,
    software::scaling::{Context, Flags},
};
use image::{GenericImage, RgbImage};

/*
const FILTERED_W: u32 = 1920 - 391 * 2;

// couldnt quite get encoding to work well
pub fn filtered_denoise_video(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    ffmpeg::init().unwrap();
    ffmpeg::log::set_level(ffmpeg::log::Level::Debug);
    let mut ictx = ffmpeg::format::input(&src).unwrap();
    let ist = ictx.streams().best(ffmpeg::media::Type::Video).unwrap();
    let ist_index = ist.index();
    let context_decoder =
        ffmpeg::codec::context::Context::from_parameters(ist.parameters()).unwrap();
    let mut decoder = context_decoder.decoder().video().unwrap();
    let mut scaler = ffmpeg::software::scaling::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::format::Pixel::GRAY8,
        decoder.width(),
        decoder.height(),
        ffmpeg::software::scaling::Flags::BILINEAR,
    )
    .unwrap();
    let config = rav1e::EncoderConfig {
        width: FILTERED_W as usize,
        height: 1080,
        ..Default::default()
    };
    let config = rav1e::Config::default()
        .with_encoder_config(config)
        .with_threads(16);
    let mut encoder: rav1e::Context<u8> = config.new_context().unwrap();

    let mut ofile = std::fs::File::create(dst).unwrap();
    ivf::write_ivf_header(&mut ofile, FILTERED_W as usize, 1080, 2997, 100);
    let mut threads_tx = vec![];
    let mut threads_rx = vec![];
    for _ in 0..16 {
        let (tx1, rx1) = std::sync::mpsc::channel::<image::DynamicImage>();
        let (tx2, rx2) = std::sync::mpsc::channel::<image::DynamicImage>();
        std::thread::spawn(move || {
            while let Ok(img) = rx1.recv() {
                tx2.send(filtered_denoise_img(img)).unwrap();
            }
        });
        threads_tx.push(tx1);
        threads_rx.push(rx2);
    }
    let queue_len = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let queue_len1 = queue_len.clone();
    let thread = std::thread::spawn(move || {
        let mut i = 0;
        loop {
            for rx in &threads_rx {
                let Ok(img) = rx.recv() else {
                    encoder.flush();
                    while let Ok(encoded) = encoder.receive_packet() {
                        ivf::write_ivf_frame(&mut ofile, encoded.input_frameno, &encoded.data);
                    }
                    ofile.flush().unwrap();
                    return;
                };
                let mut frame = encoder.new_frame();
                frame.planes[0].copy_from_raw_u8(img.as_luma8().unwrap(), img.width() as usize, 1);
                i += 1;
                println!(
                    "{} frames in queue ({i} processed)",
                    queue_len.fetch_sub(1, Ordering::Relaxed)
                );
                encoder.send_frame(frame).unwrap();
                if i % 128 == 0 {
                    while let Ok(encoded) = encoder.receive_packet() {
                        ivf::write_ivf_frame(&mut ofile, encoded.input_frameno, &encoded.data);
                    }
                    ofile.flush().unwrap();
                }
            }
        }
    });
    let mut frame_c = 0;
    let mut process_decoder = |decoder: &mut ffmpeg::decoder::Video| {
        let mut decoded = ffmpeg::util::frame::Video::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            frame_c += 1;
            let mut frame = ffmpeg::util::frame::Video::empty();
            scaler.run(&decoded, &mut frame).unwrap();
            let mut img = image::DynamicImage::new_luma8(frame.width(), frame.height());
            img.as_mut_luma8().unwrap().copy_from_slice(frame.data(0));
            queue_len1.fetch_add(1, Ordering::Relaxed);
            threads_tx[frame_c % threads_tx.len()].send(img).unwrap();
        }
    };
    for (stream, mut packet) in ictx.packets() {
        if stream.index() == ist_index {
            packet.rescale_ts(stream.time_base(), decoder.time_base());
            decoder.send_packet(&packet).unwrap();
            process_decoder(&mut decoder);
        }
    }
    decoder.send_eof().unwrap();
    process_decoder(&mut decoder);
    drop(threads_tx);
    thread.join().unwrap();
}*/

// ffmpeg -i src.mkv a/%04d.png
// run this
// ffmpeg -i b/%04d.png dst.mkv
pub fn filtered_denoise_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    let mut i = 1usize;
    let mut threads = vec![];
    for _ in 0..16 {
        let dst = dst.as_ref().to_owned();
        let (tx1, rx1) = std::sync::mpsc::channel::<(usize, PathBuf)>();
        threads.push((
            tx1,
            std::thread::spawn(move || {
                while let Ok((i, img)) = rx1.recv() {
                    println!("processing {i}");
                    crate::images::candles_denoise_img(crate::images::filtered_open(img).unwrap())
                        .save(dst.join(&format!("{i:04}.png")))
                        .unwrap();
                }
            }),
        ));
    }
    while let Some(x) = Some(src.as_ref().join(&format!("{i:04}.png"))).filter(|x| x.is_file()) {
        threads[i % threads.len()].0.send((i, x)).unwrap();
        i += 1;
    }
    for (tx, thread) in threads {
        drop(tx);
        thread.join().unwrap();
    }
}

pub fn candles_denoise_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    let mut i = 2538usize;
    let mut threads = vec![];
    for _ in 0..16 {
        let dst = dst.as_ref().to_owned();
        let (tx1, rx1) = std::sync::mpsc::channel::<(usize, PathBuf)>();
        threads.push((
            tx1,
            std::thread::spawn(move || {
                while let Ok((i, img)) = rx1.recv() {
                    println!("processing {i}");
                    image::imageops::rotate270(&crate::images::candles_denoise_img(
                        crate::images::candles_open(img),
                    ))
                    .save(dst.join(&format!("{i:04}.png")))
                    .unwrap();
                }
            }),
        ));
    }
    while let Some(x) = Some(src.as_ref().join(&format!("{i:04}.png"))).filter(|x| x.is_file()) {
        threads[i % threads.len()].0.send((i, x)).unwrap();
        i += 1;
    }
    for (tx, thread) in threads {
        drop(tx);
        thread.join().unwrap();
    }
}

pub fn frames(path: impl AsRef<Path>) -> impl Iterator<Item = image::DynamicImage> {
    let ictx = input(&path).unwrap();
    let input = ictx.streams().best(Type::Video).unwrap();
    let video_stream_index = input.index();
    let context_decoder =
        ffmpeg::codec::context::Context::from_parameters(input.parameters()).unwrap();
    let decoder = context_decoder.decoder().video().unwrap();
    let scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )
    .unwrap();
    struct FrameIter {
        decoder: ffmpeg::codec::decoder::Video,
        ictx: ffmpeg::format::context::Input,
        scaler: Context,
        stream_idx: usize,
    }
    impl FrameIter {
        fn poll(
            decoder: &mut ffmpeg::codec::decoder::Video,
            scaler: &mut Context,
        ) -> Option<image::DynamicImage> {
            let mut decoded = Video::empty();
            decoder.receive_frame(&mut decoded).ok()?;
            let mut rgb_frame = Video::empty();
            scaler.run(&decoded, &mut rgb_frame).ok()?;
            let mut img = image::DynamicImage::new_rgb8(rgb_frame.width(), rgb_frame.height());
            img.as_mut_rgb8()?.copy_from_slice(rgb_frame.data(0));
            Some(img)
        }
    }
    impl Iterator for FrameIter {
        type Item = image::DynamicImage;
        fn next(&mut self) -> Option<Self::Item> {
            if let Some(img) = Self::poll(&mut self.decoder, &mut self.scaler) {
                return Some(img);
            }
            if self.stream_idx == usize::MAX {
                return None;
            }
            for (stream, packet) in self.ictx.packets() {
                if stream.index() == self.stream_idx {
                    self.decoder.send_packet(&packet).unwrap();
                    if let Some(img) = Self::poll(&mut self.decoder, &mut self.scaler) {
                        return Some(img);
                    }
                }
            }
            self.decoder.send_eof().unwrap();
            self.stream_idx = usize::MAX;
            if let Some(img) = Self::poll(&mut self.decoder, &mut self.scaler) {
                return Some(img);
            }
            None
        }
    }

    FrameIter {
        decoder,
        ictx,
        scaler,
        stream_idx: video_stream_index,
    }
}

pub fn brightness_graph(video: impl AsRef<Path>, out: impl AsRef<Path>) {
    let mut data = Vec::new();
    let mut min = f64::MAX;
    let mut max = f64::MIN;
    for (i, frame) in frames(video).enumerate() {
        if i % 24 == 0 {
            println!("{}", i / 24);
        }
        let frame = frame.into_rgb8();
        let sum: u64 = frame.iter().map(|x| *x as u64).sum();
        let avg = sum as f64 / frame.len() as f64;
        data.push(avg);
        min = min.min(avg);
        max = max.max(avg);
    }
    let h = 2040u32;
    let mut img = image::DynamicImage::new_luma8(data.len() as u32, h + 8);
    for (i, x) in data.into_iter().enumerate() {
        let pos = h - (((x - min) * h as f64 / max) as u32 + 1).clamp(1, h) + 4;
        img.put_pixel(i as u32, pos, [0xFF, 0xFF, 0xFF, 0xFF].into());
        if pos + 1 < h {
            img.put_pixel(i as u32, pos + 1, [0xFF, 0xFF, 0xFF, 0xFF].into());
        }
        if pos > 0 {
            img.put_pixel(i as u32, pos - 1, [0xFF, 0xFF, 0xFF, 0xFF].into());
        }
    }
    img.save(out).unwrap();
}

pub fn brightness_graph2(dir: impl AsRef<Path>, out: impl AsRef<Path>, start: usize, len: usize) {
    let mut data = Vec::new();
    let mut min = f64::MAX;
    let mut max = f64::MIN;
    let mut rxs = VecDeque::new();
    for j in 1..=16 {
        let dir = dir.as_ref().to_path_buf();
        let (tx, rx) = mpsc::sync_channel(64);
        rxs.push_back(rx);
        std::thread::spawn(move || {
            for i in 0..999999 {
                let Ok(img) = image::open(dir.join(&format!("{:04}.png", i * 16 + j))) else {
                    break;
                };
                let frame = crate::images::study_denoise(img).into_rgb8();
                if tx.send(frame).is_err() {
                    break;
                }
            }
        });
    }
    let mut i = 0;
    let mut old = image::open(dir.as_ref().join("0001.png"))
        .unwrap()
        .into_rgb8();
    while let Ok(frame) = {
        let rx = rxs.pop_front().unwrap();
        let x = rx.recv();
        rxs.push_back(rx);
        x
    } {
        if i % 100 == 0 {
            println!("{i}");
        }
        if i < start {
            i += 1;
            old = frame;
            continue;
        }
        let sum: i64 = frame
            .iter()
            .zip(old.iter())
            .map(|(a, b)| (*a as i64 ^ *b as i64))
            .sum();
        let avg = sum as f64 / frame.len() as f64;
        data.push(avg);
        min = min.min(avg);
        max = max.max(avg);
        i += 1;
        if len != 0 && i >= start + len {
            break;
        }
        old = frame;
    }
    println!("{min} {max}");
    let w = data.len() as u32;
    let h = 2040u32;
    let mut img = image::DynamicImage::new_luma8(w, h + 8);
    for (i, x) in data.into_iter().enumerate() {
        let pos = h - (((x - min) * h as f64 / (max - min)) as u32 + 1).clamp(1, h) + 4;
        for i in i.saturating_sub(2)..=i.saturating_add(2).min(w as usize - 1) {
            for y in pos.saturating_sub(2)..=pos.saturating_add(2).min(h + 7) {
                img.put_pixel(i as u32, y, [0xFF, 0xFF, 0xFF, 0xFF].into());
            }
        }
    }
    img.save(out).unwrap();
}

pub fn xor_frames(dir: impl AsRef<Path>, out: impl AsRef<Path>) {
    let mut rxs = VecDeque::new();
    for j in 1..=16 {
        let dir = dir.as_ref().to_path_buf();
        let (tx, rx) = mpsc::sync_channel(64);
        rxs.push_back(rx);
        std::thread::spawn(move || {
            for i in 0..999999 {
                let Ok(img) = image::open(dir.join(&format!("{:04}.png", i * 16 + j))) else {
                    break;
                };
                let frame = img.into_rgb8();
                if tx.send(frame).is_err() {
                    break;
                }
            }
        });
    }
    let mut old = image::open(dir.as_ref().join("0001.png"))
        .unwrap()
        .into_rgb8();
    let mut i = 0;
    while let Ok(frame) = {
        let rx = rxs.pop_front().unwrap();
        let x = rx.recv();
        rxs.push_back(rx);
        x
    } {
        i += 1;
        for (x, a) in old.as_mut().iter_mut().zip(frame.as_ref().iter()) {
            *x ^= a;
        }
        old.save(out.as_ref().join(format!("{i:04}.png"))).unwrap();
        old = frame;
    }
}

// deduces why i am so fucking stupid
// 543
// 543
// 543
// 543
// 543
// 543
// aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
// 543
// 543
// xyz
// abc
// 572943
// 5435435435435435435435435435435435435435435435435435435435435435435435435435 h
pub fn why_candles_frame(src: impl AsRef<Path>, n: usize) {
    let img = image::open(src).unwrap();
    let frame = img.into_rgb8();
    let mut img = RgbImage::new(frame.width(), frame.height() / n as u32);
    let mut out = RgbImage::new(frame.width(), frame.height());
    for ofs in 0..n {
        for (dst, src) in img.rows_mut().zip(frame.rows().skip(ofs).step_by(n)) {
            for (dst, src) in dst.zip(src) {
                *dst = *src;
            }
        }
        let mut prev = None::<&[u8]>;
        // for each row, diff this row with prev row
        for (rown, row) in img
            .as_ref()
            .chunks_exact(3 * img.width() as usize)
            .enumerate()
        {
            if let Some(prev) = prev {
                for (i, (a, b)) in prev.iter().zip(row.iter()).enumerate() {
                    //sum += (*a ^ *b) as u64; // a.abs_diff(*b) as u64;
                    // *out.rows_mut().nth(ofs * n + rown).unwrap().nth(i).unwrap()
                    out.as_mut()[(ofs + rown * n) * img.width() as usize * 3 + i] = *a ^ *b;
                }
            }
            prev = Some(row);
        }
    }
    out.save("out.png").unwrap();
}

pub fn pattern_search() {
    let data: Vec<(usize, i32)> = std::fs::read_to_string("patterns.txt")
        .unwrap()
        .split('\n')
        .filter(|x| !x.is_empty())
        .map(|x| x.split_once(' ').unwrap())
        .map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap()))
        .collect();
    let data = {
        let mut x = vec![0i32; data.last().unwrap().0 + 1];
        for (i, w) in data {
            x[i] = w;
        }
        x
    };
    // let mut all_patterns = vec![];
    let mut occupied = vec![false; data.len()];
    for step in 0..data.len() {
        for start in 0..step {
            let mut cur = i32::MIN;
            let mut cur_start = usize::MAX;
            let mut len = 0;
            let mut conf = 0;
            let mut old = None;
            let mut unconf_row = 0;
            let dump = |occupied: &mut [bool],
                        len: &mut i32,
                        conf: &mut i32,
                        cur: i32,
                        cur_start: usize,
                        i: usize| {
                if ((*len > 4 && *conf >= ((*len * 2) / 3)) || (*len >= 4 && *conf == *len))
                    && cur != i32::MIN
                    && cur_start != usize::MAX
                    && !occupied[cur_start - step]
                    && data[cur_start - step] != 0
                {
                    println!(
                        "{}..{} step {step} diff {cur} len {conf}/{len}",
                        cur_start - step,
                        i - step
                    );
                    if *len > 8 {
                        for x in occupied
                            .iter_mut()
                            .skip(cur_start - step)
                            .step_by(step)
                            .take(*len as usize + 2)
                        {
                            *x = true;
                        }
                    }
                }
                *len = 0;
                *conf = 0;
            };
            let mut last_i = 0;
            let mut ignore = true;
            for (i, x) in data.iter().copied().enumerate().skip(start).step_by(step) {
                if let Some(old1) = old {
                    if !ignore && (x == 0 || x - old1 == cur) {
                        len += 1;
                        if x - old1 == cur {
                            conf += 1;
                            unconf_row = 0;
                        } else {
                            unconf_row += 1;
                        }
                    } else {
                        len -= unconf_row;
                        dump(&mut occupied, &mut len, &mut conf, cur, cur_start, i);
                        ignore = x == 0 || occupied[i];
                        cur_start = i;
                        cur = x - old1;
                    }
                    old = Some(old1 - cur);
                }
                if x != 0 {
                    old = Some(x);
                }
                last_i = i;
            }
            dump(&mut occupied, &mut len, &mut conf, cur, cur_start, last_i);
        }
    }
}

pub fn rate_candles_frame(img: image::DynamicImage, dbg: bool) -> Option<u32> {
    let frame = img.into_luma8();
    let mut cur = u64::MAX;
    let mut vals = Vec::new();
    for n in 1..256 {
        let mut img = image::GrayImage::new(frame.width(), (frame.height() + n - 1) / n);
        let mut sum = 0u64;
        for ofs in 0..n {
            let mut h = 0;
            for (dst, src) in img
                .rows_mut()
                .zip(frame.rows().skip(ofs as usize).step_by(n as usize))
            {
                h += 1;
                for (dst, src) in dst.zip(src) {
                    *dst = *src;
                }
            }
            let mut prev = None::<&[u8]>;
            let mut sum0 = 0u64;
            for row in img
                .as_ref()
                .chunks_exact(img.width() as usize)
                .take(h as usize)
            {
                if let Some(prev) = prev {
                    for (a, b) in prev.iter().zip(row.iter()) {
                        sum += (*a ^ *b) as u64
                            // a.abs_diff(*b) as u64
                            ;
                    }
                }
                prev = Some(row);
            }
            sum0 /= (h - 1) as u64;
            sum += sum0;
        }
        sum /= n as u64;
        if sum < cur {
            if n > 4 {
                vals.push(((((cur as f64) / (sum as f64)) * 1000.0) as u64, n));
            } else if n == 2 {
                vals.push(((((cur as f64) / (sum as f64)) * 1000.0 / 3.) as u64, n));
            } else if n == 4 {
                vals.push(((((cur as f64) / (sum as f64)) * 1000.0 / 1.25) as u64, n));
            } else if n == 3 {
                vals.push(((((cur as f64) / (sum as f64)) * 1000.0 / 2.125) as u64, n));
            }
            /*if sum as f64 <= (cur as f64) / 1.4 {
                println!("{n} {sum}!");
            } else {
                println!("{n} {sum}");
            }*/
            cur = sum;
        }
    }
    vals.sort();
    if dbg {
        println!("{vals:?}");
    }
    if let Some((a, b)) = vals.last() {
        (*b >= 10 && *a >= 1300).then_some(*b)
        // Some(*b)
    } else {
        None
    }
}

pub fn rate_candles_frames(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    let mut threads = vec![];
    for j in 1..=16 {
        let src = src.as_ref().to_path_buf();
        let dst = dst.as_ref().to_path_buf();
        threads.push(std::thread::spawn(move || {
            for i in 0..999999 {
                let src1 = src.join(&format!("{:04}.png", i * 16 + j));
                let Ok(_img) = image::open(&src1) else {
                    break;
                };
                let src1 = image::open(src1).unwrap();
                if let Some(n) = rate_candles_frame(src1, false) {
                    candles_frames(&src, &dst, format!("{:04}", i * 16 + j), n as usize);
                }
            }
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
}

pub fn candles_frame(src: impl AsRef<Path>, dst: impl AsRef<Path>, n: usize, ofs: usize) {
    let img = image::open(src).unwrap();
    let frame = img.into_rgb8();
    let mut img = RgbImage::new(frame.width(), frame.height() / n as u32);
    for (dst, src) in img.rows_mut().zip(frame.rows().skip(ofs).step_by(n)) {
        for (dst, src) in dst.zip(src) {
            *dst = *src;
        }
    }
    let img = image::imageops::resize(
        &img,
        frame.width(),
        frame.height(),
        image::imageops::FilterType::Nearest,
    );
    img.save(dst).unwrap();
}

pub fn candles_frames(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    dst_name: impl AsRef<str>,
    n: usize,
) {
    let dst_name = dst_name.as_ref();
    let mut src = src.as_ref().to_owned();
    src.push(format!("{dst_name}.png"));
    for ofs in 0..n {
        let mut dst = dst.as_ref().to_owned();
        dst.push(format!("{dst_name}_{ofs:03}.png"));
        candles_frame(&src, dst, n, ofs)
    }
}

pub fn candles_cycles(dir: impl AsRef<Path>, out: impl AsRef<Path>) {
    let mut threads = vec![];
    const W: u32 = 3904; // 3904 might be fine too
    for j in 0..16 {
        let dir = dir.as_ref().to_path_buf();
        let out = out.as_ref().to_path_buf();
        threads.push(std::thread::spawn(move || {
            let x = "abcdefg";
            let start = 26006;
            for i in 0..26 {
                let Ok(img) = image::open(dir.join(&format!("{:04}.png", i * 16 + j + start)))
                else {
                    break;
                };
                // let w = img.width();
                // let h = img.height();
                let frame = image::DynamicImage::from(image::imageops::resize(
                    &image::imageops::rotate90(&img),
                    img.height(),
                    W,
                    image::imageops::FilterType::Triangle,
                ))
                .into_rgb8();
                let i = i * 16 + j + 1;
                for (o, c) in x.chars().enumerate() {
                    let mut img = RgbImage::new(frame.width(), frame.height() / x.len() as u32);
                    for (dst, src) in img.rows_mut().zip(frame.rows().skip(o).step_by(x.len())) {
                        for (dst, src) in dst.zip(src) {
                            *dst = *src;
                        }
                    }
                    image::imageops::resize(
                        &image::imageops::rotate270(&img),
                        W,
                        2160,
                        image::imageops::FilterType::Nearest,
                    )
                    .save(out.join(format!("{c}{i:04}.png")))
                    .unwrap();
                }
            }
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
}
