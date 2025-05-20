use std::fmt;

struct LinkedList<T> {
    head: Option<Box<ListNode<T>>>,
    size: u32
}

struct ListNode<T> {
    val: T,
    next: Option<Box<ListNode<T>>>
}

impl<T> ListNode<T> {
    fn new(val: T) -> Self {
        ListNode {
            val,
            next: None,
        }
    }
}

impl<T> LinkedList<T> {
    fn new(val: T) -> Self {
        LinkedList {
            head: Some(Box::new(ListNode::new(val))),
            size: 1,
        }
    }

    fn push_front(&mut self, val: T) {
        let mut new_node = Box::new(ListNode::new(val));
        
        match self.head.take() {
            Some(old_head) => new_node.next = Some(old_head),
            None => new_node.next = None,
        }
        self.head = Some(new_node);
        self.size += 1;
    }

    fn push_back(&mut self, val: T) {
        let new_node = Box::new(ListNode::new(val));
        
        match &mut self.head {
            None => {
                self.head = Some(new_node);
            },
            Some(head) => {
                // traverse to end of list
                let mut current = head;
                while let Some(ref mut next) = current.next {
                    current = next;
                }
                current.next = Some(new_node);
            }
        }
        self.size += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        match self.head.take() {
            None => None,
            Some(node) => {
                self.head = node.next;
                self.size -= 1;
                Some(node.val)
            }
        }
    }

    fn peek_front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.val)
    }

    fn peek_back(&self) -> Option<&T> {
        let mut current = self.head.as_ref();

        if current.is_none() {
            return None;
        }

        while let Some(node) = current {
            match &node.next {
                None => return Some(&node.val),
                Some(_) => current = node.next.as_ref(),
            }
        }
        None
    }

    fn contains(&self, val: &T) -> bool where T: PartialEq {
        let mut current = self.head.as_ref();

        while let Some(node) = current {
            if &node.val == val {
                return true;
            }
            current = node.next.as_ref();
        }
        false
    }
}

impl<T: fmt::Display> fmt::Display for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.head {
            None => write!(f, "Empty list"),
            Some(head) => {
                write!(f,"{}", head.val)?;

                let mut current = &head.next;
                while let Some(node) = current {
                    write!(f," -> {}", node.val)?;
                    current = &node.next;
                }
                Ok(())
            }
        }
    }
}

fn main() {
    let mut my_list = LinkedList::new(String::from("Paul"));
    my_list.push_back(String::from("John"));
    my_list.push_back(String::from("Ringo"));
    my_list.push_back(String::from("George"));
    println!("{}", my_list);
    if let Some(first) = my_list.pop_front() {
        println!("Moving to back: {}", first);
        my_list.push_back(first);
    }
    println!("Updated list: {}", my_list);

    my_list.push_front(String::from("Jimmy Page"));
    println!("Updated list: {}", my_list);

    if let Some(last) = my_list.peek_back() {
        println!("Last item: {}", last);
    }

    my_list.push_back(String::from("Robert Plant"));
    println!("Updated list: {}", my_list);

    if let Some(last) = my_list.peek_back() {
        println!("Last item: {}", last);
    }
    let check = String::from("Robert Plant");
    println!("List contains {}? {}", check, my_list.contains(&check));
}