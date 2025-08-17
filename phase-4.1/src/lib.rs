use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

mod play;
mod proto;
mod database;
mod solver;

use crate::play::Board;
use crate::proto::Move;

// WebAssembly環境では専用の評価関数を使用
#[cfg(target_arch = "wasm32")]
mod eval_wasm;

// WebAssembly以外の環境では通常の評価関数
#[cfg(not(target_arch = "wasm32"))]
mod eval;

// 複雑な返り値用の構造体
#[derive(Serialize, Deserialize)]
pub struct GameResult {
    pub state: String,  // "BlackWin", "WhiteWin", "Draw", "Ongoing"
}

#[derive(Serialize, Deserialize)]
pub struct StoneCounts {
    pub black: u32,
    pub white: u32,
}

fn format_move(mv: usize) -> Move {
    if mv == 64 {
        Move::Pass
    } else {
        let x = (mv % 8) + 1;
        let y = (mv / 8) + 1;
        Move::Mv { x_ah: x as u32, y_18: y as u32 }
    }
}

fn make_board(black_board: u64, white_board: u64, turn: bool) -> Board {
    if turn {
        Board {
            my_board: white_board,
            opponent_board: black_board
        }
    }
    else {
        Board {
            my_board: black_board,
            opponent_board: white_board
        }
    }
}

// turn = trueの時、白の手番
#[wasm_bindgen]
pub fn update_board(black_board: u64, white_board: u64, mv: usize, turn: bool) -> JsValue {
    let mut board = make_board(black_board, white_board, !turn);

    let next_move = format_move(mv);
    board.do_move_interface(next_move, true);
    if !turn {
        board.change_turn();
    }
    JsValue::from_serde(&board).unwrap()
}

#[wasm_bindgen]
pub fn get_ai_move(black_board: u64, white_board: u64, turn: bool, assigned_time_ms: i32) -> usize {
    let board = make_board(black_board, white_board, turn);

    board.decide_move(assigned_time_ms)
}

#[wasm_bindgen]
pub fn is_pass(black_board: u64, white_board: u64, turn: bool) -> bool {
    let board = make_board(black_board, white_board, turn);
    board.get_valid_moves() == 0
}

#[wasm_bindgen]
pub fn get_result(black_board: u64, white_board: u64) -> JsValue {
    let board_0 = make_board(black_board, white_board, false);
    let board_1 = make_board(black_board, white_board, true);
    let state = {
        if board_0.get_valid_moves() == 0 && board_1.get_valid_moves() == 0 {
            let count_black = black_board.count_ones();
            let count_white = white_board.count_ones();

            if count_black > count_white {
                "BlackWin"
            }
            else if count_white > count_black {
                "WhiteWin"
            }
            else {
                "Draw"
            }
        }
        else {
            "Ongoing"
        }
    };

    JsValue::from_serde(&state).unwrap()
}

// 合法手のビットマスクを取得
#[wasm_bindgen]
pub fn get_valid_moves(black_board: u64, white_board: u64, turn: bool) -> u64 {
    let board = make_board(black_board, white_board, turn);
    board.get_valid_moves()
}

// 指定した手が合法かチェック
#[wasm_bindgen]
pub fn is_valid_move(black_board: u64, white_board: u64, mv: usize, turn: bool) -> bool {
    let board = make_board(black_board, white_board, turn);
    let valid_moves = board.get_valid_moves();
    (valid_moves & (1u64 << mv)) != 0
}

// 石の個数を取得（デバッグ用）
#[wasm_bindgen]
pub fn get_stone_counts(black_board: u64, white_board: u64) -> JsValue {
    let counts = serde_json::json!({
        "black": black_board.count_ones(),
        "white": white_board.count_ones()
    });
    JsValue::from_serde(&counts).unwrap()
}