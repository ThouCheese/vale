use rkt::data::{Data, FromData, Outcome, Transform, Transformed};
use rkt::http::Status;
use rkt::request::Request;
use std::ops::Deref;
use std::ops::DerefMut;

/// A struct that can be used in `Rocket` routes. If you have some type that implements `Validate`,
/// you can designate in your controller that you want to have a validated instance of that type.
///
/// ### Example
/// ```rust
/// # #![feature(decl_macro)]
/// # 
/// # #[derive(vale::Validate)]
/// # struct User {}
/// # impl rocket::data::FromDataSimple for User { 
/// #     type Error = String;
/// #     fn from_data(req: &rocket::Request, data: rocket::Data) -> rocket::data::Outcome<Self, String> {
/// #         rocket::data::Outcome::Success(Self {})
/// #     }
/// # }
/// #
/// # extern crate rkt as rocket;
/// use vale::Valid;
///
/// #[rocket::post("/user", data = "<user>")]
/// fn update_user(user: Valid<User>) {
///     // user is now validated, this code is not reached if the validation failed
/// }
/// # fn main() {}
/// ```
///
/// It is also possible to nest this type with other wrappers:
///
/// ```rust
/// # #![feature(decl_macro, proc_macro_hygiene)]
/// # #[derive(vale::Validate, serde::Deserialize)]
/// # struct User {}
/// # use vale::{Valid};
/// # use rkt_contrib::json::Json;
/// # extern crate rkt as rocket;
/// #[rocket::post("/user", data = "<user>")]
/// fn update_user(user: Valid<Json<User>>) {
///     let user = user.into_inner().into_inner();
/// }
/// # fn main() {}
/// ```
/// ### Features
/// Requires the `rocket` feature to be enabled
pub struct Valid<T> {
    data: T,
}

impl<T: crate::Validate> Valid<T> {
    fn new(t: T) -> Self {
        Self {
            data: t,
        }
    }

    /// Consumes the `Valid` wrapper and returns the inner item.
    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T: crate::Validate> Deref for Valid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: crate::Validate> DerefMut for Valid<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub enum ValidationError<T> {
    FromDataError(T),
    ValidationError(Vec<String>),
}

impl<'a, T> From<Vec<String>> for ValidationError<T> {
    fn from(s: Vec<String>) -> Self {
        Self::ValidationError(s)
    }
}

impl<T> ValidationError<T> {
    fn from_data_error(t: T) -> Self {
        Self::FromDataError(t)
    }

    fn get_t(self) -> T {
        match self {
            Self::FromDataError(t) => t,
            Self::ValidationError(_) => panic!("Called `get_t` on `ValidationError::FromDataError`"),
        }
    }
}

impl<'a, T: 'a> FromData<'a> for Valid<T>
where
    T: FromData<'a> + crate::Validate
{
    type Error = ValidationError<T::Error>;
    type Owned = T::Owned;
    type Borrowed = T::Borrowed;

    fn transform(r: &Request, d: Data) -> Transform<Outcome<Self::Owned, Self::Error>> {
        match T::transform(r, d) {
            Transform::Owned(out) => Transform::Owned(out.map_failure(|(s, f)| (s, Self::Error::from_data_error(f)))),
            Transform::Borrowed(out) => {
                Transform::Borrowed(out.map_failure(|(s, f)| (s, Self::Error::from_data_error(f))))
            }
        }
    }

    fn from_data(r: &Request, o: Transformed<'a, Self>) -> Outcome<Self, Self::Error> {
        let outcome = match o {
            Transform::Owned(o) => {
                Transform::Owned(o.map_failure(|(s, f)| (s, f.get_t())))
            }
            Transform::Borrowed(o) => {
                Transform::Borrowed(o.map_failure(|(s, f)| (s, f.get_t())))
            }
        };
        let mut inner = match T::from_data(r, outcome) {
            Outcome::Success(s) => s,
            Outcome::Failure((s, f)) => return Outcome::Failure((s, Self::Error::from_data_error(f))),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };
        if let Err(msg) = inner.validate() {
            return Outcome::Failure((Status::BadRequest, msg.into()));
        }
        Outcome::Success(Valid::new(inner))
    }
}

impl<T, U> crate::Validate for U
where
    U: Deref<Target=T> + DerefMut,
    T: crate::Validate,
{
    fn validate(&mut self) -> Result<(), Vec<String>> {
        let t: &mut T = self.deref_mut();
        t.validate()
    }
}
