use vale::Validate;

struct Struct {
    value: u32,
    string: String,
    boolean: bool,
    even_value: i32,
    transformer: String,
    transfailer: String,
}

impl vale::Validate for Struct {
#[vale::ruleset]
fn validate(&mut self) -> Result<(), Vec<String>> {
    vale::rule!(self.value > 10, "Too low");
    vale::rule!(self.string.len() > 3, "Too short");
    vale::rule!(self.boolean, "Too false!");
    vale::rule!(is_even(self.even_value), "Not even!");
    self.transformer = self.transformer.trim().to_string();
    vale::rule!(self.transformer.len() < 10, "Too short");
    self.transformer = self.transformer.to_lowercase();
    vale::rule!(self.transfailer.len() < 10);
    self.transfailer = self.transfailer.trim().to_string();
}
}

fn is_even(arg: i32) -> bool {
    arg % 2 == 0
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
#[should_panic(expected = "Too low")]
fn test_too_small() {
    let mut s = valid_struct();
    s.value = 8;
    s.validate().unwrap();
}

#[test]
#[should_panic(expected = "Too short")]
fn test_too_short() {
    let mut s = valid_struct();
    s.string = "hi".to_string();
    s.validate().unwrap();
}

#[test]
#[should_panic(expected = "Too false!")]
fn test_too_false() {
    let mut s = valid_struct();
    s.boolean = false;
    s.validate().unwrap();
}

#[test]
#[should_panic(expected = "Not even!")]
fn test_with() {
    let mut s = valid_struct();
    s.even_value = 7;
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
#[should_panic(expected = "No message provided")]
fn transfail() {
    let mut s = valid_struct();
    s.transfailer = "     CAST ME       ".to_string();
    s.validate().unwrap();
    assert_eq!(s.transformer, "cast me");
}
