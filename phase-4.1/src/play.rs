// ここいじる
use crate::database;
use crate::eval;
use super::proto::Move;
use crate::solver::solve;

use itertools::Itertools;
use std::fmt::{self, Display, Formatter};
use std::io::Write;  // selfを削除
use std::fs::OpenOptions;
use std::time::{Instant, Duration};
use chrono::Local;


const PASS : u8 = 64;
const NIL_MOVE : u8 = 255;
const DEPTH_INF : u8 = 64;

const WIN_SCORE : f32 = 1000.0;
const LOSE_SCORE : f32 = -1000.0;


pub struct InitGame {
    pub opponent_name: String,
    pub assigned_time_ms: i32,
}

pub struct TimeManager {
    start: Instant,
    limit: Duration
}

impl TimeManager {
    // 割り当て時間はフルに使うため、バッファは自分で持たせる
    pub fn new(limit_ms: i32) -> Self {
        Self {
            start: Instant::now(),
            limit: Duration::from_millis(limit_ms as u64)
        }
    }

    pub fn get_assigned_time(&self) -> i32 {
        self.limit.as_millis() as i32
    }

    pub fn should_stop(&self) -> bool {
        self.start.elapsed() >= self.limit
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Board{
    pub my_board : u64,
    pub opponent_board : u64
}

type Pos = (usize, usize);

impl Board {
    // ログ出力用のヘルパー関数
    fn write_to_log(&self, message: &str) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("log.txt") {
            let _ = writeln!(file, "{}", message);
        }
    }

    // 盤面をコンパクトにログ出力
    fn log_board_compact(&self, context: &str) {
        // 現在の時間を表示
        let now = Local::now();
        self.write_to_log(&format!("{} - {}", now.format("\n%Y-%m-%d %H:%M:%S"), context));
        
        // 石の数を表示
        let my_count = self.my_board.count_ones();
        let op_count = self.opponent_board.count_ones();
        self.write_to_log(&format!("O: {} pieces, X: {} pieces", my_count, op_count));
        
        // 配置可能位置を取得
        let placeable = self.get_valid_moves();
        
        // 座標軸（列）を表示
        self.write_to_log("  ABCDEFGH");
        
        // 盤面の出力
        for j in 0..8 {
            let mut line = format!("{} ", j + 1); // 行番号
            for i in 0..8 {
                let pos = j * 8 + i;
                let mark = if self.check_opponent(i, j) {
                    'X'
                } else if self.check_mine(i, j) {
                    'O'
                } else if placeable & (1 << pos) != 0 {
                    '✓'  // 配置可能位置
                } else {
                    '·'
                };
                line.push(mark);
            }
            self.write_to_log(&line);
        }
        
        // 配置可能位置があれば詳細を表示
        if placeable != 0 {
            let mut placeable_positions = Vec::new();
            for pos in 0..64 {
                if placeable & (1 << pos) != 0 {
                    let row = pos / 8;
                    let col = pos % 8;
                    let col_char = (b'A' + col as u8) as char;
                    placeable_positions.push(format!("{}{}", col_char, row + 1));
                }
            }
            self.write_to_log(&format!("Placeable: {}", placeable_positions.join(" ")));
        } else {
            self.write_to_log("No placeable positions");
        }
    }

    // turn = falseなら先手、trueなら後手
    pub fn new(turn : bool) -> Self {
        let board = if turn {
            Self {
                my_board: 0x0000001008000000,
                opponent_board: 0x0000000810000000,
            }
        } 
        else {
            Self {
                my_board: 0x0000000810000000,
                opponent_board: 0x0000001008000000,
            }
        };
        
        // 初期盤面をログに出力
        board.log_board_compact("Initial Board");
        board
    }

    pub fn check_mine(&self, x: usize, y: usize) -> bool {
        self.my_board & (1 << (y * 8 + x)) != 0
    }

    pub fn check_opponent(&self, x: usize, y: usize) -> bool {
        self.opponent_board & (1 << (y * 8 + x)) != 0
    }

    pub fn change_turn(&mut self) {
        std::mem::swap(&mut self.my_board, &mut self.opponent_board);
    }

    pub fn print_board(&self) {
        const ME: char = 'O';
        const OP: char = 'X';
        println!(" |A B C D E F G H ");
        println!("-+----------------");
        for j in 0..8 {
            print!("{}|", j + 1);
            for i in 0..8 {
                let mark = if self.check_opponent(i, j) {
                    OP
                }
                else if self.check_mine(i, j) {
                    ME
                }
                else {
                    ' '
                };
                print!("{} ", mark);
            }
            println!();
        }
        println!("  ({}: ME, {}: OP)", ME, OP);
    }

    // moveは1-indexed
    // インターフェース用のため、最適化はあまりされていない
    pub fn do_move_interface(&mut self, m: Move, is_my_turn: bool) {
        match m {
            Move::Mv { x_ah, y_18 } => {
                let pos = ((x_ah - 1) + (y_18 - 1) * 8) as usize;
                
                let flipper = if is_my_turn {
                    let f = database::get_flipper(self, pos);
                    self.my_board |= 1 << pos;
                    f
                }
                else {
                    self.change_turn();
                    let f = database::get_flipper(self, pos);
                    self.change_turn();
                    self.opponent_board |= 1 << pos;
                    f
                };

                self.my_board ^= flipper;
                self.opponent_board ^= flipper;

                // 手を打った後の盤面をログ
                let col_char = (b'A' + (x_ah - 1) as u8) as char;
                self.write_to_log(&format!("Move: {}{}", col_char, y_18));
                if !is_my_turn {
                    self.log_board_compact("Opponent's Move");
                }
            }
            Move::Pass => {
                self.write_to_log("Move: PASS");
            }
            Move::GiveUp => {
                self.write_to_log("Move: GIVEUP");
            }
        }
    }

    pub fn get_valid_moves(&self) -> u64 {
        return database::get_placeable(self);
    }

    fn get_turn(&self) -> usize {
        (self.my_board | self.opponent_board).count_ones() as usize
    }

    // 一手動かす(手番変更なし)
    pub fn do_move(&mut self, m: u8) {
        let flipper = database::get_flipper(self, m as usize);
        if flipper == 0 {
            self.print_board();
            panic!("Invalid move attempted: {}", m);
        }
        self.my_board |= 1 << m;
        self.my_board ^= flipper;
        self.opponent_board ^= flipper;
    }

    fn eval(&self) -> f32 {
        // eval呼び出しをカウント
        database::increment_eval_count();
        
        let my_piece_count = self.my_board.count_ones() as i32;
        let op_piece_count = self.opponent_board.count_ones() as i32;
        let turn = (my_piece_count + op_piece_count) as usize;
        
        if turn == 64 {
            let diff = my_piece_count - op_piece_count;
            return if my_piece_count > op_piece_count {
                WIN_SCORE + diff as f32
            } 
            else if my_piece_count < op_piece_count {
                LOSE_SCORE + diff as f32
            }
            else {
                0.0
            };
        }

        let my_placeable = self.get_valid_moves();
        let my_placeable_count = my_placeable.count_ones() as i32;
        let mut t = self.clone();
        t.change_turn();
        let op_placeable = t.get_valid_moves();
        let op_placeable_count = op_placeable.count_ones() as i32;
        
        if my_placeable == 0 && op_placeable == 0 {
            let diff = my_piece_count - op_piece_count;
            return if my_piece_count > op_piece_count {
                WIN_SCORE + diff as f32
            } 
            else if my_piece_count < op_piece_count {
                LOSE_SCORE + diff as f32
            }
            else {
                0.0
            };
        }

        // (my_piece_count - op_piece_count) as f32 * (1.0 - database::get_sigmoid_table()[turn]) + 
        // (my_placeable_count - op_placeable_count) as f32 * database::get_sigmoid_table()[turn]
        -eval::EVAL_FUNCTION.eval(&self)
    }
    
    pub fn decide_move(&self, assigned_time_ms: i32) -> usize {
        self.log_board_compact("AI Thinking");
        
        // カウンターをリセット
        database::reset_counters();
        
        let turn = self.get_turn();
        let mut time_manager = {
            if (assigned_time_ms >= 600000){
                if turn <= 46 {
                    TimeManager::new(2000);
                }
                else {
                    TimeManager::new(15000);
                }
            }

            if turn < 46 {
                TimeManager::new(assigned_time_ms * 2 / (50 - turn as i32))
            }
            else {
                TimeManager::new(assigned_time_ms * 2 / 3)
            }
        };
        self.write_to_log(&format!("Remaining time: {} ms, Allocated time: {} ms", assigned_time_ms, time_manager.get_assigned_time()));

        let moves = self.get_valid_moves();
        if moves == 0 {
            self.write_to_log("No valid moves - PASS");
            return PASS as usize;
        }

        if let Some(mv) = database::lookup_book(self) {
            self.write_to_log("Using book move");
            return mv as usize;
        }
        else {
            self.write_to_log("No book move found");
        }

        if turn >= 46 {
            let (res, mv) = solve(self, &time_manager);
            if (res != -2){
                if (res == -1) {
                    self.write_to_log("Solved: Losing position");
                }
                else if (res == 0) {
                    self.write_to_log(&format!("Solved: Draw position with move {}", mv));
                }
                else {
                    self.write_to_log(&format!("Solved: Winning position with move {}", mv));
                }
            }
            if res >= 0 {
                return mv as usize;
            }
            time_manager = TimeManager::new(assigned_time_ms / 12);
        }

        let alpha = f32::NEG_INFINITY;
        let beta = f32::INFINITY;
        let mut best_score = f32::NEG_INFINITY;
        let mut best_move = PASS as usize;
        let mut total_nodes = 0;
        let mut total_evals = 0;
        
        for depth in 1..61 {
            // 前の深度での値を記録
            let prev_nodes = database::get_node_count();
            let prev_evals = database::get_eval_count();
            
            let (score, finished, next_move, terminated) = nega_scout(&self, alpha, beta, depth, &time_manager);

            // この深度での新規訪問数
            let depth_nodes = database::get_node_count() - prev_nodes;
            let depth_evals = database::get_eval_count() - prev_evals;
            
            if !terminated {
                best_score = score;
                best_move = next_move as usize;
                total_nodes = database::get_node_count();
                total_evals = database::get_eval_count();
                
                // 統計情報をログ出力
                self.write_to_log(&format!(
                    "Depth {}: nodes={}, evals={}, score={:.2}, move={}", 
                    depth, depth_nodes, depth_evals, score, next_move
                ));
            }

            if finished || time_manager.should_stop() || depth == 60 {
                self.write_to_log(&format!(
                    "Search completed: Total nodes={}, Total evals={}, Ratio={:.2}%", 
                    total_nodes, total_evals, 
                    if total_nodes > 0 { (total_evals as f64 / total_nodes as f64) * 100.0 } else { 0.0 }
                ));
                
                self.write_to_log(&format!("Final: Depth={}, Best move={}, Score={:.2}, Finished={}", 
                    depth, best_move, best_score, finished));
                let now = Local::now();
                self.write_to_log(&format!("End : {}\n", now.format("%Y-%m-%d %H:%M:%S")));

                return best_move as usize;
            }
        }

        return PASS as usize;
    }
    
    // バックグラウンド思考専用のdecide_move（中断可能）
    pub fn decide_move_background(&self, stop_flag: &std::sync::Arc<std::sync::atomic::AtomicBool>) -> usize {
        //println!("Background thinking started...");
        
        // バックグラウンド思考専用のカウンターをリセット
        database::reset_background_counter();
        
        let moves = self.get_valid_moves();
        if moves == 0 {
            //println!("Background thinking: No valid moves - PASS");
            return PASS as usize;
        }

        // book moveをチェック
        if let Some(mv) = database::lookup_book(self) {
            //println!("Background thinking: Using book move {}", mv);
            return mv as usize;
        }

        let alpha = f32::NEG_INFINITY;
        let beta = f32::INFINITY;
        let mut best_move = PASS as usize;
        
        // 浅い深度から開始して、中断されるまで反復深化
        for depth in 1..=10 {  // バックグラウンドなので深度は制限
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                //println!("Background thinking interrupted at depth {}", depth);
                break;
            }
            
            let (score, _finished, next_move, terminated) = nega_scout_background(
                &self, alpha, beta, depth, stop_flag
            );

            if !terminated && !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                best_move = next_move as usize;
                let total_nodes = database::get_background_node_count();
                // println!("Background thinking: Depth {} completed, {} nodes, best move: {}", 
                //     depth, total_nodes, next_move);
            }

            if terminated || stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
        }

