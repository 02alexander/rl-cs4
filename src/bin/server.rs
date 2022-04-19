extern crate actix_web;
extern crate lazy_static;
extern crate gamesolver;
extern crate serde;
extern crate serde_json;
extern crate env_logger;
extern crate futures;

use num_traits::{FromPrimitive};
use lazy_static::lazy_static;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_web::middleware::Logger;
use actix_files::Files;

use gamesolver::evaluators::CNNEval;
use gamesolver::agents::{CompositeAgent, Agent};
use gamesolver::evaluators::Stack4Evaluators;
use gamesolver::qlearning::{QLearning};
use gamesolver::games::Player;
use gamesolver::games::stack4::Stack4;
use gamesolver::games::Game;
use serde::{Serialize, Deserialize};
use std::fs;

static AI_PATH: &str = "new_cons.json";

#[derive(Serialize, Deserialize)]
struct MoveRequest {
    board: Vec<u8>,
    player_to_move: u8,
}

#[derive(Serialize, Deserialize)]
struct Move {
    x: usize,
    y: usize,
    player: u8,
}

lazy_static! {
    static ref EVALUATOR: Stack4Evaluators = {
        let ai: QLearning<Stack4Evaluators> = serde_json::from_str(&fs::read_to_string(AI_PATH).unwrap()).unwrap();
        ai.evaluator
    };
}

fn calc_move(board: &Stack4, player: Player) -> <Stack4 as Game>::Action {
    //let agent = MinimaxAgent::<Stack4Evaluators>::new(&EVALUATOR, 5);
    let agent = CompositeAgent::<Stack4Evaluators>::new(&EVALUATOR, 4, 0, 6);
    agent.get_action(board, player)
}

#[get("/{name}/index.html")]
async fn index(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    format!("Hello {}!", name)
}

async fn request_move(info: web::Json<MoveRequest>) -> web::Json<Move> {
    println!("post!");

    let mut board = Stack4::new();
    for (i, &player) in info.board.iter().enumerate() {
        let row = i%8;
        let col = i/8;
        if player == 0 {
            board.set(col, row, player);
        } else {
            board.set(col, row, player);
        }
    }
    let player = FromPrimitive::from_u8(info.player_to_move).expect("an u8");
    let (x, y) = calc_move(&board, player);
    println!("{:?}", board);
    println!("{:?}", (x,y));
    web::Json(
        Move {
            x,
            y,
            player: info.player_to_move
        }
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| App::new()
        .wrap(Logger::default())
        .route("/", web::post().to(request_move))
        .service(index)
        .service(Files::new("/", "static/").prefer_utf8(true))
    )
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
