use std::io::{self, Write};

fn print_calculation(a: f64, b: f64, operator: &str) {
    print!("{a} {} {b} = ", operator);
    match operator {
        "+" => println!("{}", a + b),
        "-" => println!("{}", a - b),
        "*" => println!("{}", a * b),
        "/" => println!("{}", a / b),
        "%" => println!("{}", a % b),
        _ => println!("Invalid operator \"{operator}\""),
    }
}

fn main() {
    let mut temp = String::new();
    let mut operator = String::new();

    print!("Enter a number >");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut temp)
        .expect("Failed to read line");
    let a: f64 = temp.trim().parse().expect("Failed to parse number a");
    temp.clear();

    print!("Enter an operator [ + - * / % ] >");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut operator)
        .expect("Failed to read line");
    let operator = operator.trim();

    print!("Enter a number >");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut temp)
        .expect("Failed to read line");
    let b: f64 = temp.trim().parse().expect("Failed to parse number b");

    print_calculation(a, b, operator);
}
