use std::cmp;
use std::fmt;

use structopt::StructOpt;

const MAX_SETS: u32 = 6;
const MAX_REPS: u32 = 5;

type DistributionFn = fn(f32, f32) -> f32;

#[derive(Debug)]
enum Distribution {
    Classic,
    Sin,
    Linear,
}

fn parse_sets(s: &str) -> Result<u32, String> {
    let value =
    match u32::from_str_radix(s, 10) {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    if value > MAX_SETS {
        return Err(format!("Must be {} or less", MAX_SETS));
    }

    Ok(value)
}

fn parse_dist(s: &str) -> Result<Distribution, String> {
    match s {
        "c" | "classic" => Ok(Distribution::Classic),
        "s" | "sin" => Ok(Distribution::Sin),
        "l" | "linear" => Ok(Distribution::Linear),
        _ => Err(format!("Must be one of: classic sin linear")),
    }
}

#[derive(StructOpt, Debug)]
#[structopt(about, author)]
struct Options {
    /// The bar weight.
    #[structopt(short, long, default_value = "45")]
    bar: u32,

    /// The number of sets.
    #[structopt(short, long, default_value = "4", parse(try_from_str = parse_sets))]
    sets: u32,

    #[structopt(short, long, default_value = "sin", parse(try_from_str = parse_dist))]
    dist: Distribution,

    /// Sets the weight of the work set.  Must be great than or equal to the bar weight.
    work_set: u32,
}

struct Set {
    /// The total weight of the set
    weight: u32,
    /// The number of repititions
    reps: u32,
    /// The number of times the set is repeated
    repeat: u32,
}

impl fmt::Display for Set {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}x{}", self.weight, self.reps, self.repeat)
    }
}

impl fmt::Debug for Set {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}x{}", self.weight, self.reps, self.repeat)
    }
}

fn main() {
    let options = Options::from_args();
    let set_weights = match options.dist {
        Distribution::Classic => block_weights_classic(options.bar, options.work_set, options.sets),
        Distribution::Sin => block_weights_dist(options.bar, options.work_set, options.sets, dist_sin),
        Distribution::Linear => block_weights_dist(options.bar, options.work_set, options.sets, dist_linear),
    };
    let sets = get_sets(&set_weights);
    print_sets(options.bar, &sets);
}

fn print_sets(base: u32, sets: &[Set]) {
    for set in sets {
        println!("{:>7} {:?}", set.to_string(), get_plates(set.weight - base));
    }
}

fn block_weights_classic(min: u32, max: u32, sets: u32) -> Vec<u32> {
    let mut rv = Vec::new();
    let delta = round_up_5((max - min) / (sets - 1));

    for set in 0..sets {
        // This ensures weight for second to last set != last set
        let set_max = match set {
            n if n == sets - 1 => max,
            _ => max - 5,
        };

        let weight = cmp::min(min + delta * set, set_max);

        rv.push(weight);
    }

    rv
}

fn get_sets(set_weights: &[u32]) -> Vec<Set> {
    let num_sets = set_weights.len();

    set_weights
        .iter()
        .enumerate()
        .map(|(set_idx, weight)| Set{
            weight: *weight,
            reps: get_reps(set_idx as u32, num_sets as u32),
            repeat: get_set_repeats(set_idx as u32, num_sets as u32),
        })
        .collect()
}

fn round_up_5(x: u32) -> u32 {
    (x + 4) / 5 * 5
}

fn get_reps(set: u32, sets: u32) -> u32 {
    let max = MAX_REPS;
    let upper_bound = sets - 1;

    match set {
        n if n == upper_bound => max,
        n => cmp::max(max - n, 1),
    }
}

fn get_set_repeats(set: u32, sets: u32) -> u32 {
    let lower_bound = 0;
    let upper_bound = sets - 1;

    match set {
        n if n == upper_bound => 3,
        n if n == lower_bound => 2,
        _ => 1,
    }
}

fn get_plates(weight: u32) -> Vec<f64> {
    if weight == 0 {
        return Vec::new();
    }

    let available_plates = vec![45.0, 35.0, 25.0, 10.0, 5.0, 5.0, 2.5];
    let mut required_plates: Vec<f64> = Vec::new();
    let mut available_plates_iter = available_plates.iter();

    let weight = weight as f64 / 2.0;
    let mut next_sum: f64 = 0.0;

    // Cap iterations to prevent infinite loop in case of no solution
    for _ in 0..10 {
        let sum = next_sum;

        // Eliminate available plates until we find one that doesn't exceed our desired weight
        while let Some(plate) = available_plates_iter.next() {
            next_sum = sum + plate;

            if next_sum <= weight {
                required_plates.push(*plate);
                break;
            }
        }

        // Are we done?
        if next_sum == weight {
            return required_plates;
        } else if next_sum > weight {
            panic!("sum exceeds weight");
        }
    }

    panic!("no solution found");
}

fn dist_sin(x: f32, delta_normalized: f32) -> f32 {
    delta_normalized * (std::f32::consts::PI * x / delta_normalized / 2.0).sin()
}

fn dist_linear(x: f32, _: f32) -> f32 {
    x
}

