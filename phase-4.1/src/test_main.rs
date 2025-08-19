// テスト用のmainファイル
use std::env;

mod play;
mod proto;  
mod database;
mod solver;
mod eval;

use crate::play::Board;
use crate::proto::Move;

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

pub fn initialize() {
    println!("Initializing database tables...");
    database::initialize_tables();
    println!("Initializing book...");
    database::init_book();
    println!("Initialization complete!");
}

pub fn get_ai_move(board_str: &str, turn: bool, assigned_time_ms: i32) -> usize {
    println!("Getting AI move for board: {}", board_str);
    println!("Turn: {}, Time: {}ms", turn, assigned_time_ms);
    
    let (black_board, white_board) = string_to_boards(board_str);
    let board = make_board(black_board, white_board, turn);
    
    println!("Board created successfully");
    println!("Valid moves: {:064b}", board.get_valid_moves());
    
    let result = board.decide_move(assigned_time_ms);
    println!("AI move result: {}", result);
    
    result
}

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

fn main() {
    println!("Starting test...");
    
    // 初期化をテスト
    println!("=== Testing initialize() ===");
    match std::panic::catch_unwind(|| {
        initialize();
    }) {
        Ok(_) => println!("✅ Initialize succeeded"),
        Err(e) => {
            println!("❌ Initialize failed: {:?}", e);
            return;
        }
    }
    
    // 標準開始局面でテスト
    let board_str = "...........................WB......BW...........................";
    
    println!("\n=== Testing get_valid_moves() ===");
    match std::panic::catch_unwind(|| {
        get_valid_moves(board_str, false)
    }) {
        Ok(valid_moves) => println!("✅ Valid moves: {}", valid_moves),
        Err(e) => {
            println!("❌ get_valid_moves failed: {:?}", e);
            return;
        }
    }
    
    println!("\n=== Testing get_ai_move() ===");
    match std::panic::catch_unwind(|| {
        get_ai_move(board_str, false, 100) // 短時間でテスト
    }) {
        Ok(ai_move) => println!("✅ AI move: {}", ai_move),
        Err(e) => {
            println!("❌ get_ai_move failed: {:?}", e);
            return;
        }
    }
    
    println!("\n=== Testing with longer time ===");
    match std::panic::catch_unwind(|| {
        get_ai_move(board_str, false, 1000)
    }) {
        Ok(ai_move) => println!("✅ AI move (1s): {}", ai_move),
        Err(e) => {
            println!("❌ get_ai_move (1s) failed: {:?}", e);
        }
    }
    
    println!("\nTest complete!");
}
