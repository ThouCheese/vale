#![cfg_attr(feature = "rocket", feature(decl_macro, proc_macro_hygiene))]
#![forbid(unsafe_code, missing_docs)]

//! Vale stands for Valid Entity, and is a simple library that provides entity validation through
//! either annotations, or through a Fluent-style implementation. At the core of the library is the
//! `vale::Validate` trait, which implies that a piece of data can be validated. The library also
//! offers supoort for the `rocket` webframework. If support for more webframeworks is desired, it
//! should be fairly trivial to implement support for those frameworks.
//!
//! ### Example
//! This example shows how to derive the validation trait
//! ```rust
//! # fn get_entity() -> Entity {
//! #     Entity::default()
//! # }
//! use vale::Validate;
//!
//! # #[derive(Default)]
//! #[derive(Validate)]
//! struct Entity {
//!     #[validate(gt(0))]
//!     id: i32,
//!     #[validate(len_lt(5), with(add_element))]
//!     others: Vec<i32>,
//!     #[validate(eq("hello world"))]
//!     msg: String,
//! }
//!
//! fn add_element(v: &mut Vec<i32>) -> bool {
//!     v.push(3);
//!     true
//! }
//!
//! fn main() {
//!     let mut e = get_entity();
//!     if let Err(errs) = e.validate() {
//!         println!("The following validations failed: {:?}", errs);
//!     }
//! }
//! ```
//!
//! It is also possible to use fluent-style validation:
//! ```rust
//! struct Entity {
//!     id: i32,
//!     others: Vec<i32>,
//!     msg: String,
//! }
//!
//! impl vale::Validate for Entity {
//!     #[vale::ruleset]
//!     fn validate(&mut self) -> vale::Result {
//!         vale::rule!(self.id > 0, "`id` is nonpositive!");
//!         vale::rule!(self.others.len() < 5, "Too many others");
//!     }
//! }
//! ```

#[cfg(feature = "rocket")]
mod rocket_impls;

#[cfg(feature = "rocket")]
pub use rocket_impls::Valid;
/// The rule macro is used to create new rules that dictate how a field of the validated entity
/// should be tranformed and validated.
///
/// ### Example
/// ```rust
/// struct MyStruct {
///     a: i32,
/// }
///
/// impl vale::Validate for MyStruct {
///     #[vale::ruleset]
///     fn validate(&mut self) -> vale::Result {
///         vale::rule!(self.a == 3, "A was not three!");
///         // if the second argument is omitted, a standard error message is returned.
///         vale::rule!(self.a % 3 == 0);
///     }
/// } 
/// ```
pub use vale_derive::rule;
/// Use this macro to annotate yout implementation of `vale::Validate` for your struct to help
/// write the error reporting boilerplate for you. See the documentation of `vale::rule` for usage
/// examples.
pub use vale_derive::ruleset;
/// A proc macro used to implement `Validate` automatically for a struct.
/// 
/// There are a couple of options for validating a structure. The are listed below:
///
/// * `lt`: Check if the value is less than the provided argument,
/// * `eq`: check if the value is equal to the provided argument,
/// * `gt`: check if the value is greater than the provided argument,
/// * `neq`: check if the `len()` of the value is not equal to the provided argument,
/// * `len_lt`: Check if the `len()` of the value is less than the provided argument,
/// * `len_eq`: check if the `len()` of the value is equal to the provided argument,
/// * `len_gt`: check if the `len()` of the value is greater than the provided argument,
/// * `len_neq`: check if the `len()` of the value is not equal to the provided argument,
/// * `with`: Rrn the provided function to perform validation,
/// * `trim`: always succeeds, and trims the string that is inputted,
/// * `to_lower_case`: convert the provided value to lowercase.
///
/// ### Example
/// ```rust,no_run
/// # use vale::Validate;
/// #[derive(vale::Validate)]
/// struct Struct {
///     #[validate(gt(10))]
///     value: u32,
///     #[validate(len_gt(3))]
///     string: String,
///     #[validate(eq(true))]
///     boolean: bool,
///     #[validate(with(is_even))]
///     even_value: i32,
///     #[validate(trim, len_lt(10), to_lower_case)]
///     transformer: String,
///     #[validate(len_lt(10), trim)]
///     transfailer: String,
/// }
///
/// fn is_even(num: &mut i32) -> bool {
///     *num % 2 == 0
/// }
/// ```
pub use vale_derive::Validate;

/// A type alias for the `Result` returned by the `Validate::validate` function.
pub type Result = std::result::Result<(), Vec<String>>;

/// The core trait of this library. Any entity that implements `Validate` can be validated by
/// running the `validate` function. This will either return an `Ok(())`, or an `Err` containing a
/// list of errors that were triggered during validation. It is also possible for `validate` to
/// perform tranformations on the entity that is being validated.
pub trait Validate {
    /// Performs the validation.
    fn validate(&mut self) -> Result;
}

