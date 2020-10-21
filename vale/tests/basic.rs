use vale::Validate;

#[derive(Validate)]
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

#[test]
fn test_valid() {
    let mut s = valid_struct();
    s.validate().unwrap();
}

#[test]
#[should_panic(expected = "[\"Failed to validate field `value`, value too low\"]")]
fn test_too_small() {
    let mut s = valid_struct();
    s.value = 8;
    s.validate().unwrap();
}

#[test]
#[should_panic(expected = "[\"Failed to validate field `string`, value too short\"]")]
fn test_too_short() {
    let mut s = valid_struct();
    s.string = "hi".to_string();
    s.validate().unwrap();
}

#[test]
#[should_panic(expected = "[\"Failed to validate field `boolean`, value incorrect\"]")]
fn test_too_false() {
    let mut s = valid_struct();
    s.boolean = false;
    s.validate().unwrap();
}

#[test]
fn transform() {
    let mut s = valid_struct();
    s.transformer = "     CAST ME       ".to_string();
    s.validate().unwrap();
    assert_eq!(s.transformer, "cast me");
}

#[test]
#[should_panic(expected = "[\"Failed to validate field `transfailer`, value too long\"]")]
fn transfail() {
    let mut s = valid_struct();
    s.transfailer = "     CAST ME       ".to_string();
    s.validate().unwrap();
    assert_eq!(s.transformer, "cast me");
}
