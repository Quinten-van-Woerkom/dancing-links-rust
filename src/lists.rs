type Index = std::num::NonZeroUsize;

/**
 * Items represent a condition to be fulfilled. They are linked together in a
 * linked list denoting the items that remain to be covered in the subproblem
 * represented by the "composition" of the dancing links.
 */
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct ItemNode {
    previous: usize,
    next: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct Items {
    nodes: Box<[ItemNode]>
}

impl Items {
    fn new(size: usize) -> Self {
        let mut nodes = vec![ItemNode{ previous: 0, next: 0 }; size + 1].into_boxed_slice();
        
        for (index, ref mut node) in nodes.iter_mut().enumerate() {
            node.previous = index.wrapping_sub(1);
            node.next = index.wrapping_add(1);
        }

        nodes.first_mut().unwrap().previous = nodes.len() - 1;
        nodes.last_mut().unwrap().next = 0;

        Items{ nodes }
    }

    fn items(&mut self) -> Item {
        Item {
            current: 0,
            end: self.nodes.first().unwrap().previous,
            list: self,
        }
    }

    fn item(&mut self, index: Index) -> Item {
        Item {
            current: index.get(),
            end: self.nodes[index.get()].previous,
            list: self,
        }
    }
}

impl std::ops::Index<usize> for Items {
    type Output = ItemNode;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl std::ops::IndexMut<usize> for Items {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

/**
 * Items themselves are accessed through an iterator representing a single item.
 * This iterator allows an item to remove and reinsert itself from and to its
 * parent linked list.
 */
#[derive(Debug, Eq, PartialEq)]
struct Item<'a> {
    end: usize,
    current: usize,
    list: &'a mut Items,
}

impl<'a> Item<'a> {
    fn remove(&mut self) {
        let previous = self.previous;
        let next = self.next;

        self.list[previous].next = next;
        self.list[next].previous = previous;
    }

    fn reinsert(&mut self) {
        let previous = self.previous;
        let next = self.next;

        self.list[previous].next = self.current;
        self.list[next].previous = self.current;
    }

    fn index(&self) -> usize {
        self.current
    }
}

impl<'a> std::ops::Deref for Item<'a> {
    type Target = ItemNode;

    fn deref(&self) -> &Self::Target {
        &self.list[self.current]
    }
}

impl<'a> std::ops::DerefMut for Item<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list[self.current]
    }
}

impl<'a> Iterator for Item<'a> {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        if self.current == self.end {
            None
        } else {
            self.current = self.list[self.current].next;
            Some(())
        }
    }
}


#[cfg(test)]
mod items {
    use super::*;

    #[test]
    fn init() {
        let a = Items::new(7);
        let b = Items{ nodes: vec![
            ItemNode{ previous: 7, next: 1 },
            ItemNode{ previous: 0, next: 2 },
            ItemNode{ previous: 1, next: 3 },
            ItemNode{ previous: 2, next: 4 },
            ItemNode{ previous: 3, next: 5 },
            ItemNode{ previous: 4, next: 6 },
            ItemNode{ previous: 5, next: 7 },
            ItemNode{ previous: 6, next: 0 },
        ].into_boxed_slice()};
        assert_eq!(a, b, "Linked list nodes should point to directly adjacent nodes upon construction");
    }

    #[test]
    fn iterable() {
        let mut a = Items::new(7);
        assert_eq!(a.items().count(), 7);
    }

    #[test]
    fn removable() {
        let mut a = Items::new(7);
        a.item(Index::new(1).unwrap()).remove();
        assert_eq!(a.items().count(), 6);
    }

    #[test]
    fn reinsertable() {
        let mut a = Items::new(7);
        a.item(Index::new(1).unwrap()).remove();
        assert_eq!(a.items().count(), 6);
        a.item(Index::new(1).unwrap()).reinsert();
        assert_eq!(a.items().count(), 7);
    }

    #[test]
    fn emptyable() {
        let mut a = Items::new(7);

        for i in 1..=7 {
            a.item(Index::new(i).unwrap()).remove();
        }
        assert_eq!(a.items().count(), 0);

        for i in 1..=7 {
            a.item(Index::new(i).unwrap()).reinsert();
        }
        assert_eq!(a.items().count(), 7);
    }
}


struct SpacerNode {
    previous: Index,
    next: Index,
}

struct OptionNode {
    parent: Index,
    previous: Index,
    next: Index,
}