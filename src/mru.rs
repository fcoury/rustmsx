use std::collections::VecDeque;

pub struct MRUList<T> {
    capacity: usize,
    items: VecDeque<T>,
}

impl<T: PartialEq> MRUList<T> {
    pub fn new(capacity: usize) -> Self {
        MRUList {
            capacity,
            items: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, item: T) {
        // If the item is already in the list, remove it
        if let Some(index) = self.items.iter().position(|x| *x == item) {
            self.items.remove(index);
        } else if self.items.len() == self.capacity {
            // If the list is full, remove the least recently used item
            self.items.pop_back();
        }

        // Add the item to the front of the list
        self.items.push_front(item);
    }

    #[allow(unused)]
    pub fn get_items(&self) -> &VecDeque<T> {
        &self.items
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            inner: self.items.iter(),
        }
    }
}

pub struct Iter<'a, T> {
    inner: std::collections::vec_deque::Iter<'a, T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
