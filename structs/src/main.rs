fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}

fn print_num(x: Option<i32>) {
    match x {
        None => println!("None value"),
        Some(i) => println!("{i}"),
    }
}

fn main() {
    let five = Some(5);
    let six = plus_one(five);
    let none = plus_one(None);

    print_num(six);
    print_num(none);
}