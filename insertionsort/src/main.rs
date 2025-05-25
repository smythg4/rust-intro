use std::time::Instant;
use std::thread;
use rayon::prelude::*;

fn benchmark_sort<T, F>(name: &str, data: &mut [T], sort_fn: F) 
where 
    T: Clone + Ord,
    F: Fn(&mut [T])
{
    let start = Instant::now();
    sort_fn(data);
    let duration = start.elapsed();
    println!("{}: {:?}", name, duration);
}

fn bubble_sort<T: Ord + Clone>(arr: &mut [T]) {
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

fn merge<T>(arr: &mut [T], midpoint: usize) where  T: Ord + Clone {
    // copy the left side because you're gonna be dumping values into the acrual array's left side
    let lhs: Vec<T> = arr[0..midpoint].to_vec();
    
    let mut i = 0;
    let mut j = midpoint;
    let mut k = 0;

    while i < lhs.len() && j < arr.len() {
        if lhs[i] <= arr[j] {
            arr[k] = lhs[i].clone();
            i += 1;
        } else {
            arr[k] = arr[j].clone();
            j += 1;
        }
        k += 1;
    }

    // copy remaining elements from left side into array
    while i < lhs.len() {
        arr[k] = lhs[i].clone();
        i += 1;
        k += 1;
    }   

    // right side should already be in place

}

fn merge_sort<T: Ord + Clone>(arr: &mut [T]) {
    // sorts array in place
    if arr.len() == 0 || arr.len() == 1 {
        return;
    }
    let midpoint = arr.len() / 2;
    let (lhs, rhs) = arr.split_at_mut(midpoint);
    merge_sort(lhs);
    merge_sort(rhs);
    merge(arr, midpoint);
}

fn merge_sort_mt<T: Ord + Clone + Send>(arr: &mut [T]) {
    if arr.len() == 0 || arr.len() == 1 {
        return;
    }

    const THRESHOLD: usize = 1000;

    let midpoint = arr.len() / 2;
    let (lhs, rhs) = arr.split_at_mut(midpoint);

    if midpoint*2 > THRESHOLD {
        rayon::join(
            || merge_sort_mt(lhs),
            || merge_sort_mt(rhs),
        );
    } else {
        merge_sort(lhs);
        merge_sort(rhs);
    }
    merge(arr, midpoint);
}

fn main() {
    let sizes = vec![100, 1000, 5000, 10000, 20000];
    
    for size in sizes {
        println!("\nTesting with {} elements:", size);
        
        // Generate random data
        let mut data: Vec<i32> = (0..size).map(|_| rand::random::<i32>()).collect();
        
        // Test each algorithm
        let mut bubble = data.clone();
        benchmark_sort("Bubble Sort", &mut bubble, bubble_sort);
        
        let mut insertion = data.clone();
        benchmark_sort("Insertion Sort", &mut insertion, insertion_sort);
        
        let mut merge_st = data.clone();
        benchmark_sort("Sequential Merge Sort", &mut merge_st, merge_sort);

        let mut merge_mt = data.clone();
        benchmark_sort("Parallel Merge Sort", &mut merge_mt, merge_sort_mt);

        // ensure they're sorted properly
        data.sort();
        assert_eq!(data, bubble);
        assert_eq!(data, insertion);
        assert_eq!(data, merge_st);
        assert_eq!(data, merge_mt);
    }
}
