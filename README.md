# simple-builder

Easily generate builder patterns in Rust.

## Usage

```rust
use build_it::Builder;
#[derive(Default, Builder)]
struct MyAwesomeStruct {
    name: Option<String>,
    pub age: Option<u32>,
    #[skip]
    address: String,
    #[skip]
    pub phone: Option<String>,
}
let builder = MyAwesomeStruct::default()
    .name("Alice".to_string())
    .age(42);
    // Note that `address` and `phone` do not have builder methods because of the #[skip]
    // attribute.
assert_eq!(builder.name, Some("Alice".to_string()));
assert_eq!(builder.age, Some(42));

// These fields are skipped, so they're value will still be the default value.
assert_eq!(builder.address, String::default());
assert_eq!(builder.phone, None);
```
