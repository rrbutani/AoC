use std::{collections::VecDeque, iter, slice};

use aoc::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Entry {
    File(u8),
    Free(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Disk {
    map: Vec<Entry>,
}

impl FromStr for Disk {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = s.chars().enumerate().map(|(i, c)| {
            assert!(c.is_numeric(), "'{c}'");
            let n = c as u8 - b'0';

            use Entry::*;
            if i % 2 == 0 {
                File(n)
            } else {
                Free(n)
            }
        });

        Ok(Disk { map: map.collect() })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Block {
    File { id: usize },
    Free,
}

#[derive(Debug, Clone)]
struct EntryStateInDirection {
    entry: Option<Entry>,
    file_id: usize,
}

#[derive(Debug, Clone)]
struct BlockIter<It: DoubleEndedIterator + Iterator<Item = Entry>> {
    it: It,
    forward: EntryStateInDirection,
    rear: EntryStateInDirection,
}

impl EntryStateInDirection {
    fn next(
        &mut self,
        increment: bool,
        mut get_next: impl FnMut() -> Option<Entry>,
    ) -> Option<Block> {
        loop {
            match self.entry {
                // if empty, throw away the current entry: it's done
                Some(Entry::File(0) | Entry::Free(0)) => {
                    self.entry = None;
                }
                // if no entry, try to get an entry (or bail):
                None => {
                    let new = get_next()?;
                    if let Entry::File(_) = new {
                        if increment {
                            self.file_id += 1;
                        } else {
                            self.file_id -= 1;
                        }
                    }
                    self.entry = Some(new);
                }
                // otherwise, yield from the current entry:
                Some(Entry::File(n)) => {
                    self.entry = Some(Entry::File(n - 1));
                    return Some(Block::File { id: self.file_id });
                }
                Some(Entry::Free(n)) => {
                    self.entry = Some(Entry::Free(n - 1));
                    return Some(Block::Free);
                }
            }
        }
    }
}

impl<It: DoubleEndedIterator + Iterator<Item = Entry>> Iterator for BlockIter<It> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        // if we've exhausted the front, try stealing from anything currently in
        // queue on the rear:
        if let Some(f) = self.forward.next(true, || self.it.next()) {
            return Some(f);
        } else if self.rear.entry.is_some() {
            self.next_back()
        } else {
            None
        }
    }
}

impl<It: DoubleEndedIterator + Iterator<Item = Entry>> DoubleEndedIterator for BlockIter<It> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // if we're exhausted the rear, try stealing from anything currently in
        // queue on the front:
        if let Some(r) = self.rear.next(false, || self.it.next_back()) {
            return Some(r);
        } else if self.forward.entry.is_some() {
            self.next()
        } else {
            None
        }
    }
}

impl Disk {
    fn iter(&self) -> BlockIter<iter::Copied<slice::Iter<'_, Entry>>> {
        // NOTE: leaning on the property that `map` contains alternating `File`
        // and `Free` entries
        let mut it = self.map.iter().copied();
        let forward = it.next();
        let back = it.next_back();

        BlockIter {
            it,
            forward: EntryStateInDirection { entry: forward, file_id: 0 },
            rear: EntryStateInDirection { entry: back, file_id: self.map.len() / 2 },
        }
    }
}

fn checksum_after_compacting_by_block(d: &Disk) -> usize {
    let mut blocks = d.iter();
    let mut checksum = 0;
    let mut block_idx = 0;
    'compact: while let Some(slot) = blocks.next() {
        match slot {
            Block::File { id } => checksum += id * block_idx,
            Block::Free => {
                // take a block from the end to fill this slot
                let id = loop {
                    match blocks.next_back() {
                        Some(Block::Free) => continue,
                        Some(Block::File { id }) => break id,
                        None => break 'compact, // ran out of blocks, all done!
                    }
                };

                checksum += id * block_idx
            }
        }
        block_idx += 1;
    }

    checksum
}

#[derive(Default)]
struct ComputeChecksum {
    checksum: usize,
    block_num: usize,
}
impl ComputeChecksum {
    fn feed_file(&mut self, len: u8, id: usize) {
        /* for i in 0..len {
            self.checksum += (self.block_num + i) * id;
        } */
        // strength reduction
        let len = len as usize;
        let sum = |a, b| (b + a) * (b - a + 1) / 2;
        self.checksum += sum(self.block_num, self.block_num + len - 1) * id;
        self.block_num += len;

        #[cfg(debug_assertions)]
        {
            // eprint!("|");
            for _ in 0..len {
                eprint!("{id}");
            }
        }
    }
    fn feed_empty(&mut self, len: u8) {
        self.block_num += len as usize;
        #[cfg(debug_assertions)]
        {
            // eprint!("|");
            for _ in 0..len {
                eprint!(".")
            }
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for ComputeChecksum {
    fn drop(&mut self) {
        eprintln!()
    }
}

fn checksum_after_compacting_by_file(entries: &Disk) -> usize {
    let mut checksum = ComputeChecksum::default();
    let mut entries = VecDeque::from_iter(entries.map.iter().copied().enumerate());
    let mut trailing_entries = Vec::with_capacity(entries.len() / 2);
    'compact: loop {
        // drop leading `File` entries & zero-size `Free` entries:
        while let Some((_, Entry::File(_) | Entry::Free(0))) = entries.front() {
            if let (i, Entry::File(f)) = entries.pop_front().unwrap() {
                checksum.feed_file(f, i / 2);
            }
        }

        // drop trailing `Free` entries:
        while let Some((_, Entry::Free(_))) = entries.back() {
            trailing_entries.push(entries.pop_back().unwrap());
        }

        // if we've got ourselves a trailing `File` entry to process, scan
        // forwards to try to find a new home for it:
        let Some(&f @ (_, Entry::File(n))) = entries.back() else {
            break; // no more files to consider, bail
        };
        entries.pop_back();

        for (idx, &(id, ent)) in entries.iter().enumerate() {
            // if we find a new home for it, great
            if let Entry::Free(space) = ent {
                if let Some(extra) = space.checked_sub(n) {
                    // if it's an exact match in size, just swap:
                    if extra == 0 {
                        entries[idx] = f;
                    } else {
                        // otherwise, shrink the `Free` and insert the `File`
                        // before:
                        entries[idx] = (id, Entry::Free(extra));
                        entries.insert(idx, f);
                    }

                    // retire the free space that's now where `File` was:
                    trailing_entries.push((id, Entry::Free(n)));

                    continue 'compact;
                }
            }
        }

        // if we don't find a new home for the file, retire it to `unmoved` and
        // carry on:
        trailing_entries.push(f);
    }

    // feed in anything remaining in `entries` and everything that we weren't
    // able to find better spots for:
    for (i, ent) in entries
        .into_iter()
        .chain(trailing_entries.into_iter().rev())
    {
        match ent {
            Entry::File(n) => checksum.feed_file(n, i / 2),
            Entry::Free(n) => checksum.feed_empty(n),
        }
    }

    checksum.checksum
}

const EX: &str = "2333133121414131402";

fn main() {
    let mut aoc = AdventOfCode::new(2024, 9);
    let inp = aoc.get_input();
    // let inp = EX;
    let disk = inp.trim().parse().unwrap();

    let p1 = checksum_after_compacting_by_block(&disk);
    dbg!(p1);
    _ = aoc.submit_p1(p1);

    let p2 = checksum_after_compacting_by_file(&disk);
    dbg!(p2);
    _ = aoc.submit_p2(p2);
}

// TODO: perf