        let final_nodes = database::get_background_node_count();
        // println!("Background thinking completed: {} total nodes, best move: {}", 
        //     final_nodes, best_move);
        
        return best_move;
    }
}

fn nega_scout(board: &Board, original_alpha: f32, beta: f32, depth: u8, time_manager: &TimeManager) -> (f32, bool, u8, bool) {
    // ノード訪問をカウント
    database::increment_node_count();
    
    if time_manager.should_stop() {
        return (f32::NEG_INFINITY, false, PASS, true);
    }

    let mut alpha = original_alpha;
    match database::get_cache().get(&board){
        Some(node) => {
            if node.complete && node.value >= beta {
                return (node.value, node.complete, node.next_move, false);
            }

            if node.depth >= depth {
                if node.exact {
                    return (node.value, node.complete, node.next_move, false);
                }
                if node.lower_bound && node.value >= beta {
                    return (node.value, node.complete, node.next_move, false);
                }
                if node.upper_bound && node.value <= original_alpha {
                    return (node.value, node.complete, node.next_move, false);
                }

                if node.lower_bound {
                    alpha = alpha.max(node.value);
                }
            }
        }
        None => ()
    }

    // 終端条件
    if board.get_turn() == 64 {
        let eval = board.eval();  // ここでeval()が呼ばれる
        database::get_cache().set(board, DEPTH_INF, NIL_MOVE, eval, true, true, false, false);
        return (eval, true, NIL_MOVE, false);
    }

    if depth == 0 {
        let eval = board.eval();  // ここでeval()が呼ばれる
        let finished = eval >= WIN_SCORE || eval <= LOSE_SCORE;
        database::get_cache().set(board, 0, NIL_MOVE, eval, false, finished, false, false);
        return (eval, finished, NIL_MOVE, false);
    }

    let moves = board.get_valid_moves();
    if moves == 0 {
        let mut t = board.clone();
        t.change_turn();
        if t.get_valid_moves() == 0 {
            let eval = board.eval();  // ここでeval()が呼ばれる
            database::get_cache().set(&board, DEPTH_INF, NIL_MOVE, eval, true, true, false, false);
            return (eval, true, NIL_MOVE, false);
        }

        let p = nega_scout(&t, -beta, -alpha, depth - 1, &time_manager);
        let (eval, finished, _, terminated) = (-p.0, p.1, p.2, p.3);
        if terminated {return (eval, finished, PASS, true);}

        let (exact, lower_bound, upper_bound) = if eval <= original_alpha {
            (false, false, true)   // Upper bound
        } else if eval >= beta {
            (false, true, false)   // Lower bound
        } else {
            (true, false, false)   // Exact
        };
        
        database::get_cache().set(board, depth, PASS, eval, exact, finished, lower_bound, upper_bound);
        return (eval, finished, PASS, false);
    }

    // 手順生成と並び替え
    let mut ordered_moves: Vec<(f32, u8, Board)> = (0..64)
        .filter(|&m| moves & (1 << m) != 0)
        .map(|m| {
            let mut t = board.clone();
            t.do_move(m);
            t.change_turn();
            let eval = match database::get_cache().get(&t){
                Some(node) => node.value,
                None => 0.0 //t.eval()  // ここでeval()が呼ばれる
            };
            (eval, m, t)
        })
        .collect_vec();
    ordered_moves.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // 残りの探索は既存のまま...
    let p = nega_scout(&ordered_moves[0].2, -beta, -alpha, depth - 1, &time_manager);
    let (v, finished, _, terminated) = (-p.0, p.1, p.2, p.3);
    let mut best_score = v;
    let mut best_move = ordered_moves[0].1;
    let mut is_finished = finished;
    alpha = alpha.max(v);

    if terminated {return (best_score, is_finished, best_move, true);}

    if beta <= alpha {
        database::get_cache().set(board, depth, best_move, best_score, false, is_finished, true, false);
        return (best_score, is_finished, best_move, false);
    }

    for (_eval, m, t) in ordered_moves.iter().skip(1) {
        let p = nega_scout(t, -alpha - 1.0, -alpha, depth - 1, &time_manager);
        let (mut v, mut finished, _, terminated) = (-p.0, p.1, p.2, p.3);
        if terminated {return (best_score, is_finished, best_move, true);}

        if alpha < v && v < beta {
            let p = nega_scout(t, -beta, -v, depth - 1, &time_manager);
            (v, finished) = (-p.0, p.1);
            if p.3 {return (best_score, is_finished, best_move, true);}
        }
        
        if v > best_score {
            best_score = v;
            best_move = *m;
            is_finished = finished;
            alpha = v;
        }
        
        if beta <= alpha {
            database::get_cache().set(board, depth, best_move, best_score, false, is_finished, true, false);
            return (best_score, is_finished, best_move, false);
        }
    }
    
    let (exact, lower_bound, upper_bound) = if best_score <= original_alpha {
        (false, false, true)   // Upper bound
    } else if best_score >= beta {
        (false, true, false)   // Lower bound  
    } else {
        (true, false, false)   // Exact
    };

    database::get_cache().set(board, depth, best_move, best_score, exact, is_finished, lower_bound, upper_bound);
    (best_score, is_finished, best_move, false)
}

