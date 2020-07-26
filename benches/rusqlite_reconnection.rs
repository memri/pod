use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use rusqlite::Connection;

/// Test the performance of opening an sqlite (rusqlite) connection,
/// accessing the DB and closing the connection.
fn open_file_connection() {
    let conn = Connection::open("target/criterion/deleteme.db").unwrap();
    let params: &[i64] = &[];
    let result = conn
        .execute("UPDATE test SET uid = 91 WHERE uid = 91;", params)
        .unwrap();
    assert_eq!(result, 0);
}

fn criterion_benchmark(c: &mut Criterion) {
    let db_address = "target/criterion/deleteme.db";
    let conn = Connection::open(db_address).unwrap();
    conn.execute_batch("CREATE TABLE IF NOT EXISTS test ( uid INTEGER );")
        .unwrap();
    conn.close().unwrap();

    c.bench_function("reopening SQLite connections in a loop", |b| {
        b.iter(|| open_file_connection())
    });

    std::fs::remove_file(db_address).unwrap();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
