use std::fs::File;
use std::io::{prelude::*, stdin, stdout};
use std::thread::sleep;
use std::time::Duration;
use std::{env, process};

fn main() {
    let args: Vec<_> = env::args().collect();
    let envs: Vec<_> = env::vars().collect();

    let _ = &args.iter().for_each(|arg| println!("arg: {:#?}", arg));
    let _ = &envs.iter().for_each(|env| println!("env {:#?}", env));

    let mut cmd = "daemon";
    if args.len() >= 2 {
        cmd = &args[1];
    }

    let mut input = String::new();
    stdin().read_to_string(&mut input).expect("msg");

    println!("input: {:#?} --", input);

    match cmd {
        "echo" => println!("{}", &args[2..].join(" ")),
        "sleep" => sleep(Duration::from_secs_f64(args[2].parse::<f64>().unwrap())),
        "exit" => process::exit(args[2].parse::<i32>().unwrap()),
        "write" => {
            let mut file = File::create(&args[2]).unwrap();
            file.write_all(args[3..].join(" ").as_bytes()).unwrap();
        }
        "daemon" => loop {
            println!(
                "This is a song that never ends.\nYes, it goes on and on my friends.\nSome people \
                 started singing it not knowing what it was,\nSo they'll continue singing it \
                 forever just because...\n"
            );
            sleep(Duration::from_secs(1));
        },
        _ => {
            eprintln!("unknown command: {0}", args[1]);
            process::exit(1);
        }
    }

    let output = String::from("{\"id\":1,\"name\":\"张三\"}");

    let mut stdout_lock = stdout().lock();

    stdout_lock
        .write_all(output.as_bytes())
        .expect("Error when returning the response");
    stdout_lock.write_all(b"\n").expect("msg");

    stdout_lock.flush().unwrap();
}
