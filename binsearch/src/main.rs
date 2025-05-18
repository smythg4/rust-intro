fn binsearch(arr: &[i32], val: i32) -> Option<usize> {
    //accepts an array of ints and a value to search for. returns index of said value.
    if arr.is_empty() {
        return None;
    }
    let mut low = 0;
    let mut high = arr.len() - 1;
    while low <= high {
        let mid = low + (high-low)/2; // avoids risk of integer overflow
        if arr[mid] == val {
            return Some(mid);
        } else if arr[mid] < val {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    None
}

fn main() {
    let a = [1, 2, 3, 4, 5, 6, 10, 100, 101, 120, 155, 10001];

    let cases = [6, 120, 155, 2, 7]; // 7 isn't in list for testing purposes

    for &val in &cases {
        match binsearch(&a, val) {
            Some(i) => {
                println!("Found {} at index {}", val, i);
                assert_eq!(a[i],val);
            },
            None => println!("{} not found in array", val),
        }
    }

    //test on empty list
    let empty: [i32; 0] = [];
    match binsearch(&empty, 5) {
        Some(i) => println!("Found 5 at index {}", i),
        None => println!("5 not found in empty array, as expected"),
    }

}
