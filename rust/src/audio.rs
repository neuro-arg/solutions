pub fn meaning_of_life() {
    let src = "../../yt/out.wav";
    let dst = "out.wav";
    let wav = hound::WavReader::open(src).unwrap();
    let spec = wav.spec();
    let samples = wav
        .into_samples::<i16>()
        .skip(888144 * 2)
        .take((41464889 - 888144) * 2)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    // let sec = spec.channels as usize * spec.sample_rate as usize;
    // very bad code for determening the loop period
    // (normally you'd use a completely different method for this but i'm lazy like that)
    /*let mut best = sec * 24;
    let mut best_score: u64 = u64::MAX;
    for period in ((sec as f64 * 24.61f64) as usize)..((sec as f64 * 24.67f64) as usize) {
        let mut sum = vec![0i64; period];
        for chunk in samples.chunks_exact(period).take(5) {
            for (x, y) in sum.iter_mut().zip(chunk) {
                *x += *y as i64;
            }
        }
        let mut score = 0u64;
        for chunk in samples.chunks_exact(period).take(5) {
            for (x, y) in sum.iter().zip(chunk) {
                score += x.abs_diff(*y as i64);
            }
        }
        if score < best_score {
            println!("{} is better with {score}", period);
            best_score = score;
            best = period;
        }
    }*/
    assert!(spec.sample_rate == 48000);
    let period = 2362976;
    /*let mut freq = vec![HashMap::<i16, usize>::new(); period];
    for chunk in samples.chunks_exact(period) {
        for (x, y) in freq.iter_mut().zip(chunk) {
            *x.entry(*y).or_default() += 1;
        }
    }
    let samples: Vec<_> = freq
        .into_iter()
        .map(|x| x.into_iter().map(|(k, v)| (v, k)).max().unwrap().1)
        .collect();*/
    let mut sum = vec![0i64; period];
    for chunk in samples.chunks_exact(period) {
        for (x, y) in sum.iter_mut().zip(chunk) {
            *x += *y as i64;
        }
    }
    /*let samples: Vec<_> = sum
    .into_iter()
    .map(|x| i16::try_from((x as f64 / (samples.len() / period) as f64) as i64).unwrap_or(0))
    .collect();*/
    /*for chunk in samples.chunks_exact_mut(period) {
        for (x, y) in chunk.iter_mut().zip(&avg) {
            *x = (*x).saturating_sub(*y);
        }
    }*/
    let mut out = hound::WavWriter::create(dst, spec).unwrap();
    let mut writer = out.get_i16_writer(samples.len().try_into().unwrap());
    for sample in samples {
        writer.write_sample(sample);
    }
    writer.flush().unwrap();
    out.finalize().unwrap();
}

pub fn filtered() {
    let src = "../../yt/right.wav";
    // let src = "f.wav";
    // let dst = "out.wav";
    let wav = hound::WavReader::open(src).unwrap();
    let spec = wav.spec();
    let samples = wav
        .into_samples::<i16>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut comps: Vec<_> = samples
        .iter()
        .map(|x| num::Complex::from((*x as f64) / i16::MAX as f64 * 2.))
        .collect();
    let mut fft = rustfft::FftPlanner::new();
    println!("{}", spec.sample_rate);
    let planner = fft.plan_fft_forward(spec.sample_rate as usize);
    comps.resize(
        (comps.len() + planner.len() - 1) / planner.len() * planner.len(),
        num::Complex::default(),
    );
    planner.process(&mut comps);
    let mut img = image::DynamicImage::new_luma8(
        (planner.len() / 2) as u32,
        (comps.len() / planner.len()) as u32,
    );
    let comps: Vec<_> = comps
        .chunks(planner.len() / 2)
        .enumerate()
        .filter_map(|(i, x)| (i % 2 == 0).then_some(x))
        .flatten()
        .collect();
    for (img_row, srow) in img
        .as_mut_luma8()
        .unwrap()
        .chunks_mut(planner.len() / 2)
        .zip(comps.chunks(planner.len() / 2))
    {
        let noise = 0.; // srow[1800].norm();
        for (i, (pix, comp)) in img_row.iter_mut().zip(srow.iter()).enumerate() {
            *pix = (((comp.norm() - (noise * (0.7 + 0.015f64 * (planner.len() / 2 - i) as f64)))
                .max(0.)
                * 512.0)
                .sqrt() as i32)
                .clamp(0, 0xFF)
                .try_into()
                .unwrap_or(0xFF);
        }
    }
    /*let mut img = img.into_rgb8();
    for (i, (img_row, srow)) in img
        .chunks_mut(planner.len() / 2 * 3)
        .zip(comps.chunks(planner.len() / 2))
        .enumerate()
    {
        let mut marked = vec![false; srow.len()];
        let mut done = false;
        let noise = srow[1800].norm();
        for i in (0..srow.len()).rev() {
            if marked[i] {
                continue;
            }
            if (((srow[i].norm() - (noise * (1.0 + 0.01f64 * (planner.len() / 2 - i) as f64)))
                .max(0.)
                * 512.0)
                .sqrt()
                .clamp(0., 255.)) as i32
                <= 50
            {
                continue;
            }
            for div in 50..=(i / 4) {
                let mut seq = Vec::new();
                let mut j = div * 2;
                let mut good = true;
                while j <= i {
                    seq.push(srow[j]);
                    j += div;
                    if seq.len() > 1
                        && (((seq[seq.len() - 2].norm()
                            - (noise * (1.0 + 0.01f64 * (planner.len() / 2 - i) as f64)))
                            .max(0.)
                            * 512.0)
                            .sqrt()
                            .clamp(0., 255.))
                            <= (((seq[seq.len() - 1].norm()
                                - (noise * (1.0 + 0.01f64 * (planner.len() / 2 - i) as f64)))
                                .max(0.)
                                * 512.0)
                                .sqrt()
                                .clamp(0., 255.))
                    {
                        good = false;
                        break;
                    }
                }
                if seq.len() < 3 {
                    continue;
                }
                if good {
                    let mut j = div;
                    while j <= i {
                        marked[j] = true;
                        j += div;
                    }
                    break;
                }
            }
        }
        for i in marked
            .into_iter()
            .enumerate()
            .filter_map(|(i, x)| x.then_some(i))
        {
            img_row[i * 3] = 0xFF;
        }
    }*/
    image::DynamicImage::from(img)
        // .rotate270()
        .save("right.png")
        .unwrap();
    /*let mut out = hound::WavWriter::create(dst, spec).unwrap();
    let mut writer = out.get_i16_writer(samples.len().try_into().unwrap());
    for sample in samples {
        writer.write_sample(sample);
    }
    writer.flush().unwrap();
    out.finalize().unwrap();*/
}

pub fn geiger() {
    let mut data: Vec<u32> = std::fs::read_to_string("geiger.txt")
        .unwrap()
        .split('\n')
        .filter(|x| !x.is_empty())
        .map(|x| x.parse().unwrap())
        .collect();
    let dst = "out.wav";
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut out = hound::WavWriter::create(dst, spec).unwrap();
    let mut writer = out.get_i16_writer((data.last().unwrap() + 1) * 1470);
    data.sort();
    let mut data = data.as_slice();
    for i in 0..=*data.last().unwrap() {
        let mut w = false;
        while !data.is_empty() && i >= *data.first().unwrap() {
            w = true;
            data = &data[1..];
        }
        let sample = if w { 32767i16 } else { 0i16 };
        for x in 0..1470 {
            writer.write_sample(if x % 2 == 1 { -sample } else { sample });
        }
    }
    writer.flush().unwrap();
    out.finalize().unwrap();
}
