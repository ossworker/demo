use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, ItemFn, ItemUse, LitBool, LitStr, Path, ReturnType, Type, braced,
    parse::{self, Parse},
    parse_file, parse_macro_input, parse_quote,
    visit_mut::VisitMut,
};

#[allow(dead_code)]
#[derive(Debug)]
struct ApiResult<T> {
    //编码
    pub code: Option<u32>,
    //msg
    pub msg: Option<String>,
    //数据
    pub data: Option<T>,
}

#[allow(unused_macros)]
macro_rules! call_result {
    ($function: expr_2021) => {
        ApiResult {
            code: Some(0),
            msg: Some("ok".to_string()),
            data: Some($function),
        }
    };
}

#[allow(unused_macros)]
macro_rules! call_error {
    ($code: expr_2021, $msg: expr_2021) => {
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

// fn example(attr: &Attribute) -> syn::Result<()> {
//     attr.parse_nested_meta(|meta| {
//         if meta.path.is_ident("kind") {
//             // ...
//             Ok(())
//         } else {
//             Err(meta.error("unsupported tea property"))
//         }
//     })?;
//     Ok(())
// }

#[derive(Default, Debug)]
struct EventAttr {
    kind: Option<LitStr>,
    hot: Option<LitBool>,
    vars: Vec<VarAttr>,
}

#[derive(Default, Debug)]
struct VarAttr {
    sugar: Option<LitStr>,
    milk: Option<LitStr>,
}

impl Parse for VarAttr {
    fn parse(content: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attr = VarAttr::default();
        // let content;
        // syn::parenthesized!(content in input);
        // println!("vars string: {:#?}", &content.to_string());

        while !content.is_empty() {
            let key: syn::Ident = content.parse()?;
            // println!("vars key: {:#?}", &key.to_string());

            content.parse::<syn::Token![=]>()?;

            match key.to_string().as_str() {
                "sugar" => {
                    attr.sugar = Some(content.parse()?);
                }
                "milk" => {
                    attr.milk = Some(content.parse()?);
                }
                _ => {
                    // return Err(syn::Error::new(
                    //     key.span(),
                    //     format!("unsupported tea property: {}", key),
                    // ));
                }
            }

            if content.is_empty() {
                break;
            }
            content.parse::<syn::Token![,]>()?;
        }
        Ok(attr)
    }
}

impl Parse for EventAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        println!("event: {:#?}", &input.to_string());
        let mut attr = EventAttr::default();
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            // println!("key: {:#?}", &key.to_string());

            match key.to_string().as_str() {
                "kind" => {
                    input.parse::<syn::Token![=]>()?;
                    attr.kind = Some(input.parse()?);
                }
                "hot" => {
                    input.parse::<syn::Token![=]>()?;
                    attr.hot = Some(input.parse()?);
                }
                "vars" => {
                    let content;
                    syn::parenthesized!(content in input);

                    // let vars: syn::punctuated::Punctuated<VarAttr, syn::Token![,]> =
                    //     input.parse_terminated(VarAttr::parse, syn::Token![,])?;
                    // attr.vars = vars.into_iter().collect();

                    attr.vars.extend(
                        syn::punctuated::Punctuated::<VarAttr, syn::Token![,]>::parse_terminated(
                            &content,
                        )?
                        .into_iter(),
                    );
                }
                _ => {
                    // return Err(syn::Error::new(
                    //     key.span(),
                    //     format!("unsupported tea property: {}", key),
                    // ));
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<syn::Token![,]>()?;
        }
        Ok(attr)
    }
}

fn extract_inner_type(ty: &Type) -> Type {
    match ty {
        Type::Path(type_path) => {
            if type_path.path.segments.last().unwrap().ident == "Result" {
                if let Some(segments) = type_path.path.segments.first() {
                    if let syn::PathArguments::AngleBracketed(args) = &segments.arguments {
                        if let Some(first_arg) = args.args.first() {
                            return parse_quote!(#first_arg);
                        }
                    }
                }
            }
        }
        _ => todo!(),
    }
    ty.clone()
}

fn is_result_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(path) => {
            let segments = &path.path.segments;
            return segments.last().unwrap().ident == "Result";
        }
        _ => {}
    }
    false
}

struct ReturnTypeReplacer {
    is_result: bool,
}

