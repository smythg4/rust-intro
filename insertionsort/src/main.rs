fn insertion_sort(arr: &mut [i32]) {
    // sorts array in place
    for i in 1..arr.len() {
        let key = arr[i];
        let mut j = i-1;
        while j > 0 && arr[j] > key {
            arr[j+1] = arr[j];
            j -= 1;
        }
        if j == 0 && arr[j] > key {
            arr[1] = arr[0];
            arr[0] = key;
        } else {
            arr[j+1] = key;
        }
    }
}

fn main() {
    let mut a = [1, 10, 3, 8, 12, 2, 1, 0, 13, 10001, -5];
    println!("Unsorted: {:?}",a);
    insertion_sort(&mut a);
    println!("Sorted:   {:?}",a);
}
