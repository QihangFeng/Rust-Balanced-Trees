use std::fs::{File, create_dir_all};
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};

use project2::avl::AvlTree;
use project2::red_black::RedBlackTree;


fn main() -> std::io::Result<()> {
    let sizes = [10000, 40000, 70000, 100000, 130000];

    create_dir_all("benches")?;
    let mut w = BufWriter::new(File::create("benches/results.csv")?);
    writeln!(w, "structure,op,size,total_ms,ns_per_op")?;

    for &n in &sizes {
        let query_cnt = n / 10;

        // Red-Black Tree
        // 1) insert_inc
        let mut rb = RedBlackTree::<i32>::new();
        let start = Instant::now();
        for x in 0..(n as i32) {
            rb.insert(x);
        }
        let total = start.elapsed();
        write_row(&mut w, "RB", "insert_inc", n, total, n);

        // 2) search_low on the same tree
        let start = Instant::now();
        for x in 0..(query_cnt as i32) {
            let _ = rb.contains(x);
        }
        let total = start.elapsed();
        write_row(&mut w, "RB", "search_low", n, total, query_cnt);

        // 3) delete_inc on the same tree (delete all in increasing order)
        let start = Instant::now();
        for x in 0..(n as i32) {
            rb.delete(x);
        }
        let total = start.elapsed();
        write_row(&mut w, "RB", "delete_inc", n, total, n);
        writeln!(w)?;


        // AVL Tree
        // 1) insert_inc
        let mut avl = AvlTree::<i32>::new();
        let start = Instant::now();
        for x in 0..(n as i32) {
            avl.insert(x);
        }
        let total = start.elapsed();
        write_row(&mut w, "AVL", "insert_inc", n, total, n);

        // 2) search_low on the same tree
        let start = Instant::now();
        for x in 0..(query_cnt as i32) {
            let _ = avl.contains(x);
        }
        let total = start.elapsed();
        write_row(&mut w, "AVL", "search_low", n, total, query_cnt);

        // 3) delete_inc on the same tree
        let start = Instant::now();
        for x in 0..(n as i32) {
            avl.delete(x);
        }
        let total = start.elapsed();
        write_row(&mut w, "AVL", "delete_inc", n, total, n);
        writeln!(w)?;
    }

    w.flush()?;
    println!("Saved CSV to benches/results.csv");
    Ok(())
}

fn write_row(
    w: &mut BufWriter<File>,
    structure: &str,
    op: &str,
    size: usize,
    total: Duration,
    ops: usize,
) {
    let total_ms = dur_ms(total);
    let ns_per_op = (total.as_nanos() as f64) / (ops as f64);
    let _ = writeln!(
        w,
        "{},{},{},{:.3},{:.3}",
        structure, op, size, total_ms, ns_per_op
    );
}

fn dur_ms(d: Duration) -> f64 {
    (d.as_secs() as f64) * 1000.0 + (d.subsec_nanos() as f64) / 1_000_000.0
}
