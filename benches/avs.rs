use criterion::{black_box, criterion_group, criterion_main, Criterion};

extern crate gamesolver;
use gamesolver::games::connect4::{self, Connect4};
use gamesolver::games::Game;
use gamesolver::games::stack4::Stack4;
use gamesolver::search::{*};
use gamesolver::evaluators::{ConsequtiveEval, LinesEval, SimpleEval, CNNEval, Evaluator};
use tch::nn::{ModuleT};



fn consecutive_eval_benchmark(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_action(action);
    }
    let p = board.cur_player;
    let evaluator = ConsequtiveEval::new();
    c.bench_function("ConsecutiveEval", |b| b.iter(||{
        black_box(evaluator.value(&board, p))
    }));
}

fn lines_eval_benchmark(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_action(action);
    }
    let p = board.cur_player;
    let evaluator = LinesEval::new();
    c.bench_function("LinesEval", |b| b.iter(||{
        black_box(evaluator.value(&board, p))
    }));
}

fn cnn_eval_benchmark(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_action(action);
    }
    let p = board.cur_player;
    let evaluator = CNNEval::new(String::from("models/bench_model.pt"));
    c.bench_function("CNNEval:value", |b| b.iter(||{
        black_box( evaluator.value(&board, p))
    }));
}

fn cnn_eval_forward(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_action(action);
    }
    let p = board.cur_player;
    let evaluator = CNNEval::new(String::from("models/bench_model.pt"));
    let vectorized_board = board.vectorize(p);
    let tensor = unsafe {
        let ptr = vectorized_board.as_ptr();
        let t = tch::Tensor::of_blob(
            ptr as *const u8, 
            &[1,1,connect4::BOARD_HEIGHT as i64, connect4::BOARD_WIDTH as i64], 
            &[0, 0, connect4::BOARD_WIDTH as i64, 1],
            tch::Kind::Double,
            tch::Device::Cpu,
        );
        t
    };
    c.bench_function("CNNEval:forward", |b| b.iter(||{
        black_box(evaluator.model.forward_t(&tensor, true))
    }));
}

fn cnn_eval_forward_no_grad(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_action(action);
    }
    let p = board.cur_player;
    let evaluator = CNNEval::new(String::from("models/bench_model.pt"));
    let vectorized_board = board.vectorize(p);
    let tensor = unsafe {
        let ptr = vectorized_board.as_ptr();
        let t = tch::Tensor::of_blob(
            ptr as *const u8, 
            &[1,1,connect4::BOARD_HEIGHT as i64, connect4::BOARD_WIDTH as i64], 
            &[0, 0, connect4::BOARD_WIDTH as i64, 1],
            tch::Kind::Double,
            tch::Device::Cpu,
        );
        t
    };
    c.bench_function("CNNEval:forward_nograd", |b| b.iter(||{
        let _guard = tch::no_grad_guard();
        black_box(evaluator.model.forward_t(&tensor, true))
    }));
}

fn cnn_eval_forward_100(c: &mut Criterion) {
    let n = 100;
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_action(action);
    }
    let p = board.cur_player;
    let evaluator = CNNEval::new(String::from("models/bench_model.pt"));
    let mut vectorized_boards = Vec::with_capacity(6*7*n);
    for i in 0..n {
        vectorized_boards.append(&mut board.vectorize(p));
    }
    let tensor = unsafe {
        let ptr = vectorized_boards.as_ptr();
        let t = tch::Tensor::of_blob(
            ptr as *const u8, 
            &[n as i64,1,connect4::BOARD_HEIGHT as i64, connect4::BOARD_WIDTH as i64], 
            &[connect4::BOARD_HEIGHT as i64*connect4::BOARD_WIDTH as i64, connect4::BOARD_HEIGHT as i64*connect4::BOARD_WIDTH as i64, connect4::BOARD_WIDTH as i64, 1],
            tch::Kind::Double,
            tch::Device::Cpu,
        );
        t
    };
    c.bench_function("CNNEval:forward_100", |b| b.iter(||{
        black_box(evaluator.model.forward_t(&tensor, true))
    }));
}

fn cnn_eval_forward_100_no_grad(c: &mut Criterion) {
    let n = 100;
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_action(action);
    }
    let p = board.cur_player;
    let evaluator = CNNEval::new(String::from("models/bench_model.pt"));
    let mut vectorized_boards = Vec::with_capacity(6*7*n);
    for i in 0..n {
        vectorized_boards.append(&mut board.vectorize(p));
    }
    let tensor = unsafe {
        let ptr = vectorized_boards.as_ptr();
        let t = tch::Tensor::of_blob(
            ptr as *const u8, 
            &[n as i64,1,connect4::BOARD_HEIGHT as i64, connect4::BOARD_WIDTH as i64], 
            &[connect4::BOARD_HEIGHT as i64*connect4::BOARD_WIDTH as i64, connect4::BOARD_HEIGHT as i64*connect4::BOARD_WIDTH as i64, connect4::BOARD_WIDTH as i64, 1],
            tch::Kind::Double,
            tch::Device::Cpu,
        );
        t
    };
    c.bench_function("CNNEval:forward_nograd_100", |b| b.iter(||{
        let _guard = tch::no_grad_guard();
        black_box(evaluator.model.forward_t(&tensor, true))
    }));
}

fn cnn_search_batch(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_action(action);
    }
    let p = board.cur_player;
    let evaluator = CNNEval::new(String::from("models/bench_model.pt"));
    c.bench_function("CNNEval:search_batch_depth=4", |b| b.iter(||{
        black_box(batch_abnegamax_best_action(&board, 4, 4, &evaluator, p));
    }));
}

fn cnn_search(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_action(action);
    }
    let p = board.cur_player;
    let evaluator = CNNEval::new(String::from("models/bench_model.pt"));
    c.bench_function("CNNEval:search_depth=5", |b| b.iter(||{
        black_box(abnegamax_best_action(&board, 5, &evaluator, p));
    }));
}


fn search_benchmark(c: &mut Criterion) {
    let mut board = Connect4::new();
    let actions = vec![4, 5, 3, 1, 3, 1, 1, 1, 4, 5, 5, 1, 4, 4, 2, 5];
    for action in actions {
        board.play_action(action);
    }
    let p = board.cur_player;
    let evaluator = SimpleEval::new();
    c.bench_function("SimpleEval, depth=9", |b| b.iter(||{
        black_box(abnegamax_best_action(&mut board, 9, &evaluator, p))
    }));
}

fn stack4search(c: &mut Criterion) {
    let mut board = Stack4::new();
    let evaluator = SimpleEval::new();
    let p = board.cur_player;
    c.bench_function("Stack4::SimpleEval, depth=4", |b| b.iter(|| {
        black_box(abnegamax_best_action(&mut board, 4, &evaluator, p))
    }));
}


criterion_group!(
    benches, 
    consecutive_eval_benchmark, 
    lines_eval_benchmark, 
    search_benchmark, 
    cnn_eval_benchmark,
    cnn_eval_forward_no_grad,
    cnn_eval_forward_100_no_grad,
    cnn_eval_forward,
    cnn_eval_forward_100,
    cnn_search,
    cnn_search_batch,
    stack4search
);
//criterion_group!(benches, cnn_eval_benchmark);
criterion_main!(benches);