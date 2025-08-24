use crate::play::Board;
use crate::database;

pub fn test_eval_wasm() -> f32 {
    // 初期盤面を作成
    let board = Board::new(false);
    
    // eval_wasmを直接テスト
    println!("Testing eval_wasm directly...");
    
    #[cfg(target_arch = "wasm32")]
    {
        println!("WebAssembly environment detected");
        let result = -crate::eval_wasm::EVAL_FUNCTION.eval(&board);
        println!("eval_wasm result: {}", result);
        result
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("Native environment - using eval");
        let result = -crate::eval::EVAL_FUNCTION.eval(&board);
        println!("eval result: {}", result);
        result
    }
}

pub fn test_decide_move_simple() -> usize {
    println!("Testing simple decide_move...");
    
    // データベースを初期化
    database::initialize_tables();
    database::init_book();
    
    // 初期盤面を作成
    let board = Board::new(false);
    
    // 短い時間でAI手を取得
    let result = board.decide_move(100, 0.0); // 100ms, disturbance=0.0
    println!("AI move result: {}", result);
    result
}
