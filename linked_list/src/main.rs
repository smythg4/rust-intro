struct Node {
    value: i32,
    next: Option<Box<Node>>,
}

struct LinkedList {
    head: Option<Box<Node>>,
    size: usize,
}

impl LinkedList {
    fn new() -> Self {
        LinkedList {
            head: None,
            size: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    fn len(&self) -> usize {
        self.size
    }

    fn push_front(&mut self, value: i32) {
        let new_node = Box::new( Node {
            value: value, // probably can just put value
            next: self.head.take(),
        });
        self.head = Some(new_node);
        self.size += 1;
    }

    fn push_back(&mut self, value: i32) {
        if self.is_empty() {
            self.push_front(value);
            return;
        }

        let mut current = &mut self.head;
        while let Some(node) = current {
            if node.next.is_none() {
                node.next = Some(Box::new(Node {
                    value,
                    next: None,
                }));
                break;
            }
            current = &mut node.next;
        }
        self.size += 1;
    }

    fn pop_front(&mut self) -> Option<i32> {
        match self.head.take() {
            None => None,
            Some(head_node) => {
                self.head = head_node.next;
                self.size -= 1;
                Some(head_node.value)
            }
        }
    }

    fn peek_front(&self) -> Option<&i32> {
        // need to understand how .as_ref() and .map() work
        self.head.as_ref().map(|node| &node.value)
    }

    // split list into two halves, returning only the second half
    fn split(&mut self) -> LinkedList {
        //with empty lists or lists with only 1 item, return an empty list
        if self.size <= 1 {
            return LinkedList::new();
        }

        // find mid point
        let mid = self.size / 2;
        let mut current = &mut self.head;

        // traverse to middle node
        for _ in 0..mid -1 {
            if let Some(node) = current {
                current = &mut node.next;
            }
        }

        //create new list with second half
        let mut second_half = LinkedList::new();

        if let Some(node) = current {
            second_half.head = node.next.take(); // this also makes self node.next None
            second_half.size = self.size - mid;
            self.size = mid;
        }
        second_half
    }

    //merge two sorted lists (self and other) into self
    fn merge(&mut self, mut other: LinkedList) {
        // if the other list is empty, do nothing. self is fine as is
        if other.is_empty() {
            return;
        }

        //if self is empty, just make self the other list
        if self.is_empty() {
            *self = other;
            return;
        }

        //create a new empty list to hold the merged one
        let mut result = LinkedList::new();

        //while both lists have elements, take the smaller one
        while !self.is_empty() && !other.is_empty() {            
            let value = match (self.peek_front(), other.peek_front()) {
                (Some(&a), Some(&b)) if a <= b => self.pop_front().unwrap(),
                _ => other.pop_front().unwrap()
            };
            result.push_back(value);
        }

        // add any remaining list elements from self
        while !self.is_empty() {
            result.push_back(self.pop_front().unwrap());
        }

        // add any remaining list elements from other
        while !other.is_empty() {
            result.push_back(other.pop_front().unwrap());
        }

        // replace self with result
        *self = result;
    }

    fn merge_sort(&mut self) {
        // base case: empty list or list with one element
        if self.size <= 1 {
            return;
        }

        // split list into two halves
        let mut second_half = self.split();

        //recursively sort both halves
        self.merge_sort();
        second_half.merge_sort();

        self.merge(second_half);
    }

    fn print(&self) {
        print!("[");

        let mut current = &self.head;
        let mut is_first = true;

        while let Some(node) = current {
            if !is_first {
                print!(" → ");
            }
            is_first = false;
            print!("{}", node.value);
            current = &node.next;
        }
        print!("]");
    }

}
fn main() {
    let mut list = LinkedList::new();

    list.push_front(3);
    list.push_front(1);
    list.push_front(5);
    list.push_front(2);
    list.push_front(4);
    list.push_front(10);

    println!("List: ");
    list.print();
    let size = list.len();
    println!(" Size: {size}");

    list.merge_sort();
    list.print();
    let size = list.len();
    println!(" Size: {size}");

    while !list.is_empty() {
        list.pop_front();
    }
    println!("List after clearing: ");
    list.print();
}
