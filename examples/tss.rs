use std::io::{stdin, stdout, Write};

use mines::{location::Loc, mmap::MineMap};
// use rand::{seq::SliceRandom, thread_rng, Rng};
// use std::fmt::Write;
// use smallvec::SmallVec;

const TIP: &str = "input: <do x y>\ndo: r  reveal\n    f  flag\n    a  reveal around\n    c  count around\n    q  quit";

fn main() {
    let args: Vec<u16> = std::env::args()
        .filter_map(|a| a.parse::<u16>().ok())
        .collect();
    let [w, h, c] = args[..] else {
        panic!("args: <width, height, count>")
    };
    let mut mines = MineMap::new(c, w as u8, h as u8).unwrap();

    println!("{TIP}");

    let mut input = String::with_capacity(16);
    let mut out = stdout();
    let cin = stdin();
    let mut flag = 0;
    loop {
        print!("> ");
        out.flush().unwrap();
        cin.read_line(&mut input).unwrap();
        let inp = input.trim().to_string();
        input.clear();
        if inp.is_empty() {
            continue;
        }
        if inp == "q" {
            return;
        }
        let [o, x, y] = inp.split(' ').collect::<Vec<&str>>()[..] else {
            panic!("must {}", TIP)
        };
        let x: usize = x.parse().unwrap();
        let y: usize = y.parse().unwrap();
        match o {
            "c" => {
                let c = mines.count_flagged_around(x, y);
                println!("count around flag: {c}");
                continue;
            }
            "r" => {
                if flag == 0 {
                    mines.new_game(Some(Loc(x as u8, y as u8)));
                    println!("{}", mines.format_str());
                    mines.reveal_around(x, y);
                    flag += 1;
                } else {
                    mines.reveal(x, y);
                }
            }
            "a" => {
                let _ = mines.reveal_around(x, y);
            }
            "f" => mines.switch_flag(x, y),
            _ => {}
        }
        println!("{}", mines.format_stat_str());
    }
    // mines.new_game(Some(Loc(x, y)));
    // for _ in 0..32 {
    //     mines.new_game(None);
    // }
    // println!("{}", mines.format_str());
    // print_format(&mines);

    // mines.switch_flag(0, 0);
    // mines.open(0, 0);
    // let mines::cell::Cell(v) = mines.get(0, 0).unwrap();
    // println!("first: {:->8b}", v);
    // // copy
    // let cp = mines.export(false);
    // let cp = MineMap::by(cp, true).unwrap();
    // // println!("{}", cp.format_str());
    // let mines::cell::Cell(v) = cp.get(0, 0).unwrap();
    // println!("copy first: {:0>8b}", v);
}

// fn print_format(map: &MineMap) {
//     let (w, h, size) = map.my_size();
//     use std::fmt::Write;
//     let mut ln = 0;
//     for rr in &map.blanks {
//         let mut buf = String::with_capacity(size * 2 + h);
//         for i in 0..size {
//             let v = map.map[i];
//             if rr.contains(&i) {
//                 match v {
//                     0 => buf.push_str("  "),
//                     1..=8 => write!(buf, " {v}").unwrap(),
//                     _ => buf.push_str(" ·"),
//                 }
//             } else {
//                 buf.push_str(" ·");
//             }
//             if ln < w - 1 {
//                 ln += 1;
//             } else {
//                 buf.push('\n');
//                 ln = 0;
//             }
//         }
//         println!("{}\n", buf);
//     }
// }
