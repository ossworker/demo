#[warn(dead_code)]
#[derive(Debug)]
struct ApiResult<T> {
    //编码
    pub code: Option<u32>,
    //msg
    pub msg: Option<String>,
    //数据
    pub data: Option<T>,
}

#[macro_export]
macro_rules! call_result {
    ($function: expr) => {
        ApiResult {
            code: Some(0),
            msg: Some("ok".to_string()),
            data: Some($function),
        }
    };
}

#[macro_export]
macro_rules! call_error {
    ($code: expr, $msg: expr) => {
        ApiResult {
            code: Some($code),
            msg: Some($msg.to_string()),
            data: None,
        }
    };
}

// #[proc_macro]
// pub fn call_success(input: TokenStream) -> TokenStream {
//     input
// }

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let result = call_result!({
            let a: i32 = 1;
            let b: i32 = 2;
            a + b
        });
        stringify!();
        println!("{:#?}", result);
        let result: ApiResult<String> = call_error!(10, "错误");
        println!("{:#?}", result);
    }
}
