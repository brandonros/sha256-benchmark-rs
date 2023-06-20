use rayon::{prelude::{IntoParallelRefIterator, ParallelIterator}, ThreadPoolBuilder};
use sha2::Digest;
use std::time::Duration;

const NUM_ITERATIONS: usize = 32768;
const SHA256_HASH_SIZE: usize = 32;
const DISPLAY_INTERVAL: u32 = 1000;

struct Iteration {
    pub input: Vec<u8>,
    pub input_length: u32
}

fn run_test() -> (usize, Duration) {
    // parameters
    let mut iterations = Vec::new();
    for _ in 0..NUM_ITERATIONS {
        let input1 = b"hello1";
        iterations.push(Iteration {
            input: input1.as_slice().to_vec(),
            input_length: input1.len() as u32
        });
    }
    // custom thread pool
    let num_threads = 8;
    let pool = ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap();
    let (num_hashes, elapsed) = pool.install(|| {
        // iterate
        let start = std::time::Instant::now();
        let output = iterations.par_iter().map(|iteration| {
            sha2::Sha256::digest(&iteration.input)
        }).collect::<Vec<_>>();
        let elapsed = start.elapsed();
        (output.len(), elapsed)
    });
    
    (num_hashes, elapsed)
}

fn main() {
    let mut total_hashes = 0;
    let mut total_elapsed = Duration::new(0, 0);
    let mut num_iterations = 0;

    loop {
        let (num_hashes, elapsed) = run_test();
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
