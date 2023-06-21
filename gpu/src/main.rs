use std::time::Duration;
use hex_literal::hex;

use metal::*;
use objc::rc::autoreleasepool;

const PROGRAM: &'static str = include_str!("./kernel.metal");
const SHA256_HASH_SIZE: usize = 32;
const DISPLAY_INTERVAL: usize = 10;

fn run_test(device: &Device, cmd_queue: &CommandQueue, compute_function: &FunctionRef) -> usize {
    return autoreleasepool(|| {
        // parameters
        let num_thread_groups = 512;
        let num_threads_per_thread_group = 48;
        let thread_chunk_size = 4;
        let num_iterations = num_thread_groups * num_threads_per_thread_group * thread_chunk_size;
        let mut inputs: Vec<u8> = Vec::new();
        let mut input_lengths: Vec<u32> = Vec::new();
        for _ in 0..num_iterations {
            let input1 = b"hello1";
            inputs.extend_from_slice(input1);
            input_lengths.push(input1.len() as u32);
        }
        let outputs: Vec<u8> = vec![0; SHA256_HASH_SIZE * num_iterations];
        // encode
        let cmd_buffer = cmd_queue.new_command_buffer();
        let cmd_encoder = cmd_buffer.new_compute_command_encoder();
        let pipeline_state = device.new_compute_pipeline_state_with_function(compute_function).unwrap();
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
        // input lengths
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
        // thread_chunk_size
        let encoded_thread_chunk_size_array = {
            let thread_chunk_size_array = [thread_chunk_size as u32];
            device.new_buffer_with_data(
                thread_chunk_size_array.as_ptr() as *const _,
                (thread_chunk_size_array.len() * std::mem::size_of::<u32>()) as u64,
                MTLResourceOptions::CPUCacheModeDefaultCache,
            )
        };
        cmd_encoder.set_buffer(3, Some(&encoded_thread_chunk_size_array), 0);
        // dispatch
        cmd_encoder.dispatch_thread_groups(
            MTLSize {
                width: num_thread_groups as u64,
                height: 1,
                depth: 1,
            },
            MTLSize {
                width: num_threads_per_thread_group as u64,
                height: 1,
                depth: 1,
            },
        );
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
    // benchmark stats
    let mut total_hashes = 0;
    let mut total_elapsed = Duration::new(0, 0);
    let mut num_iterations = 0;
    // device
    let devices = Device::all();
    let device = devices.iter().find(|device| device.name() == "Apple M1").unwrap();
    // library
    let library_compile_options = CompileOptions::new();
    library_compile_options.set_fast_math_enabled(true);
    let library = device.new_library_with_source(PROGRAM, &library_compile_options).unwrap();
    // kernel function
    let kernel_function = library.get_function("sha256_kernel", None).unwrap();
    // command queue
    let cmd_queue = device.new_command_queue();
    // pipeline
    let pipeline_state_descriptor = ComputePipelineDescriptor::new();
    pipeline_state_descriptor.set_thread_group_size_is_multiple_of_thread_execution_width(true);
    pipeline_state_descriptor.set_compute_function(Some(&kernel_function));
    // compute function
    let compute_function = pipeline_state_descriptor.compute_function().unwrap();
    // loop
    loop {
        let start = std::time::Instant::now();
        let num_hashes = run_test(&device, &cmd_queue, compute_function);
        let elapsed = start.elapsed();
        total_hashes += num_hashes;
        total_elapsed += elapsed;
        num_iterations += 1;
        if num_iterations % DISPLAY_INTERVAL == 0 {
            let hashes_per_second = total_hashes as f64 / total_elapsed.as_secs_f64();
            println!(
                "GPU: After {} iterations: {:.0} hashes per second",
                num_iterations, hashes_per_second
            );
        }
    }
}