// バックグラウンド思考専用のnega_scout（1000ノードごとに進捗出力）
fn nega_scout_background(
    board: &Board, 
    original_alpha: f32, 
    beta: f32, 
    depth: u8, 
    stop_flag: &std::sync::Arc<std::sync::atomic::AtomicBool>
) -> (f32, bool, u8, bool) {
    // バックグラウンド思考専用のノード訪問カウント
    let current_nodes = database::increment_background_node_count();
    
    // 1000ノードごとに進捗を出力
    if current_nodes % 1000 == 0 {
        //println!("Background thinking: {} nodes evaluated", current_nodes);
    }
    
    // 中断フラグをチェック
    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
        return (f32::NEG_INFINITY, false, PASS, true);
    }

    let mut alpha = original_alpha;
    
    // 終端条件
    if board.get_turn() == 64 {
        let eval = board.eval();
        return (eval, true, NIL_MOVE, false);
    }

    if depth == 0 {
        let eval = board.eval();
        let finished = eval >= WIN_SCORE || eval <= LOSE_SCORE;
        return (eval, finished, NIL_MOVE, false);
    }

    let moves = board.get_valid_moves();
    if moves == 0 {
        let mut t = board.clone();
        t.change_turn();
        if t.get_valid_moves() == 0 {
            let eval = board.eval();
            return (eval, true, NIL_MOVE, false);
        }
        
        let (score, complete, _, terminated) = nega_scout_background(&t, -beta, -alpha, depth - 1, stop_flag);
        return (-score, complete, PASS, terminated);
    }

    let mut best_score = f32::NEG_INFINITY;
    let mut best_move = PASS;
    let mut is_finished = true;
    let first_move = moves.trailing_zeros() as u8;

    // 最初の手
    let mut t = board.clone();
    t.do_move(first_move);
    t.change_turn();
    let (score, complete, _, terminated) = nega_scout_background(&t, -beta, -alpha, depth - 1, stop_flag);
    
    if terminated {
        return (f32::NEG_INFINITY, false, PASS, true);
    }
    
    let score = -score;
    best_score = score;
    best_move = first_move;
    if !complete {
        is_finished = false;
    }
    
    alpha = alpha.max(score);
    if beta <= alpha {
        return (best_score, is_finished, best_move, false);
    }

    // 残りの手
    let remaining_moves = moves & !(1u64 << first_move);
    for i in 0..64 {
        if remaining_moves & (1u64 << i) == 0 {
            continue;
        }
        
        // 中断チェック
        if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            return (best_score, is_finished, best_move, true);
        }

        let mut t = board.clone();
        t.do_move(i);
        t.change_turn();
        
        // null window search
        let (mut score, complete, _, terminated) = nega_scout_background(&t, -alpha - 1.0, -alpha, depth - 1, stop_flag);
        
        if terminated {
            return (best_score, is_finished, best_move, true);
        }
        
        score = -score;
        
        if alpha < score && score < beta {
            // re-search with full window
            let (score2, complete2, _, terminated2) = nega_scout_background(&t, -beta, -score, depth - 1, stop_flag);
            
            if terminated2 {
                return (best_score, is_finished, best_move, true);
            }
            
            let score2 = -score2;
            if score2 > best_score {
                best_score = score2;
                best_move = i;
            }
            if !complete2 {
                is_finished = false;
            }
        } else if score > best_score {
            best_score = score;
            best_move = i;
            if !complete {
                is_finished = false;
            }
        }

        alpha = alpha.max(best_score);
        if beta <= alpha {
            return (best_score, is_finished, best_move, false);
        }
    }
    
    (best_score, is_finished, best_move, false)
}

impl Default for Board {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        const ME: char = 'O';
        const OP: char = 'X';
        writeln!(f, " |A B C D E F G H ")?;
        writeln!(f, "-+----------------")?;
        for j in 0..8 {
            write!(f, "{}|", j + 1)?;
            for i in 0..8 {
                let mark = if self.check_opponent(i, j) {
                    OP
                }
                else if self.check_mine(i, j) {
                    ME
                }
                else {
                    ' '
                };
                write!(f, "{} ", mark)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "  ({}: ME, {}: OP)", ME, OP)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_fmt() {
        let b = Board::new(false);
        println!("{}", b);
    }
}
