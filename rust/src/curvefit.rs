use argmin::core::{CostFunction, Executor};
use argmin::solver::neldermead::NelderMead;

use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

/* Important shenanigans */

/// The target function
fn target_function_factory_by_ref(x: i32) -> impl Fn(&i32, &i32, &i32) -> i32 {
    move |c, d, e| (-c as f64 / (x - d) as f64 + *e as f64).round() as i32
}

/// The error metric to use; each point is then summed
fn target_function_error(y_expected: &f64, y_actual: &f64) -> f64 {
    ((*y_expected - *y_actual) as f64).powi(2)
}

struct Shenanigans {
    x: Vec<f64>,
    y: Vec<f64>,
}
impl CostFunction for Shenanigans {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, argmin::core::Error> {
        let (c, d, e) = (param[0], param[1], param[2]);
        let residual = self
            .x
            .iter()
            .zip(&self.y)
            .map(|(&x, &y)| {
                let fun = target_function_factory_by_ref(x as i32);
                target_function_error(&y, &(fun(&(c as i32), &(d as i32), &(e as i32)) as f64))
            })
            .sum();
        Ok(residual)
    }
}

#[derive(Debug)]
pub struct CurveFitResult {
    pub coefficients: Vec<i32>,
    pub hamming_distance: f32,
    pub error: f64,
}

/* Boring driver shenanigans */

fn read_points_from_file(path: &str) -> Vec<(i32, i32)> {
    std::fs::read(path)
        .expect("cannot read file")
        .split(|s| *s == b'\n')
        .dropping_back(1)
        .collect_vec()
        .into_iter()
        .map(|line| {
            (&line
                .split(|s| *s == b',')
                .map(|s| String::from_utf8(s.to_vec()).expect("not a string, skill issue"))
                .chunks(2))
                .into_iter()
                .map(|cols| cols.collect_vec())
                .map(|cols| (cols[0].clone(), cols[1].clone()))
                .map(|(x, y)| {
                    (
                        i32::from_str_radix(&x, 10).unwrap(),
                        i32::from_str_radix(&y, 10).unwrap(),
                    )
                })
                .collect::<Vec<_>>()
                .first()
                .expect("not in the correct format")
                .clone()
        })
        .collect_vec()
}

fn hamming_distance(a: &[i32], b: &[i32]) -> f32 {
    assert_eq!(a.len(), b.len());

    a.iter()
        .zip(b)
        .map(|(ax, bx)| (ax.abs_diff(*bx) != 0) as u32)
        .sum::<u32>() as f32
        / a.len() as f32
}

fn curve_fit(points: &Vec<(i32, i32)>) -> (i32, i32, i32) {
    let x = points.iter().map(|(x, _)| *x as f64).collect_vec();
    let y = points.iter().map(|(_, y)| *y as f64).collect_vec();

    let fit = Shenanigans { x, y };

    let nelder = NelderMead::new(vec![
        vec![1000.0, 500.0, -1.0],
        vec![2000.0, 1000.0, 0.0],
        // vec![3058.22, 2444.1, 1.0],
        vec![3000.0, 3000.0, 1.0],
    ])
    .with_sd_tolerance(1.0)
    .unwrap();
    let result = Executor::new(fit, nelder).run().unwrap();

    let best_params = result.state().best_param.clone().unwrap();
    println!("NelderMead Fit: {:?}", best_params);
    (
        best_params[0] as i32,
        best_params[1] as i32,
        best_params[2] as i32,
    )
}

fn lock_in_candidates(
    points: Vec<(i32, i32)>,
    c: (i32, i32),
    d: (i32, i32),
    e: (i32, i32),
    hamming_threshold: f32,
) -> Vec<(i32, i32, i32, f64)> {
    let (c_min, c_max) = c;
    let (d_min, d_max) = d;
    let (e_min, e_max) = e;

    let parallelism = std::thread::available_parallelism().unwrap();
    let product_length =
        (c_max - c_min + 1) as usize * (d_min - d_max + 1) as usize * (e_max - e_min + 1) as usize;
    println!("Will split into {parallelism} amount of tasks");

    let ys = points.iter().map(|(_, y)| y).cloned().collect_vec();

    (e_min..=e_max)
        .array_chunks::<3>()
        .par_bridge()
        .map(|vec_of_range| {
            let new_e_range = vec_of_range[0]..=vec_of_range[2];
            let product = [c_min..=c_max, d_min..=d_max, new_e_range]
                .into_iter()
                .multi_cartesian_product()
                .map(|p| (p[0], p[1], p[2]));
            let to_satisfy = points.iter().map(|(x, _)| {
                |c: &i32, d: &i32, e: &i32| target_function_factory_by_ref(*x)(c, d, e)
            });
            product
                .chunks(product_length / parallelism)
                .into_iter()
                .flat_map(|chunks| {
                    chunks
                        .collect_vec()
                        .par_iter()
                        .filter_map(|(c, d, e)| {
                            let results = to_satisfy
                                .clone()
                                .map(|func| func(c, d, e))
                                .collect::<Vec<_>>();
                            if hamming_distance(&results, &ys) <= hamming_threshold {
                                Some((
                                    *c,
                                    *d,
                                    *e,
                                    results
                                        .into_iter()
                                        .zip(ys.clone())
                                        .map(|(expect, actual)| {
                                            target_function_error(
                                                &(expect as f64),
                                                &(actual as f64),
                                            )
                                        })
                                        .sum(),
                                ))
                            } else {
                                None
                            }
                        })
                        .collect_vec_list()
                })
                .flatten()
                .collect_vec()
        })
        .flatten()
        .collect::<Vec<_>>()
}

pub fn curvefit(points_path: &str) -> CurveFitResult {
    let points = read_points_from_file(points_path);
    let (close_c, close_d, close_e) = curve_fit(&points);

    let (best_ratio, (best_c, best_d, best_e, lowest_err)) = (0..points.len())
        .filter_map(|i| {
            let ratio = i as f32 / points.len() as f32;
            if let Some(candidate) = lock_in_candidates(
                points.clone(),
                (close_c - 500, close_c + 500),
                (close_d - 500, close_d + 500),
                (close_e - 5, close_e + 5),
                ratio,
            )
            .into_iter()
            .min_by(|(_, _, _, err_a), (_, _, _, err_b)| err_a.partial_cmp(err_b).unwrap())
            {
                Some((ratio, candidate))
            } else {
                None
            }
        })
        .take(1)
        .collect_vec()[0];

    CurveFitResult {
        coefficients: vec![best_c, best_d, best_e],
        hamming_distance: best_ratio,
        error: lowest_err,
    }
}

#[cfg(test)]
mod tests {
    use super::hamming_distance;

    #[test]
    fn test_hamming_distance() {
        assert_eq!(hamming_distance(&[1, 2, 3, 4], &[1, 2, 3, 4]), 0.0);
        assert_eq!(hamming_distance(&[1, 2, 3, 4], &[0, 1, 2, 3]), 1.0);
        assert_eq!(hamming_distance(&[1, 2, 3, 4], &[0, 2, 3, 4]), 0.25);
    }
}
