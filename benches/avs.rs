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
    let evaluator = ConsequtiveEval::new(board.cur_player);
    c.bench_function("ConsecutiveEval, depth=5", |b| b.iter(||{
        black_box(abpruning_action_values(&mut board, 4, &evaluator))
    }));
}

fn lines_eval_benchmark(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_move(action);
    }
    let evaluator = LinesEval::new(board.cur_player);
    c.bench_function("LinesEval, depth=5", |b| b.iter(||{
        black_box(abpruning_action_values(&mut board, 4, &evaluator))
    }));
}

fn search_benchmark(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_move(action);
    }
    let evaluator = SimpleEval::new(board.cur_player);
    c.bench_function("LinesEval, depth=9", |b| b.iter(||{
        black_box(abpruning_action_values(&mut board, 9, &evaluator))
    }));
}

criterion_group!(benches, consecutive_eval_benchmark, lines_eval_benchmark, search_benchmark);
criterion_main!(benches);