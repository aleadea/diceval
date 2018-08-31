extern crate diceval;

#[test]
fn expr() {
    let result = diceval::eval("21+21".to_string()).unwrap();
    assert_eq!(result, (42, "21 + 21".to_string()))
}
