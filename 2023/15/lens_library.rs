use aoc::{AdventOfCode, Itertools};

fn hash(s: &str) -> u8 {
    s.bytes()
        .fold(0, |acc: u8, byte| acc.wrapping_add(byte).wrapping_mul(17))
}

#[test]
fn hash_tests() {
    assert_eq!(hash("HASH"), 52);
}

const INP: &str = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";

struct HashMap<'l> {
    // if we're going to do lots of removals this might be better but... the
    // input is small and `LinkedList`'s useful APIs are still unstable
    // boxes: [LinkedList<(&'l str, u8)>; 256],
    boxes: [Vec<(&'l str, u8)>; 256],
}

impl<'l> HashMap<'l> {
    fn new() -> Self {
        Self {
            boxes: [(); 256].map(|()| Vec::new()),
        }
    }

    fn add(&mut self, label: &'l str, val: u8) -> Option<u8> {
        let hash = hash(label);
        let bucket = &mut self.boxes[hash as usize];

        for (label_slot, val_slot) in bucket.iter_mut() {
            if *label_slot == label {
                let old_val = *val_slot;
                *val_slot = val;
                // eprintln!("replace {label} = {val}");
                return Some(old_val);
            }
        }

        // otherwise, append:
        // eprintln!("insert {label} = {val}");
        bucket.push((label, val));
        None
    }

    fn remove(&mut self, label: &str) -> Option<u8> {
        let bucket = &mut self.boxes[hash(label) as usize];
        if let Some((idx, _)) = bucket.iter().find_position(|&&(l, _)| l == label) {
            let (_, val) = bucket.remove(idx);
            // eprintln!("replace {label}");
            Some(val)
        } else {
            None
        }
    }

    fn focusing_power(&self) -> usize {
        let mut power = 0;
        for (box_idx, b) in self.boxes.iter().enumerate() {
            for (slot_idx, &(_label, focal_len)) in b.iter().enumerate() {
                // eprintln!("{_label}: box {box_idx}, slot {slot_idx} = {focal_len}");
                power += (box_idx + 1) * (slot_idx + 1) * (focal_len as usize);
            }
        }

        power
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2023, 15);
    let inp = aoc.get_input();
    // let inp = INP;
    let steps = inp.lines().next().unwrap().split(',').collect_vec();

    let p1: usize = steps.iter().map(|s| hash(s)).map(|h| h as usize).sum();
    _ = aoc.submit_p1(p1);

    let p2 = {
        let steps = steps.into_iter().map(|s| {
            if let Some((label, focal_length)) = s.split_once('=') {
                (label, Some(focal_length.parse::<u8>().unwrap()))
            } else if let Some(label) = s.strip_suffix('-') {
                (label, None)
            } else {
                panic!()
            }
        });

        let mut hm = HashMap::new();
        steps.into_iter().for_each(|(label, val)| {
            if let Some(val) = val {
                hm.add(label, val);
            } else {
                hm.remove(label);
            }
        });

        hm.focusing_power()
    };
    _ = aoc.submit_p2(p2);
}
