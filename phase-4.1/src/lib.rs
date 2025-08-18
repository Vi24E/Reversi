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

// 文字列から盤面を作成
fn string_to_boards(board_str: &str) -> (u64, u64) {
    let mut black_board = 0u64;
    let mut white_board = 0u64;
    
    for (i, c) in board_str.chars().enumerate() {
        if i >= 64 { break; }
        
        match c {
            'B' => black_board |= 1u64 << i,
            'W' => white_board |= 1u64 << i,
            _ => {} // '.' or other characters are treated as empty
        }
    }
    
    (black_board, white_board)
}

// 盤面を文字列に変換
fn boards_to_string(black_board: u64, white_board: u64) -> String {
    let mut result = String::with_capacity(64);
    
    for i in 0..64 {
        let mask = 1u64 << i;
        if black_board & mask != 0 {
            result.push('B');
        } else if white_board & mask != 0 {
            result.push('W');
        } else {
            result.push('.');
        }
    }
    
    result
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
pub fn update_board(board_str: &str, mv: usize, turn: bool) -> String {
    let (black_board, white_board) = string_to_boards(board_str);
    let mut board = make_board(black_board, white_board, !turn);

    let next_move = format_move(mv);
    board.do_move_interface(next_move, true);
    if !turn {
        board.change_turn();
    }
    
    boards_to_string(board.my_board, board.opponent_board)
}

#[wasm_bindgen]
pub fn get_ai_move(board_str: &str, turn: bool, assigned_time_ms: i32) -> usize {
    let (black_board, white_board) = string_to_boards(board_str);
    let board = make_board(black_board, white_board, turn);

    board.decide_move(assigned_time_ms)
}

#[wasm_bindgen]
pub fn is_pass(board_str: &str, turn: bool) -> bool {
    let (black_board, white_board) = string_to_boards(board_str);
    let board = make_board(black_board, white_board, turn);
    board.get_valid_moves() == 0
}

#[wasm_bindgen]
pub fn get_result(board_str: &str) -> String {
    let (black_board, white_board) = string_to_boards(board_str);
    let board_0 = make_board(black_board, white_board, false);
    let board_1 = make_board(black_board, white_board, true);
    
    if board_0.get_valid_moves() == 0 && board_1.get_valid_moves() == 0 {
        let count_black = black_board.count_ones();
        let count_white = white_board.count_ones();

        if count_black > count_white {
            "BlackWin".to_string()
        } else if count_white > count_black {
            "WhiteWin".to_string()
        } else {
            "Draw".to_string()
        }
    } else {
        "Ongoing".to_string()
    }
}

// 合法手のビットマスクを文字列で取得
#[wasm_bindgen]
pub fn get_valid_moves(board_str: &str, turn: bool) -> String {
    let (black_board, white_board) = string_to_boards(board_str);
    let board = make_board(black_board, white_board, turn);
    let valid_moves = board.get_valid_moves();
    
    // ビットマスクを文字列で表現（64文字、'1'は有効手、'0'は無効手）
    let mut result = String::with_capacity(64);
    for i in 0..64 {
        if valid_moves & (1u64 << i) != 0 {
            result.push('1');
        } else {
            result.push('0');
        }
    }
    result
}

// 指定した手が合法かチェック
#[wasm_bindgen]
pub fn is_valid_move(board_str: &str, mv: usize, turn: bool) -> bool {
    let (black_board, white_board) = string_to_boards(board_str);
    let board = make_board(black_board, white_board, turn);
    let valid_moves = board.get_valid_moves();
    (valid_moves & (1u64 << mv)) != 0
}

// 石の個数を取得（デバッグ用）
#[wasm_bindgen]
pub fn get_stone_counts(board_str: &str) -> JsValue {
    let (black_board, white_board) = string_to_boards(board_str);
    let counts = serde_json::json!({
        "black": black_board.count_ones(),
        "white": white_board.count_ones()
    });
    JsValue::from_serde(&counts).unwrap()
}