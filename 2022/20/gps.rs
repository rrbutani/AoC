use std::ops::{Index, IndexMut};

use aoc::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct LinkedListNode<T> {
    val: T,
    id: usize,
    prev: Option<usize>,
    next: Option<usize>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct List<'a> {
    store: &'a mut Vec<LinkedListNode<isize>>,
    root_id: usize,
}

impl Display for List<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_list();
        let mut curr = self.root_id;
        loop {
            d.entry(&self.store[curr].val);
            curr = self.store[curr].next.unwrap();

            if curr == self.root_id {
                break;
            }
        }

        d.finish()
    }
}

impl<'a> List<'a> {
    fn from_str(s: &str, store: &'a mut Vec<LinkedListNode<isize>>) -> List<'a> {
        let it = s.lines().map(|l| l.parse().unwrap());
        Self::from_it(it, store)
    }

    fn from_it(
        mut it: impl Iterator<Item = isize>,
        store: &'a mut Vec<LinkedListNode<isize>>,
    ) -> List<'a> {
        store.reserve(it.size_hint().1.unwrap_or(100));

        let root = store.len();
        store.push(LinkedListNode {
            val: it.next().unwrap(),
            id: root,
            prev: None,
            next: None,
        });
        let mut prev_id = root;

        for num in it {
            let node = LinkedListNode {
                val: num,
                id: store.len(),
                prev: Some(prev_id),
                next: None,
            };

            store[prev_id].next = Some(node.id);

            prev_id = node.id;
            store.push(node);
        }

        store[prev_id].next = Some(root);
        store[root].prev = Some(prev_id);

        List {
            store,
            root_id: root,
        }
    }
}

impl Index<usize> for List<'_> {
    type Output = LinkedListNode<isize>;

    fn index(&self, index: usize) -> &Self::Output {
        let mut id = self.root_id;
        for _ in 0..index {
            id = self.store[id].next.unwrap();
        }

        &self.store[id]
    }
}

impl IndexMut<usize> for List<'_> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let mut id = self.root_id;
        for _ in 0..index {
            id = self.store[id].next.unwrap();
        }

        &mut self.store[id]
    }
}

impl List<'_> {
    fn mix(&mut self) {
        for id in 0..self.store.len() {
            if DBG {
                eprintln!();
            }
            // remove this node; set prev->next to next, next->prev to prev:
            let LinkedListNode {
                prev,
                next,
                val,
                id: id2,
            } = self.store[id];
            assert_eq!(id, id2);
            let prev = prev.unwrap();
            let next = next.unwrap();

            self.store[prev].next = Some(next);
            self.store[next].prev = Some(prev);

            // minus 1 to account for the fact that we've removed ourselves from
            // the loop
            let shift = val % ((self.store.len() - 1) as isize);
            // let shift = val;

            let step_func: fn(store: &Vec<LinkedListNode<_>>, id: usize) -> usize =
                if shift.is_positive() {
                    if DBG {
                        eprintln!("move forward");
                    }
                    |store, id| store[id].next.unwrap()
                } else {
                    if DBG {
                        eprintln!("move backward");
                    }
                    |store, id| store[id].prev.unwrap()
                };
            // insert before
            let mut curr = if shift.is_negative() {
                self.store[prev].next.unwrap()
            } else {
                next
            };

            // adjust root
            if id == self.root_id {
                match shift.signum() {
                    // root moves backwards, whatever was prev is now root
                    -1 => {
                        self.root_id = self.store[id].prev.unwrap();
                    }
                    0 => {}
                    // root moves forward, whatever was next is now root
                    1 => {
                        self.root_id = self.store[id].next.unwrap();
                    }
                    _ => unreachable!(),
                }
            }

            if DBG {
                dbg!(curr);
            }
            // let start_curr = curr;
            for _i in 0..shift.abs() {
                // if curr == start_curr {
                //     eprintln!("loop after {_i} iterations!");
                // }
                curr = step_func(self.store, curr);
                // if DBG { dbg!(curr); }
            }
            if DBG {
                dbg!(curr);
            }

            // insert before `curr`:
            let node = LinkedListNode {
                next: Some(curr),
                prev: self.store[curr].prev,
                id,
                val,
            };

            // prev -> new
            self.store[node.prev.unwrap()].next = Some(id);
            // new <- curr
            self.store[curr].prev = Some(id);

            // place new:
            self.store[id] = node;

            // eprintln!("moved {shift} [{id}]");
            // eprintln!("{self}");
        }
    }
}

impl List<'_> {
    fn find(&self, num: isize) -> Option<usize> {
        let mut curr = self.root_id;
        let mut idx = 0;
        loop {
            if self.store[curr].val == num {
                break Some(idx);
            }
            idx += 1;
            curr = self.store[curr].next.unwrap();

            if curr == self.root_id {
                break None;
            }
        }
    }

    fn map_val(&mut self, mut func: impl FnMut(usize, isize) -> isize) {
        let mut idx = 0;
        let mut curr = self.root_id;

        loop {
            let slot = &mut self.store[curr];
            slot.val = func(idx, slot.val);

            curr = slot.next.unwrap();
            idx += 1;

            if curr == self.root_id {
                break;
            }
        }
    }
}

const DBG: bool = false;

fn main() {
    let mut aoc = AdventOfCode::new(2022, 20);
    let inp = aoc.get_input();
    // let inp = include_str!("ex");

    let p1: isize = {
        let mut store = Vec::new();
        let mut l: List = List::from_str(&inp, &mut store);
        // dbg!(l);
        // eprintln!("{l}");
        l.mix();
        // eprintln!("{l}");

        let idx = l.find(0).unwrap();
        assert_eq!(l[idx].val, 0);
        [1000, 2000, 3000]
            .into_iter()
            .map(|i| dbg!(l[i + idx].val))
            .sum()
    };
    dbg!(p1);
    aoc.submit_p1(dbg!(p1)).unwrap();

    let p2: isize = {
        const KEY: isize = 811589153;
        let mut store = Vec::new();

        let it = inp
            .lines()
            .map(|l| l.parse().unwrap())
            .map(|v: isize| v * KEY);
        let mut l: List = List::from_it(it, &mut store);
        // l.map_val(|_, v| v * KEY);

        // eprintln!("{l:12}");
        for _ in 0..10 {
            // dbg!("yo");
            l.mix();
            // eprintln!("{l:12}");

            // l.map_val(|_, v| v / KEY);
            // eprintln!("{l:12}");
            // panic!();
        }

        let idx = l.find(0).unwrap();
        assert_eq!(l[idx].val, 0);
        [1000, 2000, 3000]
            .into_iter()
            .map(|i| dbg!(l[i + idx].val))
            .sum()
    };
    dbg!(p2);
    aoc.submit_p2(dbg!(p2)).unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn p1_small() {
        const EX_SMALL: &str = "1,1,1\n2,1,1\n";
        assert_eq!(super::p1(EX_SMALL), 10);
    }

    #[test]
    fn p1_ex() {
        assert_eq!(super::p1(include_str!("ex")), 64);
    }

    #[test]
    fn p2_ex() {
        assert_eq!(super::p2(include_str!("ex")), 58);
    }
}
