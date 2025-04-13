use atspi_common::Role;
use criterion::{
    black_box, criterion_group, criterion_main, measurement::Measurement, BenchmarkGroup,
    Criterion, Throughput,
};
use indextree_method_structural_nav::{A11yNode, Tree, TreeCount, TreeTraversal};
use rayon::iter::ParallelIterator;
use serde_json::from_str;
use std::time::Duration;

const SYNTH_FN: &str = "../../data/synthetic.json";
const REAL_FN: &str = "../../data/single-page-html-spec.json";

fn seq_bench<M: Measurement, T: TreeTraversal>(mut g: BenchmarkGroup<'_, M>, t: &T, synth: bool) {
    g.throughput(Throughput::Elements(1_u64));
    g.sample_size(200);
    if synth {
        g.measurement_time(Duration::from_secs(150));
    } else {
        g.measurement_time(Duration::from_secs(30));
    }
    g.bench_function("find_first", |b| {
        b.iter(|| {
            // technically black box knowledge here; the largest item ID = 129
            let role_id = rand::random_range(0..=129);
            let role = Role::try_from(role_id).expect("Valid role ID!");
            let x = t.find_first(role);
            black_box(x);
        })
    });
    g.bench_function("iter_leafs", |b| {
        b.iter(|| {
            t.iter_leafs().for_each(|x| {
                black_box(x);
            });
        })
    });
    g.bench_function("how_many", |b| {
        b.iter(|| {
            // technically black box knowledge here; the largest item ID = 129
            let role_id = rand::random_range(0..=129);
            let role = Role::try_from(role_id).expect("Valid role ID!");
            let x = t.how_many(role);
            black_box(x);
        })
    });
    g.bench_function("how_many_roleset", |b| {
        b.iter(|| {
            // technically black box knowledge here; the largest item ID = 129
            let role_id = rand::random_range(0..=129);
            let role = Role::try_from(role_id).expect("Valid role ID!");
            let x = t.how_many_roleset(role);
            black_box(x);
        })
    });
    g.bench_function("max_dpeth", |b| {
        b.iter(|| {
            let x = t.max_depth();
            black_box(x);
        })
    });
    g.bench_function("unique_roles", |b| {
        b.iter(|| {
            let x = t.unique_roles();
            black_box(x);
        })
    });
    g.bench_function("unique_roles_roleset", |b| {
        b.iter(|| {
            let x = t.unique_roles_roleset();
            black_box(x);
        })
    });
    g.bench_function("find_first_roleset", |b| {
        b.iter(|| {
            // technically black box knowledge here; the largest item ID = 129
            let role_id = rand::random_range(0..=129);
            let role = Role::try_from(role_id).expect("Valid role ID!");
            let x = t.find_first_roleset(role);
            black_box(x);
        })
    });
    g.bench_function("find_first_stack", |b| {
        b.iter(|| {
            // technically black box knowledge here; the largest item ID = 129
            let role_id = rand::random_range(0..=129);
            let role = Role::try_from(role_id).expect("Valid role ID!");
            let x = t.find_first_stack(role);
            black_box(x);
        })
    });
    g.finish()
}
fn par_bench<M: Measurement, T: TreeTraversal>(mut g: BenchmarkGroup<'_, M>, t: &T, synth: bool) {
    g.throughput(Throughput::Elements(1_u64));
    g.sample_size(200);
    if synth {
        g.measurement_time(Duration::from_secs(60));
    } else {
        g.measurement_time(Duration::from_secs(15));
    }
    g.bench_function("par_iter_leafs", |b| {
        b.iter(|| {
            t.par_iter_leafs().for_each(|x| {
                black_box(x);
            });
        })
    });
    g.bench_function("par_how_many", |b| {
        b.iter(|| {
            // technically black box knowledge here; the largest item ID = 129
            let role_id = rand::random_range(0..=129);
            let role = Role::try_from(role_id).expect("Valid role ID!");
            let x = t.par_how_many(role);
            black_box(x);
        })
    });
    g.bench_function("par_how_many_roleset", |b| {
        b.iter(|| {
            // technically black box knowledge here; the largest item ID = 129
            let role_id = rand::random_range(0..=129);
            let role = Role::try_from(role_id).expect("Valid role ID!");
            let x = t.par_how_many_roleset(role);
            black_box(x);
        })
    });
    g.bench_function("par_max_dpeth", |b| {
        b.iter(|| {
            let x = t.par_max_depth();
            black_box(x);
        })
    });
    g.bench_function("par_unique_roles", |b| {
        b.iter(|| {
            let x = t.par_unique_roles();
            black_box(x);
        })
    });
    g.bench_function("par_find_first", |b| {
        b.iter(|| {
            // technically black box knowledge here; the largest item ID = 129
            let role_id = rand::random_range(0..=129);
            let role = Role::try_from(role_id).expect("Valid role ID!");
            let x = t.par_find_first(role);
            black_box(x);
        })
    });
    g.bench_function("par_find_first_roleset", |b| {
        b.iter(|| {
            // technically black box knowledge here; the largest item ID = 129
            let role_id = rand::random_range(0..=129);
            let role = Role::try_from(role_id).expect("Valid role ID!");
            let x = t.par_find_first_roleset(role);
            black_box(x);
        })
    });
    g.finish()
}

fn benchmarks(c: &mut Criterion) {
    let real_data = std::fs::read_to_string(REAL_FN).expect("Valid file");
    let synth_data = std::fs::read_to_string(SYNTH_FN).expect("Valid file");

    let real_tree: A11yNode = from_str(&real_data).expect("Valid JSON data!");
    let synth_tree: A11yNode = from_str(&synth_data).expect("Valid JSON data!");
    let real_tree_plain = Tree::from_root_node(real_tree.clone());
    let real_tree_count = TreeCount::from_root_node(real_tree);
    let synth_tree_plain = Tree::from_root_node(synth_tree.clone());
    let synth_tree_count = TreeCount::from_root_node(synth_tree);

    {
        let b = c.benchmark_group("real/tree/parallel");
        par_bench(b, &real_tree_plain, false);
    }
    {
        let b = c.benchmark_group("real/tree/sequential");
        seq_bench(b, &real_tree_plain, false);
    }
    {
        let b = c.benchmark_group("real/count_tree/parallel");
        par_bench(b, &real_tree_count, false);
    }
    {
        let b = c.benchmark_group("real/count_tree/sequential");
        seq_bench(b, &real_tree_count, false);
    }
    {
        let b = c.benchmark_group("synth/tree/parallel");
        par_bench(b, &synth_tree_plain, true);
    }
    {
        let b = c.benchmark_group("synth/tree/sequential");
        seq_bench(b, &synth_tree_plain, true);
    }
    {
        let b = c.benchmark_group("synth/count_tree/parallel");
        par_bench(b, &synth_tree_count, true);
    }
    {
        let b = c.benchmark_group("synth/count_tree/sequential");
        seq_bench(b, &synth_tree_count, true);
    }
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
