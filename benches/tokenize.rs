#[macro_use]
extern crate criterion;

extern crate lax;

use criterion::Criterion;

use std::fs::File;
use std::io::prelude::*;
use std::iter;

use lax::tokenize::*;

fn tokenize_benchmark(c: &mut Criterion) {
   let mut f = File::open("lax/tokenize.lax").expect("file not found");

   let mut source = String::new();
   f.read_to_string(&mut source)
      .expect("reading the file failed");

   let min = 2usize.pow(12);
   let count = 1 + (min - 1) / source.len();

   assert!(count * source.len() >= min && (count - 1) * source.len() < min);

   let input = iter::repeat(source)
      .take(count)
      .collect::<Vec<_>>()
      .join("\n");

   let lines = input.matches('\n').count();
   println!("{} lines/{} kb", lines, input.len() / 1024);

   let chars: Vec<_> = input.chars().collect();

   c.bench_function("tokenize", move |b| b.iter(|| tokenize(&chars)));
}

criterion_group!(tokenize_group, tokenize_benchmark);
criterion_main!(tokenize_group);
