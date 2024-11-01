use axum::Json;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use mozart::{
    model::{Parameter, ParameterType, Submission, TestCase},
    submit,
};
use tokio::runtime::Runtime;

fn pass(c: &mut Criterion) {
    let mut test_cases = Vec::with_capacity(100);
    for id in 0..100 {
        let test_case = TestCase {
            id,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("5"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("5"),
            }]),
        };

        test_cases.push(test_case);
    }

    let submission = Submission {
        solution: [
            "solution x =",
            "  if x < 0",
            "    then x * (-1)",
            "    else x",
        ]
        .join("\n"),
        test_cases: test_cases.into_boxed_slice(),
    };

    c.bench_function("pass baseline", |b| {
        b.to_async(Runtime::new().expect("failed to initialise tokio runtime"))
            .iter_batched(
                || Json(submission.clone()),
                |submission: Json<Submission>| submit(black_box(submission)),
                BatchSize::SmallInput,
            )
    });
}

fn fail(c: &mut Criterion) {
    let mut test_cases = Vec::with_capacity(100);
    for id in 0..100 {
        let test_case = TestCase {
            id,
            input_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("5"),
            }]),
            output_parameters: Box::new([Parameter {
                value_type: ParameterType::Int,
                value: String::from("5"),
            }]),
        };

        test_cases.push(test_case);
    }

    let submission = Submission {
        solution: String::from("solution x = x"),
        test_cases: test_cases.into_boxed_slice(),
    };

    c.bench_function("fail baseline", |b| {
        b.to_async(Runtime::new().expect("failed to initialise tokio runtime"))
            .iter_batched(
                || Json(submission.clone()),
                |submission: Json<Submission>| submit(black_box(submission)),
                BatchSize::SmallInput,
            )
    });
}

criterion_group!(benches, pass, fail);
criterion_main!(benches);
