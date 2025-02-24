use proc::event_handler;

// #[event_handler(kind = "EarlGrey", hot = true, with(sugar = "lol", milk))]
#[event_handler(
    kind = "EarlGrey",
    hot = true,
    vars(sugar = "zs", milk = "lisi"),
    vars(sugar = "aa", milk = "bb")
)]
pub fn test(a: i32, b: i32) -> i32 {
    // std::thread::sleep(std::time::Duration::from_secs(1));
    a + b
}

fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    let test_result = test(1, 2);

    println!("test_result: {:?}", test_result?);
    Ok(())
}
