use std::{cell::RefCell, collections::HashMap, sync::Arc};

use aoc::*;

// it'd be cute to write this one using shell utilities (i.e. parse the input
// and map it to `mkdir` and `touch`, then run `du`..)

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct Path<'a>(Vec<&'a str>);

impl<'p> Path<'p> {
    fn push(&mut self, seg: &'p str) {
        self.0.push(seg)
    }

    fn pop(&mut self) {
        self.0.pop().unwrap();
    }

    fn with(&self, seg: &'p str) -> Self {
        let mut new = self.clone();
        new.push(seg);
        new
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry<'a> {
    Dir {
        name: &'a str,
        entries: Vec<Ent<'a>>,
        size_sum: Option<usize>, // caching
    },
    File {
        name: &'a str,
        size: usize,
    },
}

impl Entry<'_> {
    fn dir(name: &str) -> Ent<'_> {
        Arc::new(RefCell::new(Entry::Dir {
            name,
            entries: vec![],
            size_sum: None,
        }))
    }
    fn file(name: &str, size: usize) -> Ent<'_> {
        Arc::new(RefCell::new(Entry::File { name, size }))
    }

    fn push_into_dir<'e, 'p>(this: &'e Ent<'p>, e: Ent<'p>) -> Ent<'p> {
        let mut this = this.borrow_mut();
        if let Entry::Dir { entries, .. } = &mut *this {
            entries.push(e.clone())
        } else {
            panic!("cannot add entries to a file")
        }

        e
    }

    fn is_dir(this: &Ent<'_>) -> bool {
        let this = this.borrow();
        matches!(*this, Entry::Dir { .. })
    }

    #[allow(unused)]
    fn is_file(this: &Ent<'_>) -> bool {
        let this = this.borrow();
        matches!(*this, Entry::File { .. })
    }

    fn display<'s>(this: &'s Ent<'s>) -> impl Display + 's {
        EntryDisplayHelper {
            inner: this,
            level: 0,
        }
    }

    fn size(this: &Ent<'_>) -> usize {
        let mut this = this.borrow_mut();
        match &mut *this {
            Entry::Dir {
                entries, size_sum, ..
            } => {
                if let Some(s) = size_sum {
                    *s
                } else {
                    let sum = entries.iter().map(Entry::size).sum();
                    *size_sum = Some(sum);
                    sum
                }
            }
            Entry::File { size, .. } => *size,
        }
    }
}

type Ent<'a> = Arc<RefCell<Entry<'a>>>;

struct EntryDisplayHelper<'e, 'p> {
    inner: &'e Ent<'p>,
    level: usize,
}
impl<'e, 'p> Display for EntryDisplayHelper<'e, 'p> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.level {
            // TODO: use variable padding instead of this loop
            write!(f, "  ")?;
        }
        write!(f, "- ")?;

        let this = self.inner.borrow();
        match &*this {
            Entry::Dir { name, entries, .. } => {
                writeln!(f, "{name} (dir)")?;
                for e in entries {
                    write!(
                        f,
                        "{}",
                        EntryDisplayHelper {
                            inner: &e,
                            level: self.level + 1
                        }
                    )?;
                }
            }
            Entry::File { name, size } => {
                writeln!(f, "{name} (file, size={size})")?;
            }
        }

        Ok(())
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2022, 7);
    let inp = aoc.get_input();

    //     let inp = "$ cd /
    // $ ls
    // dir a
    // 14848514 b.txt
    // 8504156 c.dat
    // dir d
    // $ cd a
    // $ ls
    // dir e
    // 29116 f
    // 2557 g
    // 62596 h.lst
    // $ cd e
    // $ ls
    // 584 i
    // $ cd ..
    // $ cd ..
    // $ cd d
    // $ ls
    // 4060174 j
    // 8033020 d.log
    // 5626152 d.ext
    // 7214296 k";

    let mut inp = inp.lines();

    let mut table: HashMap<Path<'_>, Ent<'_>> = HashMap::new();
    let root = {
        assert_eq!(inp.next(), Some("$ cd /"));
        let mut curr_path = Path::default();
        let root = Entry::dir("/");
        table.insert(curr_path.clone(), root.clone());

        while let Some(line) = inp.next() {
            if let Some(cmd) = line.strip_prefix("$ ") {
                if cmd == "ls" {
                    // do nothing, we'll parse the entries once we come around
                } else if let Some(seg) = cmd.strip_prefix("cd ") {
                    if seg == ".." {
                        curr_path.pop();
                    } else {
                        curr_path.push(seg);
                    }
                } else {
                    panic!("invalid cmd: {cmd}");
                }
            } else if let Some((l, name)) = line.split_once(" ") {
                // we don't bother checking if the previous line was indeed an
                // ls
                if l == "dir" {
                    let dir = Entry::push_into_dir(&table[&curr_path], Entry::dir(name));
                    table.insert(curr_path.with(name), dir);
                } else if let Ok(size) = l.parse() {
                    let file = Entry::push_into_dir(&table[&curr_path], Entry::file(name, size));
                    table.insert(curr_path.with(name), file);
                } else {
                    panic!("invalid output: {line}")
                }
            } else {
                panic!("invalid line: {line}")
            }
        }

        root
    };

    println!("{}", Entry::display(&root));
    let p1: usize = table
        .values()
        .filter(|s| Entry::size(s) <= 100_000 && Entry::is_dir(s))
        .inspect(|s| {
            dbg!(s);
        })
        .map(Entry::size)
        .sum();
    dbg!(p1);
    aoc.submit_p1(p1).unwrap();

    const TOTAL_SPACE: usize = 70_000_000;
    const REQ_FREE_SPACE: usize = 30_000_000;
    let current_free_space = TOTAL_SPACE - Entry::size(&root);
    let need_to_clear = REQ_FREE_SPACE - current_free_space;

    let p2 = table
        .values()
        .filter(|e| Entry::is_dir(e))
        .map(Entry::size)
        .sorted()
        .find(|&s| s >= need_to_clear)
        .unwrap();
    dbg!(p2);
    aoc.submit_p2(p2).unwrap();
}
