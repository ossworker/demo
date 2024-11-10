#[no_mangle]
#[export_name = "return_string"]
pub fn return_string(s1: &str, s2: &str) -> String {
    println!("-=-|> s1: {}, s2: {}", s1, s2);

    let mut result = String::with_capacity(s1.len() + s2.len());
    let ret_ref = &result;
    println!(
        "  --> the address of result: {:p}",
        ret_ref as *const String
    );

    result.push_str(s1);
    result.push_str(s2);
    println!("  --> result: {result}");

    result
}

fn main() {
    println!("Hello, world!");
}
