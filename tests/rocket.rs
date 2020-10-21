#![feature(decl_macro)]

extern crate rkt as rocket;

use rkt_contrib::json::Json;
use rkt::http::Status;

#[derive(vale::Validate)]
#[derive(serde::Serialize, serde::Deserialize)]
struct Struct {
    #[validate(gt(10))]
    value: u32,
    #[validate(len_gt(3))]
    string: String,
    #[validate(eq(true))]
    boolean: bool,
    #[validate(with(is_even))]
    even_value: i32,
    #[validate(trim, len_lt(10), to_lower_case)]
    transformer: String,
    #[validate(len_lt(10), trim)]
    transfailer: String,
}

fn is_even(arg: &mut i32) -> bool {
    *arg % 2 == 0
}

fn valid_struct() -> Struct {
    Struct {
        value: 12,
        string: "String".to_string(),
        boolean: true,
        even_value: 2,
        transformer: "hello".to_string(),
        transfailer: "hello".to_string(),
    }
}

#[rocket::post("/", data = "<to_validate>")]
fn route(to_validate: vale::Valid<Json<Struct>>) -> rkt_contrib::json::Json<Struct> {
    rkt_contrib::json::Json(to_validate.into_inner().into_inner())
}

fn test_rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", rocket::routes![route])
}

#[test]
fn test_valid() {
    let s = valid_struct();

    let rocket = test_rocket();
    let client = rkt::local::Client::new(rocket).unwrap();
    let resp = client
        .post("/")
        .body(serde_json::to_string(&s).unwrap())
        .dispatch();
    assert_eq!(resp.status(), Status::Ok)
}

#[test]
fn test_too_small() {
    let mut s = valid_struct();
    s.value = 8;

    let rocket = test_rocket();
    let client = rkt::local::Client::new(rocket).unwrap();
    let resp = client
        .post("/")
        .body(serde_json::to_string(&s).unwrap())
        .dispatch();
    assert_eq!(resp.status(), Status::BadRequest)
}

#[test]
fn test_too_short() {
    let mut s = valid_struct();
    s.string = "hi".to_string();

    let rocket = test_rocket();
    let client = rkt::local::Client::new(rocket).unwrap();
    let resp = client
        .post("/")
        .body(serde_json::to_string(&s).unwrap())
        .dispatch();
    assert_eq!(resp.status(), Status::BadRequest)
}

#[test]
fn test_too_false() {
    let mut s = valid_struct();
    s.boolean = false;

    let rocket = test_rocket();
    let client = rkt::local::Client::new(rocket).unwrap();
    let resp = client
        .post("/")
        .body(serde_json::to_string(&s).unwrap())
        .dispatch();
    assert_eq!(resp.status(), Status::BadRequest)
}

#[test]
fn transform() {
    let mut s = valid_struct();
    s.transformer = "     CAST ME       ".to_string();

    let rocket = test_rocket();
    let client = rkt::local::Client::new(rocket).unwrap();
    let resp = client
        .post("/")
        .body(serde_json::to_string(&s).unwrap())
        .dispatch();
    assert_eq!(resp.status(), Status::Ok)
}

#[test]
fn transfail() {
    let mut s = valid_struct();
    s.transfailer = "     CAST ME       ".to_string();

    let rocket = test_rocket();
    let client = rkt::local::Client::new(rocket).unwrap();
    let mut resp = client
        .post("/")
        .body(serde_json::to_string(&s).unwrap())
        .dispatch();
    println!("{:?}", resp.body_string());
    assert_eq!(resp.status(), Status::BadRequest);
}
