# skyline-rs

A Rust library for working with [Skyline](https://github.com/shadowninja108/Skyline) to allow you to write game code modification
for Nintendo Switch games using Rust.

For `no_std` use, disable the `std` feature (enabled by default).

Suggested for use with [`cargo-skyline`](https://github.com/jam1garner/cargo-skyline).

Example:

```rust
extern "C" fn test() -> u32 {
    2
}

#[skyline::hook(replace = test)]
fn test_replacement() -> u32 {

    let original_test = original!();

    let val = original_test();

    println!("[override] original value: {}", val); // 2

    val + 1
}

#[skyline::main(name = "skyline_rs_template")]
pub fn main() {
    println!("Hello from Skyline Rust Plugin!");

    skyline::install_hook!(test_replacement);

    let x = test();

    println!("[main] test returned: {}", x); // 3
}
```
