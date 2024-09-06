# simple-builder

Easily generate builder patterns in Rust.

## Usage

```rust
use build_it::Builder;
#[derive(Default, Builder)]
struct MyAwesomeStruct {
    name: Option<String>,
    pub age: Option<u32>,
    #[build_it(skip)]
    address: String,
    #[build_it(skip)]
    pub phone: Option<String>,
}
let builder = MyAwesomeStruct::default()
    .name("Alice".to_string())
    .age(42);
// Note that `address` and `phone` do not have builder
// methods because of the #[build_it(skip)] attribute.
assert_eq!(builder.name, Some("Alice".to_string()));
assert_eq!(builder.age, Some(42));

// These fields are skipped, so they're value will still be the default value.
assert_eq!(builder.address, String::default());
assert_eq!(builder.phone, None);
```

The `#[build_it(rename = "new_name")]` attribute can be used to rename the builder
method. In this case, the builder method will be called `new_name` instead of `renamed`:
```rust
struct MyAwesomeStruct {
    #[build_it(rename = "new_name")]
    renamed: Option<String>,
}

let builder = MyAwesomeStruct::default()
    .new_name("Alice".to_string());
```

The `#[build_it(into)]` attribute can be used to allow the builder method to accept
types that can be converted into the field type. In this case, the builder method will
accept a `&str` instead of a `String`:
```rust
struct MyAwesomeStruct {
    #[build_it(into)]
    name_into: Option<String>,
}

let builder = MyAwesomeStruct::default()
    .name_into("Alice");
```

The `#[build_it(into)]` attribute can also be used on the struct itself to allow the
builder to accept `Into` implementations for all fields:
```rust
#[derive(Builder)]
#[build_it(into)]
struct MyAwesomeStruct {
    name: Option<String>,
    language: Option<String>,
}
let builder = MyAwesomeStruct::default()
    .name("Alice")
    .language("Rust");
```
