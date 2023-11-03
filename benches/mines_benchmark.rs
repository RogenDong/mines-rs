#![allow(unused)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mines::{cell::Cell, mmap::MineMap, location::Loc};
use rand::{seq::SliceRandom, thread_rng};

const WIDTH: usize = 255;
const HEIGHT: usize = 255;
const MAX_LEN: usize = WIDTH * HEIGHT;
const COUNT_MINES_TS: usize = MAX_LEN / 2;

fn ts_fill(cri: &mut Criterion) {
    let mut v = vec![123; MAX_LEN];
    cri.bench_function("fill", |b| {
        b.iter(|| {
            v.fill(black_box(0));
            v[..COUNT_MINES_TS].fill(black_box(9));
        })
    });
}

fn ts_random_shuffle(cri: &mut Criterion) {
    let mut rng = thread_rng();
    let mut v = vec![123; MAX_LEN];
    v.fill(0);
    v[..COUNT_MINES_TS].fill(9);

    cri.bench_function("rand_shuffle", |b| b.iter(|| v.shuffle(&mut rng)));
}

#[inline]
pub fn get_idx(x: usize, y: usize) -> Option<usize> {
    if x < WIDTH && y < HEIGHT {
        Some(y * WIDTH + x)
    } else {
        None
    }
}

fn ts_bmp_warn(cri: &mut Criterion) {
    let mut rng = thread_rng();
    let mut map = vec![123; MAX_LEN];
    map.fill(0);
    map[..COUNT_MINES_TS].fill(9);
    map.shuffle(&mut rng);

    cri.bench_function("bmp", |b| {
        b.iter(|| {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let Some(i) = get_idx(x, y) else {continue};
                    let w = map[i];
                    if w > 8 {
                        for a in Loc::from(x, y).get_around() {
                            let (ax, ay) = (a.0 as usize, a.1 as usize);
                            if let Some(ia) = get_idx(ax, ay) {
                                *(map.get_mut(ia).unwrap()) += 1;
                            }
                        }
                    }
                }
            }
        })
    });
}

#[allow(non_snake_case)]
pub fn get_bmp_idx(x: usize, y: usize) -> Option<[usize; 9]> {
    let i = get_idx(x, y)?;
    if i == 0 {
        return Some([0, 0, 0, 0, 0, 1, 0, WIDTH, HEIGHT + 1]);
    }
    let (iN, iW, iE, iS) = (
        if i < WIDTH { 0 } else { i - WIDTH },
        i - 1,
        i + 1,
        i + WIDTH,
    );
    if i == MAX_LEN - 1 {
        return Some([iN - 1, iN, 0, iW, 0, 0, 0, 0, 0]);
    }
    //  A N B | A=N-1 N=i-w B=N+1
    //  W   E | W=i-1       E=i+1
    //  C S D | C=S-1 S=i+w D=S+1
    const N: usize = 1;
    const W: usize = 3;
    const E: usize = 5;
    const S: usize = 7;
    let mut ls = [
        if iN == 0 { 0 } else { iN - 1 },
        iN,
        iN + 1,
        iW,
        0,
        iE,
        iS - 1,
        iS,
        iS + 1,
    ];
    if x == 0 {
        ls[W] = 0;
        ls[N - 1] = 0;
        ls[S - 1] = 0;
    } else if x == WIDTH - 1 {
        ls[E] = 0;
        ls[N + 1] = 0;
        ls[S + 1] = 0;
    }
    if y == 0 {
        ls[N] = 0;
        ls[N - 1] = 0;
        ls[N + 1] = 0;
    } else if y == HEIGHT - 1 {
        ls[S] = 0;
        ls[S - 1] = 0;
        ls[S + 1] = 0;
    }
    Some(ls)
}
fn ts_bmp_warn_by_index(cri: &mut Criterion) {
    let mut rng = thread_rng();
    let mut map = vec![123u8; MAX_LEN];
    map.fill(0);
    map[..COUNT_MINES_TS].fill(9);
    map.shuffle(&mut rng);

    cri.bench_function("bmp_by_index", |b| {
        b.iter(|| {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    if let Some(i) = get_idx(x, y) {
                        if map[i] > 8 {
                            if let Some(ls) = get_bmp_idx(x, y) {
                                for ii in ls {
                                    if ii > 0 {
                                        *map.get_mut(ii).unwrap() += 1;
                                    }
                                }
                            } // get around
                        } // found mine
                    } // get index
                } // loop column
            } // loop row
        })
    });
}

// fn ts_mines_shuffle(cri: &mut Criterion) {
//     let mut mines = MineMap::new(COUNT_MINES_TS as u16, WIDTH as u8, HEIGHT as u8);
//     cri.bench_function("fill + shuffle + bmp", |b| b.iter(|| mines.shuffle(None)));
// }

fn ts_emp_area(cri: &mut Criterion) {
    let mut rng = thread_rng();
    let mut mines = MineMap::new(160, 20, 10);
    mines.shuffle(None);
    todo!()
    // cri.bench_function("emp", |b| b.iter(|| {
    //     for _ in 0..3 {
    //         mines.get_nearby_empty_area(rng.gen_range(0..20), rng.gen_range(0..10));
    //     }
    // }));
}

criterion_group!(
    benches,
    ts_fill,
    ts_random_shuffle,
    ts_bmp_warn,
    ts_bmp_warn_by_index,
    // ts_mines_shuffle
);
// criterion_group!(benches, ts_emp_area);
criterion_main!(benches);
