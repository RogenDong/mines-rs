use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::seq::SliceRandom;
use rand::{thread_rng};
use mines::{map::MineMap, location::Loc, cell::Cell};

const MAX_LEN: usize = 255 * 255;
const COUNT_MINES_TS: usize = 52428;

fn ts_fill(cri: &mut Criterion) {
    let mut v = vec![123; MAX_LEN];
    cri.bench_function("fill", |b| b.iter(|| {
        v.fill(black_box(0));
        v[..COUNT_MINES_TS].fill(black_box(9));
    }));
}

fn ts_random_shuffle(cri: &mut Criterion) {
    let mut rng = thread_rng();
    let mut v = vec![123; MAX_LEN];
    v.fill(0);
    v[..COUNT_MINES_TS].fill(9);

    cri.bench_function("rand_shuffle", |b| b.iter(|| v.shuffle(&mut rng)));
}

fn ts_mines_shuffle(cri: &mut Criterion) {
    let mut mines = MineMap::new(COUNT_MINES_TS as u16, 255, 255);
    cri.bench_function("mines_shuffle", |b| b.iter(|| mines.shuffle(None)));
    // println!("{}", mines.format_str());
}

fn ts_bmp_warn(cri: &mut Criterion) {
    #[inline]
    fn get_index(x: usize, y: usize) -> Option<usize> {
        if x < 255 && y < 255 {
            Some(y * 255 + x)
        } else {
            None
        }
    }
    #[inline]
    fn get(map: &Vec<u8>, x: usize, y: usize) -> Option<Cell> {
        Some(Cell(*map.get(get_index(x, y)?)?))
    }
    #[inline]
    fn get_mut(map: &mut Vec<u8>, x: usize, y: usize) -> Option<&mut u8> {
        if x < 255 && y < 255 {
            Some(map.get_mut(y * 255 + x)?)
        } else {
            None
        }
    }
    let mut rng = thread_rng();
    let mut v = vec![123; MAX_LEN];
    v.fill(0);
    v[..COUNT_MINES_TS].fill(9);
    v.shuffle(&mut rng);

    cri.bench_function("bmp", |b| b.iter(|| {
        for y in 0..255 {
            for x in 0..255 {
                let Some(c) = get(&v, x, y) else {
                    panic!("invalid location !!! ({x}, {y}) !!!")
                };
                if !c.is_mine() {
                    continue;
                }
                for Loc(ax, ay) in Loc(x as u8, y as u8).get_around() {
                    if let Some(av) = get_mut(&mut v, ax as usize, ay as usize) {
                        *av += 1;
                    }
                }
            }
        }
    }));
}

#[allow(unused)]
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

criterion_group!(benches, ts_fill, ts_random_shuffle, ts_mines_shuffle, ts_bmp_warn);
// criterion_group!(benches, ts_emp_area);
criterion_main!(benches);
