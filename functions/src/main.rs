//use std::io;

fn main() {
    let y = plus_one(10);
    println!("The value is: {y}");

    print_label_measurements(10,'h');
}

fn print_label_measurements(value: i32, unit_label: char) {
    println!("The measurement is: {value}{unit_label}");
}

fn plus_one(x: i32) -> i32 {
    x+1
}