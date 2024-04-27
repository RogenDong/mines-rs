#![allow(unused)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mines::{cell::Cell, location::Loc, mmap::MineMap};
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

#[allow(non_snake_case)]
pub fn get_bmp_idx(x: usize, y: usize) -> Option<[usize; 9]> {
    const M: usize = usize::MAX - 1;
    let mut i = get_idx(x, y)?;
    if i == 0 {
        return Some([M, M, M, M, M, 1, M, WIDTH, HEIGHT + 1]);
    }
    if i >= MAX_LEN {
        i = M;
    }
    let (iN, iW, iE, iS) = (
        if i < WIDTH { M } else { i - WIDTH },
        i - 1,
        i + 1,
        if i >= MAX_LEN { M } else { i + WIDTH },
    );
    if i == MAX_LEN - 1 {
        return Some([iN - 1, iN, M, iW, M, M, M, M, M]);
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
        M,
        iE,
        iS - 1,
        iS,
        iS + 1,
    ];
    if x == 0 {
        ls[W] = M;
        ls[N - 1] = M;
        ls[S - 1] = M;
    } else if x == WIDTH - 1 {
        ls[E] = M;
        ls[N + 1] = M;
        ls[S + 1] = M;
    }
    if y == 0 {
        ls[N] = M;
        ls[N - 1] = M;
        ls[N + 1] = M;
    } else if y == HEIGHT - 1 {
        ls[S] = M;
        ls[S - 1] = M;
        ls[S + 1] = M;
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
                    if map.get(get_idx(x, y).unwrap()).map_or(0, |&c| c) < 9 {
                        continue;
                    }
                    // get around
                    let Some(ls) = get_bmp_idx(x, y) else {
                        continue;
                    };
                    for i in ls {
                        if i < MAX_LEN {
                            map[i] += 1;
                        }
                    }
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
    // ts_fill,
    // ts_random_shuffle,
    ts_bmp_warn_by_index,
    // ts_mines_shuffle
);
// criterion_group!(benches, ts_emp_area);
criterion_main!(benches);
