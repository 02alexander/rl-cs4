use criterion::{black_box, criterion_group, criterion_main, Criterion};

extern crate gamesolver;
use gamesolver::connect4::{Connect4};
use gamesolver::search::{abpruning_action_values};
use gamesolver::evaluators::{ConsequtiveEval, LinesEval, SimpleEval};


fn consecutive_eval_benchmark(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_move(action);
    }
    let p = board.cur_player;
    let evaluator = ConsequtiveEval::new();
    c.bench_function("ConsecutiveEval, depth=5", |b| b.iter(||{
        black_box(abpruning_action_values(&mut board, 4, &evaluator, p))
    }));
}

fn lines_eval_benchmark(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_move(action);
    }
    let p = board.cur_player;
    let evaluator = LinesEval::new();
    c.bench_function("LinesEval, depth=5", |b| b.iter(||{
        black_box(abpruning_action_values(&mut board, 4, &evaluator, p))
    }));
}

fn search_benchmark(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_move(action);
    }
    let p = board.cur_player;
    let evaluator = SimpleEval::new();
    c.bench_function("SimpleEval, depth=9", |b| b.iter(||{
        black_box(abpruning_action_values(&mut board, 9, &evaluator, p))
    }));
}

criterion_group!(benches, consecutive_eval_benchmark, lines_eval_benchmark, search_benchmark);
criterion_main!(benches);