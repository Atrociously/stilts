use stilts::Template;

fn main() {
    divan::main();
}

#[derive(Template)]
#[stilts(path = "big-table.html")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

#[derive(Template)]
#[stilts(path = "teams.html")]
struct Teams {
    year: u16,
    teams: Vec<Team>,
}

struct Team {
    name: String,
    score: u8,
}

const SIZES: &[usize] = &[1, 2, 8, 16, 32, 64, 128, 256, 512, 1024];

#[divan::bench(consts = SIZES)]
fn big_table<const SIZE: usize>(b: divan::Bencher) {
    let mut table = Vec::with_capacity(SIZE);
    for _ in 0..SIZE {
        let mut inner = Vec::with_capacity(SIZE);
        for i in 0..SIZE {
            inner.push(i);
        }
        table.push(inner);
    }
    let ctx = BigTable { table };
    b.bench_local(|| ctx.render().unwrap());
}

#[divan::bench]
fn teams(b: divan::Bencher) {
    let teams = Teams {
        year: 2015,
        teams: vec![
            Team {
                name: "Jiangsu".into(),
                score: 43,
            },
            Team {
                name: "Beijing".into(),
                score: 27,
            },
            Team {
                name: "Guangzhou".into(),
                score: 22,
            },
            Team {
                name: "Shandong".into(),
                score: 12,
            },
        ],
    };
    b.bench_local(|| teams.render().unwrap());
}