fn weights(min: u32, max: u32, num_sets: u32) -> Vec<u32> {
    block_weights_dist(min, max, num_sets, dist_linear)
}

fn block_weights_dist(min: u32, max: u32, num_sets: u32, dist: DistributionFn) -> Vec<u32> {
    let delta = max - min;
    let delta_normalized = delta as f32 / 5.0;
    let increment = delta_normalized / (num_sets - 1) as f32;

    (0..num_sets)
        // Create even spread
        .map(|x| x as f32 * increment)
        // Distribute
        .map(|x| dist(x, delta_normalized))
        // Convert
        .map(|x| x as u32)
        // Denormalize
        .map(|x| x * 5)
        // Offset
        .map(|x| x + min)
        .collect()
}

#[cfg(test)]
mod tests {
    mod round_up_5 {
        use super::super::*;

        #[test]
        fn compare() {
            assert_eq!(round_up_5(0), 0);
            assert_eq!(round_up_5(1), 5);
            assert_eq!(round_up_5(2), 5);
            assert_eq!(round_up_5(3), 5);
            assert_eq!(round_up_5(4), 5);
            assert_eq!(round_up_5(5), 5);
            assert_eq!(round_up_5(6), 10);
        }
    }

    mod block_weights_classic {
        use super::super::*;

        #[test]
        fn dist_lin() {
            assert_eq!(
                weights(45, 85, 5),
                vec![45, 55, 65, 75, 85],
            );

            assert_eq!(
                weights(45, 105, 5),
                vec![45, 60, 75, 90, 105],
            );
        }

        #[test]
        fn dist_lin_fractional() {
            assert_eq!(
                weights(45, 90, 5),
                vec![45, 60, 75, 85, 90],
            );
        }

        #[test]
        fn dist_sin_fractional_delta() {
            assert_eq!(
                block_weights_dist(45, 90, 5, dist_sin),
                vec![45, 60, 75, 85, 90],
            );
        }

        #[test]
        fn dist_sin_large() {
            assert_eq!(
                block_weights_dist(65, 185, 4, dist_sin),
                vec![],
            );
        }

        #[test]
        fn dist_linear_large() {
            assert_eq!(
                block_weights_dist(65, 185, 4, dist_linear),
                vec![],
            );
        }

        #[test]
        fn typ() {
            assert_eq!(
                block_weights_classic(45, 85, 5),
                vec![45, 55, 65, 75, 85],
            );
            assert_eq!(
                block_weights_classic(45, 105, 5),
                vec![45, 60, 75, 90, 105],
            );
        }

        #[test]
        fn fractional_delta() {
            assert_eq!(
                block_weights_classic(45, 90, 5),
                vec![45, 60, 75, 85, 90],
            );
            assert_eq!(
                block_weights_classic(45, 95, 5),
                vec![45, 60, 75, 90, 95],
            );
            assert_eq!(
                block_weights_classic(45, 100, 5),
                vec![45, 60, 75, 90, 100],
            );
        }
    }

    mod get_sets {
        use super::super::*;

        #[test]
        fn basic() {
            assert_eq!(
                format!("{:?}", get_sets(&block_weights_classic(45, 85, 5))),
                "[45x5x2, 55x4x1, 65x3x1, 75x2x1, 85x5x3]"
            );
        }
    }

    mod get_reps {
        use super::super::*;

        #[test]
        fn min() {
            assert_eq!(get_reps(0, 5), 5);
        }

        #[test]
        fn mid_nominal() {
            assert_eq!(get_reps(1, 5), 4);
            assert_eq!(get_reps(2, 5), 3);
            assert_eq!(get_reps(3, 5), 2);
            assert_eq!(get_reps(4, 6), 1);
        }

        #[test]
        fn mid_min() {
            assert_eq!(get_reps(5, 7), 1);
            assert_eq!(get_reps(5, 9), 1);
        }

        #[test]
        fn max() {
            assert_eq!(get_reps(4, 5), 5);
        }
    }

    mod get_set_repeats {
        use super::super::*;

        #[test]
        fn min() {
            assert_eq!(get_set_repeats(0, 5), 2);
        }

        #[test]
        fn max() {
            assert_eq!(get_set_repeats(4, 5), 3);
            assert_eq!(get_set_repeats(0, 1), 3);
        }

        #[test]
        fn mid() {
            assert_eq!(get_set_repeats(3, 5), 1);
        }
    }

    mod get_plates {
        use super::super::*;

        #[test]
        fn min() {
            assert_eq!(get_plates(5), vec!(2.5));
        }

        #[test]
        fn max() {
            assert_eq!(get_plates(255), vec!(45.0, 35.0, 25.0, 10.0, 5.0, 5.0, 2.5));
        }

        #[test]
        fn mid() {
            assert_eq!(get_plates(90), vec!(45.0));
            assert_eq!(get_plates(30), vec!(10.0, 5.0));
        }

        #[test]
        #[should_panic(expected = "sum exceeds weight")]
        fn too_small() {
            get_plates(4);
        }

        #[test]
        #[should_panic(expected = "no solution found")]
        fn not_multiple_of_five() {
            get_plates(6);
        }

        #[test]
        #[should_panic(expected = "no solution found")]
        fn too_large() {
            get_plates(301);
        }
    }
}
