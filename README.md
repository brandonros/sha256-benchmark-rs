# sha256-benchmark-rs
SHA256 CPU + GPU benchmark for Apple Metal

## How to use

```shell
cargo run --release --bin cpu
cargo run --release --bin gpu
```

## Results

```
CPU: After 1000 iterations: 10873127 hashes per second
GPU: After 1000 iterations:  1029608 hashes per second
```

## hashcat results

```
---------------------------
* Hash-Mode 1400 (SHA2-256)
---------------------------

Speed.#1.........:   447.2 MH/s (64.33ms) @ Accel:512 Loops:256 Thr:32 Vec:1
device_info->kernel_accel_dev, 512
  device_info->kernel_loops_dev, 256
  device_info->kernel_threads_dev, 32
  device_info->vector_width_dev 1
```
