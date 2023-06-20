use std::time::Duration;

use metal::*;
use objc::rc::autoreleasepool;

const PROGRAM: &'static str = include_str!("./kernel.metal");
const NUM_ITERATIONS: usize = 32768;
const THREAD_GROUP_WIDTH: usize = 64;
const SHA256_HASH_SIZE: usize = 32;
const DISPLAY_INTERVAL: usize = 1000;

fn run_test() -> (usize, Duration) {
    let start = std::time::Instant::now();
    // parameters
    let mut inputs: Vec<u8> = Vec::new();
    let mut input_lengths: Vec<u32> = Vec::new();
    for _ in 0..NUM_ITERATIONS {
        let input1 = b"hello1";
        inputs.extend_from_slice(input1);
        input_lengths.push(input1.len() as u32);
    }
    let batch_size = input_lengths.len();
    let outputs: Vec<u8> = vec![0; SHA256_HASH_SIZE * batch_size];
    // get device
    let devices = Device::all();
    let device = devices.iter().find(|device| device.name() == "Apple M1").unwrap();
    // load kernel
    let options = CompileOptions::new();
    let library = device.new_library_with_source(PROGRAM, &options).unwrap();
    let kernel = library.get_function("sha256_kernel", None).unwrap();
    // set compute function
    let pipeline_state_descriptor = ComputePipelineDescriptor::new();
    pipeline_state_descriptor.set_compute_function(Some(&kernel));
    // encode
    let cmd_queue = device.new_command_queue();
    let cmd_buffer = cmd_queue.new_command_buffer();
    let cmd_encoder = cmd_buffer.new_compute_command_encoder();
    let pipeline_state = device.new_compute_pipeline_state_with_function(pipeline_state_descriptor.compute_function().unwrap()).unwrap();
    cmd_encoder.set_compute_pipeline_state(&pipeline_state);
    // input
    let encoded_inputs = {
        device.new_buffer_with_data(
            inputs.as_ptr() as *const _,
            inputs.len() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache,
        )
    };
    cmd_encoder.set_buffer(0, Some(&encoded_inputs), 0);
    // input length
    let encoded_input_lengths = {
        device.new_buffer_with_data(
            input_lengths.as_ptr()  as *const _,
            (input_lengths.len() * std::mem::size_of::<u32>()) as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache,
        )
    };
    cmd_encoder.set_buffer(1, Some(&encoded_input_lengths), 0);
    // output
    let encoded_outputs = {
        device.new_buffer_with_data(
            outputs.as_ptr() as *const _,
            outputs.len() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache,
        )
    };
    cmd_encoder.set_buffer(2, Some(&encoded_outputs), 0);
    // dispatch
    let thread_group_size = MTLSize {
        width: THREAD_GROUP_WIDTH as u64,
        height: 1,
        depth: 1,
    };
    let thread_group_count = MTLSize {
        width: ((batch_size + THREAD_GROUP_WIDTH - 1) / THREAD_GROUP_WIDTH) as u64, // ceil(BATCH_SIZE / 64)
        height: 1,
        depth: 1,
    };
    cmd_encoder.dispatch_thread_groups(thread_group_count, thread_group_size);
    cmd_encoder.end_encoding();
    cmd_buffer.commit();
    cmd_buffer.wait_until_completed();
    // return
    (batch_size, start.elapsed())
}

fn main() {
    autoreleasepool(|| {
        let mut total_hashes = 0;
        let mut total_elapsed = Duration::new(0, 0);
        let mut num_iterations = 0;

        loop {
            let (num_hashes, elapsed) = run_test();
            total_hashes += num_hashes; // each run_test computes NUM_ITERATIONS hashes
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
    });
}