impl VisitMut for ReturnTypeReplacer {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        // 处理显式 return
        if let syn::Expr::Return(ret) = expr {
            if let Some(content) = &mut ret.expr {
                let wrapped = if self.is_result {
                    quote! { #content? }
                } else {
                    quote! { Ok(#content) }
                };
                ret.expr = Some(syn::parse_quote!(#wrapped));
            }
        }
        syn::visit_mut::visit_expr_mut(self, expr);
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        if let Some(last_expr) = block.stmts.last_mut() {
            if let syn::Stmt::Expr(expr_mut, None) = last_expr {
                let wrapped = if self.is_result {
                    quote! { #expr_mut }
                } else {
                    quote! { Ok(#expr_mut) }
                };
                *expr_mut = syn::parse_quote!(#wrapped);
            }
        }
        syn::visit_mut::visit_block_mut(self, block);
    }

    fn visit_use_tree_mut(&mut self, i: &mut syn::UseTree) {
        println!("visit_use_tree_mut: {:?}", i);
    }

    fn visit_use_path_mut(&mut self, i: &mut syn::UsePath) {
        println!("visit_use_path_mut: {:?}", i);
    }
}

#[proc_macro_attribute]
pub fn sub_attr(_attr: TokenStream, func: TokenStream) -> TokenStream {
    println!("sub_attr ------");

    // let func_clone = func.clone();
    // println!("func_clone: {:?}", &func_clone);
    // let item_fn = parse_macro_input!(func_clone as syn::ItemFn);
    // // 假设函数位于模块顶层，可根据实际情况调整作用域
    // let use_map = build_use_map(&[syn::Item::Fn(item_fn)]);
    // println!("use_map: {:?}", use_map);

    let mut func = parse_macro_input!(func as syn::ItemFn); //传入的函数，用到ItemFn
    // let func_vis = &func.vis; //函数的可见性 pub
    let func_block = &func.block; //函数主体部分()
    let func_sig = &func.sig; //函数签名
    // let func_name = &func_sig.ident; //函数名
    // let func_generics = &func_sig.generics; //函数泛型
    // let func_inputs = &func_sig.inputs; //函数输入参数
    let func_output_return_type = &func_sig.output; //函数输出参数
    let mut is_result = false;
    // 判断函数返回类型是否是Result
    let return_type: ReturnType = match &func_output_return_type {
        syn::ReturnType::Type(_, ty) => {
            is_result = is_result_type(ty);
            let inner_ty = extract_inner_type(ty);
            println!("inner_ty: {:?}", inner_ty.to_token_stream().to_string());
            parse_quote!(-> anyhow::Result<#inner_ty>)
        }
        _ => parse_quote!(-> anyhow::Result<()>),
    };

    func.sig.output = return_type;

    // 判断函数返回类型是否是Result

    let mut visior = ReturnTypeReplacer { is_result };
    visior.visit_item_fn_mut(&mut func);

    // 修改 block 返回
    // func.block = parse_quote!({
    //     let result = #func_block;
    //     Ok(result)
    // });

    // func_output.
    let instream = quote! {
        #func
    };
    println!("instream: {}", instream.to_string());
    instream.into()
}

// 新增工具函数：解析模块中的 use 语句建立别名映射
fn build_use_map(items: &[syn::Item]) -> HashMap<String, String> {
    let mut use_map = HashMap::new();
    for item in items {
        if let syn::Item::Use(item_use) = item {
            parse_use_tree(&item_use.tree, &mut use_map, "");
        }
    }
    use_map
}

// 递归解析 UseTree
fn parse_use_tree(tree: &syn::UseTree, map: &mut HashMap<String, String>, base: &str) {
    match tree {
        syn::UseTree::Path(path) => {
            let new_base = if base.is_empty() {
                path.ident.to_string()
            } else {
                format!("{}::{}", base, path.ident)
            };
            parse_use_tree(&path.tree, map, &new_base);
        }
        syn::UseTree::Name(name) => {
            map.insert(name.ident.to_string(), base.to_string());
        }
        syn::UseTree::Rename(rename) => {
            map.insert(
                rename.rename.to_string(),
                format!("{}::{}", base, rename.ident),
            );
        }
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                parse_use_tree(tree, map, base);
            }
        }
        _ => {}
    }
}

// 修改后的路径解析逻辑
fn resolve_full_path(ty: &syn::Type, use_map: &HashMap<String, String>) -> String {
    let mut segments = Vec::new();
    if let syn::Type::Path(type_path) = ty {
        for segment in &type_path.path.segments {
            if let Some(original) = use_map.get(&segment.ident.to_string()) {
                segments.push(original.clone());
            } else {
                segments.push(segment.ident.to_string());
            }
        }
    }
    segments.join("::")
}

#[proc_macro_attribute]
pub fn event_handler(attr: TokenStream, func: TokenStream) -> TokenStream {
    // syn::Attribute::

    let func = parse_macro_input!(func as syn::ItemFn); //传入的函数，用到ItemFn

    let func_vis = &func.vis; //函数的可见性 pub
    let func_block = &func.block; //函数主体部分()
    let func_stmts = &func_block.stmts;

    let func_sig = &func.sig; //函数签名
    let func_name = &func_sig.ident; //函数名
    let func_generics = &func_sig.generics; //函数泛型
    let func_inputs = &func_sig.inputs; //函数输入参数
    let func_output: &syn::ReturnType = &func_sig.output; //函数输出参数

    println!("func_vis: {:#?}", &func_vis);
    println!("func_block: {:#?}", &func_block);
    println!("func_sig: {:#?}", &func_sig);
    println!("func_name: {:#?}", &func_name);
    println!("func_generics: {:#?}", &func_generics);
    println!("func_inputs: {:#?}", &func_inputs);
    println!("func_output: {:#?}", &func_output);

    //提取参数，可能多个
    let params: Vec<_> = func_inputs
        .iter()
        .map(|arg| match arg {
            // 提取形参的pattern
            // https://docs.rs/syn/1.0.1/syn/struct.PatType.html
            syn::FnArg::Typed(pat_type) => &pat_type.pat,
            syn::FnArg::Receiver(_receiver) => unreachable!("receiver"),
        })
        .collect();

    println!("params: {:#?}", params);

    //解析attr
    let mut kind: Option<LitStr> = None;
    let mut hot: bool = false;
    let mut vars: Vec<VarAttr> = Vec::new();

    let event_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("kind") {
            kind = Some(meta.value()?.parse::<LitStr>()?);
            Ok(())
        } else if meta.path.is_ident("hot") {
            hot = true;
            Ok(())
        } else if meta.path.is_ident("vars") {
            meta.parse_nested_meta(|vars_meta| {
                // println!("vars: {:#?}", &vars_meta.path);
                // &vars.value()?.parse::<LitStr>()?;
                let mut var = VarAttr::default();
                // with.push(&meta.path);
                if vars_meta.path.is_ident("sugar") {
                    var.sugar = Some(vars_meta.value()?.parse::<LitStr>()?);
                    // Here we can go even deeper if needed.
                } else if vars_meta.path.is_ident("milk") {
                    var.milk = Some(vars_meta.value()?.parse::<LitStr>()?);
                }
                vars.push(var);
                Ok(())
            })
        } else {
            Err(meta.error("unsupported tea property"))
        }
    });

    // parse_macro_input!(args with event_parser);

    let event_attr = parse_macro_input!(attr as EventAttr);

    println!("event_attr: {:?}", event_attr);

    // println!("kind: {:?} hot: {:?} vars: {:#?}", kind, hot, vars);
    quote! {
        use proc::sub_attr;
        #[sub_attr]
         #func_vis fn #func_name #func_generics(#func_inputs) #func_output {
            let _timer = {
                struct __MetricGuard {
                    start: ::std::time::Instant,
                    function: &'static str,
                }
                impl ::std::ops::Drop for __MetricGuard {
                    fn drop(&mut self) {
                        let elapsed = self.start.elapsed();
                        println!(
                            "[METRIC] {} cost: {:.3}ms",
                            self.function,
                            elapsed.as_secs_f64() * 1000.0
                        );
                    }
                }
                let _ = __MetricGuard {
                    start: ::std::time::Instant::now(),
                    function: module_path!(),
                };
            };

            #(#func_stmts)*

            // fn rebuild_func #func_generics(#func_inputs) #func_output #func_block
            // // 注意这个#attr的函数签名：fn time_measure<F>(func: F) -> impl Fn(u64) where F: Fn(u64)
            // // 形参是一个函数，就是rebuild_func
            // let f = #attr_ident(rebuild_func);

            // // 要修饰函数的参数，有可能是多个参数，所以这样匹配 #(#params,) *
            // f(#(#params,) *)
        }

    }
    .into()
}

#[cfg(test)]
mod tests {

    use quote::quote;

    use super::*;

    #[test]
    fn it_works() {
        println!("------");
        // let attr_stream = quote! {#[event_handler(hot, with = ["a","b"])]};
        // let func_stream = quote! {
        //     pub fn test(a: i32, b: i32) -> i32 {
        //         a + b
        //     }
        // };
        // let result_stream = event_handler(attr_stream.into(), func_stream.into());
        // println!("{:?}", result_stream);
    }
}
