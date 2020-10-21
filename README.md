# Vale

[![vale on crates.io](https://img.shields.io/crates/v/vale.svg)](https://crates.io/crates/vale)
[![stripe-rust on docs.rs](https://docs.rs/vale/badge.svg)](https://docs.rs/vale)

Vale stands for Valid Entity, and is a simple library that provides entity validation through either annotations, or through a Fluent-style implementation. At the core of the library is the `vale::Validate` trait, which implies that a piece of data can be validated. The library also offers supoort for the `rocket` webframework. If you're interested in adding support for other frameworks, do not hesitate to open a PR!

### Example
This example shows how to derive the validation trait
```rust
use vale::Validate;

#[derive(Validate)]
struct Entity {
    #[validate(gt(0))]
    id: i32,
    #[validate(len_lt(5), with(add_element))]
    others: Vec<i32>,
    #[validate(eq("hello world"))]
    msg: String,
}

fn add_element(v: &mut Vec<i32>) -> bool {
    v.push(3);
    true
}

fn main() {
    let mut e = get_entity();
    if let Err(errs) = e.validate() {
        println!("The following validations failed: {:?}", errs);
    }
}
```
It is also possible to use fluent-style validation:
```rust
struct Entity {
    id: i32,
    others: Vec<i32>,
    msg: String,
}

impl vale::Validate for Entity {
    #[vale::ruleset]
    fn validate(&mut self) -> vale::Result {
        vale::rule!(self.id > 0, "`id` is nonpositive!");
        vale::rule!(self.others.len() < 5, "Too many others");
    }
}
```