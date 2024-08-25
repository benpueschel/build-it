# simple-builder

Easily generate builder patterns in Rust.

## Usage

```rust
use simple_builder::Builder;
#[derive(Default, Builder)]
struct MyBuilder {
    a: i32,
    pub b: i32,
    #[skip]
    c: i32,
}
let builder = MyBuilder::default()
    .a(1)
    .b(2);
assert_eq!(builder.a, 1);
assert_eq!(builder.b, 2);
// c is not settable, so it remains the default value
assert_eq!(builder.c, 0);
```
