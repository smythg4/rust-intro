fn main() {

    for i in 1..10 {
        let fib = fib(i);

        println!("The {i} finonacci number is {fib}");
    }
    let f = 100.0;
    let c = f_to_c(f);
    println!("{f} F is {c} C");

    let c2 = 40.0;
    let f2 = c_to_f(c2);
    println!("{c2} C is {f2} F");
}

fn fib(n: i32) -> i32 {
    if n == 1 {
        1
    } else if n == 2 {
        1
    } else {
        fib(n-1) + fib(n-2)
    }
}

fn f_to_c(f: f32) -> f32 {
    //C = (F - 32) × 5/9
    (f - 32.0) * (5.0/9.0)
}

fn c_to_f(c: f32) -> f32 {
    (9.0/5.0)*c + 32.0
}