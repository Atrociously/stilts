#[macro_use]
extern crate criterion;

use stilts::Template;
use criterion::Criterion;
use criterion::Bencher;

criterion_main!(benches);
criterion_group!(benches, all_benches);

fn all_benches(c: &mut Criterion) {
    c.bench_function("Big Table", |b| big_table(b, 100));
    c.bench_function("Teams", teams);
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

fn big_table(b: &mut Bencher, size: usize) {
    let mut table = Vec::with_capacity(size);
    for _ in 0..size {
        let mut inner = Vec::with_capacity(size);
        for i in 0..size {
            inner.push(i);
        }
        table.push(inner);
    }
    let ctx = BigTable { table };
    b.iter(|| ctx.render().unwrap());
}

fn teams(b: &mut Bencher) {
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
    b.iter(|| teams.render().unwrap());
}
