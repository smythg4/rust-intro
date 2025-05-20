fn bubble_sort(arr: &mut [i32]) {
    for i in 0..arr.len() {
        for j in 0..(arr.len() - i - 1) {
            if arr[j] > arr[j+1] {
                arr.swap(j, j+1);
            }
        }
    }
}

fn insertion_sort<T: Ord + Copy>(arr: &mut [T]) {
    // sorts array in place
    for i in 1..arr.len() {
        let key = arr[i];
        let mut j = i;
        while j > 0 && arr[j-1] > key {
            arr[j] = arr[j-1];
            j -= 1;
        }
        arr[j] = key;
    }
}

fn main() {
    let mut a = [1, 10, 3, 8, 12, 2, 1, 0, 13, 10001, -5];
    println!("Unsorted: {:?}", a);
    insertion_sort(&mut a);
    println!("Sorted:   {:?}", a);

    let mut b = [1, 10, 3, 8, 12, 2, 1, 0, 13, 10001, -5];
    println!("Unsorted: {:?}", b);
    bubble_sort(&mut b);
    println!("Sorted:   {:?}", b);
}
