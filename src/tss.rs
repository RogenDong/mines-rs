use std::fmt::Write;
use std::ops::RangeInclusive;

use rand::Rng;

use crate::{MineMap, Position};

#[test]
fn ts_format() {
    let mut mines = MineMap::from(99, 30, 30);
    mines.shuffle(None);
    println!("{}\n", mines.format_str());

    mines.shuffle(Some(Position(11, 12)));
    println!("{}", mines.format_str());
}

#[test]
fn ts_move() {
    let (c, w, h) = (99, 15, 15);
    let width = w as usize;
    let height = h as usize;
    let mut mines = MineMap::from(c, w, h);
    mines.shuffle(None);

    let mut rng = rand::thread_rng();
    let (lx, ly, pf) = loop {
        let y = rng.gen_range(0..h);
        let x = rng.gen_range(0..w);
        if mines.get(x, y).is_mine() {
            break (limit(x, width), limit(y, height), Position(x, y));
        }
    };

    format_by_range(&mines, lx, ly);

    let pt = mines.move_mine_randomly(pf.0, pf.1);
    println!(" >>>>>>>> {pf} -- {pt} <<<<<<<< \n");

    format_by_range(&mines, limit(pt.0, width), limit(pt.1, height));
}

#[test]
fn get_empty_slots() {
    let (c, w, h) = (1024, 75, 75);
    let mut mines = MineMap::from(c, w, h);
    // mines.shuffle(None);
    let (x, y) = found(w, h, &mut mines);

    let mut buf = String::with_capacity(w as usize * h as usize * 3);
    let found = mines.get_nearby_empty_area(x, y);
    // let lim = Position(w, h);
    let sp = Position(x, y);
    println!("nearby {sp} empty slot: {}", found.len());
    for y in 0..mines.height {
        for x in 0..mines.width {
            let p = Position(x, y);
            let m = mines.get(x, y);
            let w = m.get_warn();
            if sp == p {
                buf.push_str(" @");
            } else if m.is_mine() {
                buf.push_str(" +");
            } else if found.contains(&p) {
                if w > 0 {
                    write!(buf, " {w}").unwrap();
                } else {
                    buf.push_str(" ·");
                }
            } else if m.is_empty() {
                buf.push_str(" ·");
            } else {
                buf.push_str("  ");
            }
        }
        buf.push('\n');
    }
    println!("{buf}");
}

fn limit(v: u8, max: usize) -> RangeInclusive<usize> {
    let mut mx = v + 2;
    let mut mi = 0;
    if v > 0 {
        mi = v - if v > 1 { 2 } else { 1 };
    } else if v as usize == (max - 1) {
        mx = v;
    } else if v as usize == (max - 2) {
        mx = v + 1;
    }
    (mi as usize)..=(mx as usize)
}

fn format_by_range(mines: &MineMap, lx: RangeInclusive<usize>, ly: RangeInclusive<usize>) {
    let (w, h) = (mines.width as usize, mines.height as usize);
    let mut buf = String::with_capacity(w * h * 3);
    buf.push_str("  ");
    for x in 0..mines.width {
        let mut t = x;
        while t > 9 {
            t %= 10;
        }
        write!(buf, " {t}").unwrap()
    }
    buf.push_str("\n  ");
    for _ in 0..mines.width {
        buf.push_str("__");
    }
    buf.push('\n');
    for y in 0..mines.height {
        for x in 0..mines.width {
            if x < 1 {
                let mut t = y;
                while t > 9 {
                    t %= 10;
                }
                write!(buf, "{t}|").unwrap()
            }
            let m = mines.get(x, y);
            if m.is_mine() {
                buf.push_str(" +");
                continue;
            }
            match m.get_warn() {
                0 => buf.push_str("  "),
                w => {
                    if lx.contains(&(x as usize)) && ly.contains(&(y as usize)) {
                        write!(buf, " {w}").unwrap()
                    } else {
                        buf.push_str("  ")
                    }
                }
            } // match warn
        }
        buf.push('\n');
    }
    println!("{buf}");
}

fn found(w: u8, h: u8, mines: &mut MineMap) -> (u8, u8) {
    let lim = Position(w, h);
    loop {
        mines.shuffle(None);
        for y in 1..(h - 1) {
            for x in 1..(w - 1) {
                if mines.get(x, y).get_warn() != 1 {
                    continue;
                }
                let p = Position(x, y);
                let a = p
                    .get_around(lim)
                    .into_iter()
                    .filter(|p| mines.get_by_pos(*p).is_empty())
                    .count();
                let n = p
                    .get_nearby(lim)
                    .into_iter()
                    .filter(|p| mines.get_by_pos(*p).is_empty())
                    .count();
                if n < 1 && a == 2 {
                    return (x, y);
                }
            } // for: x
        } // for: y
    } // loop
}
