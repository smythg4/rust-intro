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
    list.push_front(2);
    list.push_front(1);

    println!("List: ");
    list.print();
    println!("Size: {}", list.len());

    let first = list.pop_front();
    println!("Removed: {:?}", first);
    println!("List after removal: ");
    list.print();

    while !list.is_empty() {
        list.pop_front();
    }
    println!("List after clearing: ");
    list.print();
}
