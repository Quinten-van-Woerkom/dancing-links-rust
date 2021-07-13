/**
 * "Dancing links" utilises two types of intrusive linked lists: one to keep
 * track of the items that remain to be covered, and one to keep track of the
 * remaining options that can cover each item.
 * An intrusive linked list implementation is written here that covers both
 * use-cases.
 */
use std::cell::Cell;

/**
 * Link used to connect objects within a linked list.
 */
struct Link<'list, Node: LinkedList<'list>> {
    next: Cell<Option<&'list Node>>,
    previous: Cell<Option<&'list Node>>
}

impl<'list, Node: LinkedList<'list>> Link<'list, Node> {
    /**
     * Because we want the list to be self-referential, it cannot be directly
     * initialized but must start in an invalid state.
     */
    fn uninitialized() -> Self {
        Self {
            next: Cell::from(None),
            previous: Cell::from(None),
        }
    }
}

/**
 * An intrusive linked list. Used to allow reversible removal of items from a
 * list of active objects.
 */
trait LinkedList<'list>: std::marker::Sized {
    fn link(&'list self) -> &'list Link<'list, Self>;

    /**
     * Hook used if one wants to do something when inserting nodes into the
     * linked list e.g. keep track of size. Does nothing by default.
     */
    fn grow(&self) {}

    /**
     * Hook used if one wants to do something when removing nodes from the
     * linked list e.g. keep track of size. Does nothing by default.
     */
    fn shrink(&self) {}

    /**
     * Initializes a list by connecting it to itself.
     * Every list node must be initialized using connect_self() or by being
     * inserted into another linked list before usage.
     */
    fn connect_self(&'list self) {
        self.set_next(self);
        self.set_previous(self);
    }

    /**
     * Inserts another node to the left of the current node in this list.
     * Every list node must be initialized using connect_self() or by being
     * inserted into another linked list before usage.
     */
    fn prepend(&'list self, node: &'list Self) {
        self.grow();
        node.set_previous(self.previous());
        node.set_next(self);
        self.previous().set_next(node);
        self.set_previous(node);
    }

    /**
     * Reversibly (!) removes a node from its parent linked list.
     */
    fn remove(&'list self) {
        self.shrink();
        self.next().set_previous(self.previous());
        self.previous().set_next(self.next());
    }

    /**
     * Reinserts a node into its parent linked list.
     */
    fn reinsert(&'list self) {
        self.grow();
        self.next().set_previous(self);
        self.previous().set_next(self);
    }

    /**
     * Caller must ensure that the link is initialized to a valid state, i.e.
     * that it has neighbours.
     * TODO: See if unwrap() impacts performance, consider unwrap_unchecked().
     */
    fn next(&'list self) -> &'list Self {
        self.link().next.get().unwrap()
    }

    /**
     * Caller must ensure that the link is initialized to a valid state, i.e.
     * that it has neighbours. Panics otherwise.
     * TODO: See if unwrap() impacts performance, consider unwrap_unchecked().
     */
    fn previous(&'list self) -> &'list Self {
        self.link().previous.get().unwrap()
    }

    fn set_next(&'list self, node: &'list Self) {
        self.link().next.set(Some(node));
    }

    fn set_previous(&'list self, node: &'list Self) {
        self.link().previous.set(Some(node));
    }

    /**
     * A linked list is empty if it is connected to itself.
     */
    fn is_empty(&'list self) -> bool {
        self.next() as *const Self == self as *const Self
        && self.previous() as *const Self == self as *const Self
    }

    /**
     * A linked list node is valid if both its neighbour pointers exist.
     */
    fn is_valid(&'list self) -> bool {
        self.link().next.get().is_some() && self.link().previous.get().is_some()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct List<'list> {
        link: Link<'list, Self>,
    }

    impl<'list> LinkedList<'list> for List<'list> {
        fn link(&'list self) -> &'list Link<'list, Self> {
            &self.link
        }
    }

    #[test]
    fn initialization() {
        let list = List { link: Link::uninitialized() };
        assert!(!list.is_valid());

        list.connect_self();

        assert!(list.is_valid());
        assert!(list.is_empty());
    }

    #[test]
    fn insertion() {
        let list = List { link: Link::uninitialized() };
        list.connect_self();

        let node = List { link: Link::uninitialized() };
        list.prepend(&node);

        assert!(list.is_valid());
        assert!(node.is_valid());
        assert!(!list.is_empty());
        assert_eq!(list.next() as *const List, &node as *const List);
        assert_eq!(list.previous() as *const List, &node as *const List);
        assert_eq!(node.next() as *const List, &list as *const List);
        assert_eq!(node.previous() as *const List, &list as *const List);

        let node2 = List { link: Link::uninitialized() };
        list.prepend(&node2);

        assert!(list.is_valid());
        assert!(node.is_valid());
        assert!(!list.is_empty());
        assert_eq!(list.next() as *const List, &node as *const List);
        assert_eq!(list.previous() as *const List, &node2 as *const List);
        assert_eq!(node.next() as *const List, &node2 as *const List);
        assert_eq!(node.previous() as *const List, &list as *const List);
        assert_eq!(node2.next() as *const List, &list as *const List);
        assert_eq!(node2.previous() as *const List, &node as *const List);
    }

    #[test]
    fn removal() {
        let nodes: [List; 3] = [
            List{ link: Link::uninitialized() },
            List{ link: Link::uninitialized() },
            List{ link: Link::uninitialized() }
        ];

        nodes[0].connect_self();
        nodes[0].prepend(&nodes[1]);
        nodes[0].prepend(&nodes[2]);

        nodes[1].remove();

        assert!(nodes[0].is_valid());
        assert!(!nodes[0].is_empty());
        assert_eq!(nodes[0].next() as *const List, &nodes[2] as *const List);
        assert_eq!(nodes[0].previous() as *const List, &nodes[2] as *const List);
        assert_eq!(nodes[1].next() as *const List, &nodes[2] as *const List);
        assert_eq!(nodes[1].previous() as *const List, &nodes[0] as *const List);
        assert_eq!(nodes[2].next() as *const List, &nodes[0] as *const List);
        assert_eq!(nodes[2].previous() as *const List, &nodes[0] as *const List);

        nodes[2].remove();
        assert!(nodes[0].is_valid());
        assert!(nodes[0].is_empty());
        assert_eq!(nodes[0].next() as *const List, &nodes[0] as *const List);
        assert_eq!(nodes[0].previous() as *const List, &nodes[0] as *const List);
    }

    #[test]
    fn reinsertion() {
        let nodes: [List; 3] = [
            List{ link: Link::uninitialized() },
            List{ link: Link::uninitialized() },
            List{ link: Link::uninitialized() }
        ];

        nodes[0].connect_self();
        nodes[0].prepend(&nodes[1]);
        nodes[0].prepend(&nodes[2]);

        nodes[1].remove();
        nodes[2].remove();
        nodes[2].reinsert();

        assert!(nodes[0].is_valid());
        assert!(!nodes[0].is_empty());
        assert_eq!(nodes[0].next() as *const List, &nodes[2] as *const List);
        assert_eq!(nodes[0].previous() as *const List, &nodes[2] as *const List);
        assert_eq!(nodes[1].next() as *const List, &nodes[2] as *const List);
        assert_eq!(nodes[1].previous() as *const List, &nodes[0] as *const List);
        assert_eq!(nodes[2].next() as *const List, &nodes[0] as *const List);
        assert_eq!(nodes[2].previous() as *const List, &nodes[0] as *const List);

        nodes[1].reinsert();
        assert_eq!(nodes[0].next() as *const List, &nodes[1] as *const List);
        assert_eq!(nodes[0].previous() as *const List, &nodes[2] as *const List);
        assert_eq!(nodes[1].next() as *const List, &nodes[2] as *const List);
        assert_eq!(nodes[1].previous() as *const List, &nodes[0] as *const List);
        assert_eq!(nodes[2].next() as *const List, &nodes[0] as *const List);
        assert_eq!(nodes[2].previous() as *const List, &nodes[1] as *const List);
    }

    struct Header<'list> {
        list: SizedList<'list>,
        size: Cell<usize>,
    }

    impl<'list> Header<'list> {
        fn size(&self) -> usize {
            self.size.get()
        }
    }

    struct SizedList<'list> {
        link: Link<'list, Self>,
        parent: Cell<Option<&'list Header<'list>>>,
    }

    impl<'list> LinkedList<'list> for SizedList<'list> {
        fn link(&'list self) -> &'list Link<'list, Self> {
            &self.link
        }

        fn grow(&self) {
            let increased_size = self.parent.get().unwrap().size.get() + 1;
            self.parent.get().unwrap().size.set(increased_size);
        }

        fn shrink(&self) {
            let decreased_size = self.parent.get().unwrap().size.get() - 1;
            self.parent.get().unwrap().size.set(decreased_size);
        }
    }

    #[test]
    fn hooks() {
        let header = Header {
            list: SizedList {
                link: Link::uninitialized(),
                parent: Cell::from(None),
            },
            size: Cell::from(0),
        };

        let nodes: [SizedList; 2] = [
            SizedList{ link: Link::uninitialized(), parent: Cell::from(Some(&header)) },
            SizedList{ link: Link::uninitialized(), parent: Cell::from(Some(&header)) }
        ];

        header.list.connect_self();
        header.list.parent.set(Some(&header));
        header.list.prepend(&nodes[0]);
        header.list.prepend(&nodes[1]);

        assert_eq!(header.size(), 2);

        nodes[0].remove();
        assert_eq!(header.size(), 1);

        nodes[1].remove();
        assert_eq!(header.size(), 0);

        nodes[0].reinsert();
        assert_eq!(header.size(), 1);

        nodes[1].reinsert();
        assert_eq!(header.size(), 2);
    }
}