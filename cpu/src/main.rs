use rayon::{prelude::{IntoParallelRefIterator, ParallelIterator}, ThreadPoolBuilder, ThreadPool};
use sha2::Digest;
use hex_literal::hex;
use std::time::Duration;

const NUM_ITERATIONS: usize = usize::pow(2, 15);
const DISPLAY_INTERVAL: u32 = 1000;

struct Iteration {
    pub input: Vec<u8>
}

fn run_test(pool: &ThreadPool) -> usize {
    // parameters
    let mut iterations = Vec::new();
    for _ in 0..NUM_ITERATIONS {
        let input1 = b"hello1";
        iterations.push(Iteration {
            input: input1.as_slice().to_vec()
        });
    }
    // custom thread pool
    let num_iterations = iterations.len();
    return pool.install(|| {
        // iterate
        let output = iterations.par_iter().map(|iteration| {
            sha2::Sha256::digest(&iteration.input)
        }).collect::<Vec<_>>();
        assert_eq!(output[0][..], hex!("91e9240f415223982edc345532630710e94a7f52cd5f48f5ee1afc555078f0ab"));
        assert_eq!(output[num_iterations - 1][..], hex!("91e9240f415223982edc345532630710e94a7f52cd5f48f5ee1afc555078f0ab"));
        return output.len();
    });
}

fn main() {
    let mut total_hashes = 0;
    let mut total_elapsed = Duration::new(0, 0);
    let mut num_iterations = 0;
    let num_threads = 8;
    let pool = ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap();
    loop {
        let start = std::time::Instant::now();
        let num_hashes = run_test(&pool);
        let elapsed = start.elapsed();
        total_hashes += num_hashes;
        total_elapsed += elapsed;
        num_iterations += 1;

        if num_iterations % DISPLAY_INTERVAL == 0 {
            let hashes_per_second = total_hashes as f64 / total_elapsed.as_secs_f64();
            println!(
                "After {} iterations: {:.2} hashes per second",
                num_iterations, hashes_per_second
            );
        }
    }
}
