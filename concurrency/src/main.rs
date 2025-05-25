use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::sync::{Arc,Mutex};

fn communication () {
    let (tx, rx) = mpsc::channel();
    let tx1 = tx.clone();

    thread::spawn(move || {
        let vals = vec![ String::from("hi"), 
                        String::from("there"), 
                        String::from("you"), 
                        String::from("cool"),
                        String::from("d00d")];
        for val in vals {
            tx1.send(val).unwrap();
            thread::sleep(Duration::from_secs(1))
        }
    });

    thread::spawn(move || {
        let vals = vec![ String::from("2hi"), 
                        String::from("2there"), 
                        String::from("2you"), 
                        String::from("2cool"),
                        String::from("2d00d")];
        for val in vals {
            tx.send(val).unwrap();
            thread::sleep(Duration::from_secs(1))
        }
    });

    for received in rx {
        println!("Got: {received}");
    }
}

fn mutex() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("Result: {}", *counter.lock().unwrap());
}

fn main() {
    mutex();
    communication();
}