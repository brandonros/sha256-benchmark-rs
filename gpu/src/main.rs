use std::time::Duration;
use hex_literal::hex;

use metal::*;
use objc::rc::autoreleasepool;

const PROGRAM: &'static str = include_str!("./kernel.metal");
const NUM_ITERATIONS: usize = usize::pow(2, 15);
const SHA256_HASH_SIZE: usize = 32;
const DISPLAY_INTERVAL: usize = 1000;

fn run_test(device: &Device, kernel_function: &Function, cmd_queue: &CommandQueue, num_iterations: usize) -> usize {
    return autoreleasepool(|| {
        // parameters
        let mut inputs: Vec<u8> = Vec::new();
        let mut input_lengths: Vec<u32> = Vec::new();
        for _ in 0..num_iterations {
            let input1 = b"hello1";
            inputs.extend_from_slice(input1);
            input_lengths.push(input1.len() as u32);
        }
        let outputs: Vec<u8> = vec![0; SHA256_HASH_SIZE * num_iterations];
        // pipeline
        let pipeline_state_descriptor = ComputePipelineDescriptor::new();
        pipeline_state_descriptor.set_thread_group_size_is_multiple_of_thread_execution_width(true);
        pipeline_state_descriptor.set_compute_function(Some(&kernel_function));
        // encode
        let cmd_buffer = cmd_queue.new_command_buffer();
        let cmd_encoder = cmd_buffer.new_compute_command_encoder();
        let pipeline_state = device.new_compute_pipeline_state_with_function(pipeline_state_descriptor.compute_function().unwrap()).unwrap();
        cmd_encoder.set_compute_pipeline_state(&pipeline_state);
        //cmd_encoder.set_threadgroup_memory_length(0, 1024);
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
        let thread_groups_count = MTLSize {
            width: 256,
            height: 1,
            depth: 1,
        };
        let threads_per_threadgroup = MTLSize {
            width: 128,
            height: 1,
            depth: 1,
        };
        cmd_encoder.dispatch_thread_groups(thread_groups_count, threads_per_threadgroup);
        cmd_encoder.end_encoding();
        cmd_buffer.commit();
        cmd_buffer.wait_until_completed();
        // validate
        let encoded_outputs_contents_ptr = encoded_outputs.contents() as *mut u8;
        for i in 0..num_iterations {
            let hash_ptr = unsafe { encoded_outputs_contents_ptr.add(i * SHA256_HASH_SIZE) }; 
            let hash_slice = unsafe { std::slice::from_raw_parts(hash_ptr, SHA256_HASH_SIZE) };
            assert_eq!(hash_slice[..], hex!("91e9240f415223982edc345532630710e94a7f52cd5f48f5ee1afc555078f0ab"));
        }
        // return
        return num_iterations;
    });
}

fn main() {
    let mut total_hashes = 0;
    let mut total_elapsed = Duration::new(0, 0);
    let mut num_iterations = 0;    
    let devices = Device::all();
    let device = devices.iter().find(|device| device.name() == "Apple M1").unwrap();
    let library_compile_options = CompileOptions::new();
    library_compile_options.set_fast_math_enabled(true);
    let library = device.new_library_with_source(PROGRAM, &library_compile_options).unwrap();
    let kernel_function = library.get_function("sha256_kernel", None).unwrap();
    let cmd_queue = device.new_command_queue();
    loop {
        let start = std::time::Instant::now();
        let num_hashes = run_test(&device, &kernel_function, &cmd_queue, NUM_ITERATIONS);
        let elapsed = start.elapsed();
        total_hashes += num_hashes;
        total_elapsed += elapsed;
        num_iterations += 1;
        if num_iterations % DISPLAY_INTERVAL == 0 {
            let hashes_per_second = total_hashes as f64 / total_elapsed.as_secs_f64();
            println!(
                "GPU: After {} iterations: {:.2} hashes per second",
                num_iterations, hashes_per_second
            );
        }
    }
}
