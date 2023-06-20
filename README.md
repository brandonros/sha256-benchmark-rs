# sha256-benchmark-rs
SHA256 CPU + GPU benchmark for Apple Metal

## How to use

```shell
cargo run --release --bin cpu
cargo run --release --bin gpu
```

## Results

```
CPU: After 1000 iterations: 16843376 hashes per second (8 threads)
GPU: After 1000 iterations:  4383297 hashes per second
```
