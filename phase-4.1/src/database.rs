/*
Boardは全てxが縦、yが横で管理し、(0, 0)が左上である
また、一部8 * x + yでインデックスを管理している
*/

use crate::play;

use std::sync::LazyLock;
use std::sync::Once;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fs::File;
use std::io::Read;

// シグモイド関数の前計算（実行時初期化に変更）
static mut SIGMOID_TABLE: [f32; 65] = [0.0; 65];

// parameters
const BOARD_SCORE_A1 : i32 = 1;
const BOARD_SCORE_A2 : i32 = 1;
const BOARD_SCORE_A3 : i32 = 1;
const BOARD_SCORE_A4 : i32 = 1;
const BOARD_SCORE_B2 : i32 = 1;
const BOARD_SCORE_B3 : i32 = 1;
const BOARD_SCORE_B4 : i32 = 1;
const BOARD_SCORE_C3 : i32 = 1;
const BOARD_SCORE_C4 : i32 = 1;
const BOARD_SCORE_D4 : i32 = 1;

const K_1 : f32 = 0.6;
const X_0 : f32 = 54.0;

// 盤面点
static BOARD_SCORE : [i32; 64] = {
    let mut score = [0; 64];
    let mut i = 0;
    while i < 8 {
        let mut j = 0;
        while j < 8 {
            let ii = if i < 4 { i } else { 7 - i };
            let jj = if j < 4 { j } else { 7 - j };
            let mn = if ii < jj { ii } else { jj };
            let mx = if ii > jj { ii } else { jj };

            match (mn, mx) {
                (0, 0) => score[i * 8 + j] = BOARD_SCORE_A1,
                (0, 1) => score[i * 8 + j] = BOARD_SCORE_A2,
                (0, 2) => score[i * 8 + j] = BOARD_SCORE_A3,
                (0, 3) => score[i * 8 + j] = BOARD_SCORE_A4,
                (1, 1) => score[i * 8 + j] = BOARD_SCORE_B2,
                (1, 2) => score[i * 8 + j] = BOARD_SCORE_B3,
                (1, 3) => score[i * 8 + j] = BOARD_SCORE_B4,
                (2, 2) => score[i * 8 + j] = BOARD_SCORE_C3,
                (2, 3) => score[i * 8 + j] = BOARD_SCORE_C4,
                (3, 3) => score[i * 8 + j] = BOARD_SCORE_D4,
                _ => ()
            }
            j += 1;
        }
        i += 1;
    }
    score
};

const ROW_MASK : [u64; 8] = [
    0x00000000000000FF,
    0x000000000000FF00,
    0x0000000000FF0000,
    0x00000000FF000000,
    0x000000FF00000000,
    0x0000FF0000000000,
    0x00FF000000000000,
    0xFF00000000000000
];

// get row mask from x \in [0, 64);
fn get_row_mask(x: usize) -> u64 {
    ROW_MASK[x >> 3]
}

const COL_MASK : [u64; 8] = [
    0x0101010101010101,
    0x0202020202020202,
    0x0404040404040404,
    0x0808080808080808,
    0x1010101010101010,
    0x2020202020202020,
    0x4040404040404040,
    0x8080808080808080
];

fn get_col_mask(x: usize) -> u64 {
    COL_MASK[x & 7]
}

const DIAG_MASK_1 : [u64; 15] = [
    0x0000000000000001,
    0x0000000000000102,
    0x0000000000010204,
    0x0000000001020408,
    0x0000000102040810,
    0x0000010204081020,
    0x0001020408102040,
    0x0102040810204080,
    0x0204081020408000,
    0x0408102040800000,
    0x0810204080000000,
    0x1020408000000000,
    0x2040800000000000,
    0x4080000000000000,
    0x8000000000000000
];

fn get_diag_mask1(x: usize) -> u64 {
    DIAG_MASK_1[(x & 7) + (x >> 3)]
}

const DIAG_MASK_2 : [u64; 15] = [
    0x0000000000000080,
    0x0000000000008040,
    0x0000000000804020,
    0x0000000080402010,
    0x0000008040201008,
    0x0000804020100804,
    0x0080402010080402,
    0x8040201008040201,
    0x4020100804020100,
    0x2010080402010000,
    0x1008040201000000,
    0x0804020100000000,
    0x0402010000000000,
    0x0201000000000000,
    0x0100000000000000
];

fn get_diag_mask2(x: usize) -> u64 {
    DIAG_MASK_2[(!x & 7) + (x >> 3)]
}

// (masked_data << 3) | row_indexで管理
// 行ごとにスコアをあらかじめメモ化
static SCORE_TABLE : [i32; 2048] = {
    let mut score = [0; 2048];
    let mut i = 0;

    while i < 8 {
        let mut j = 0;
        while j < 256 {
            let mut k = 0;
            while k < 8 {
                if j & (1 << k) != 0 {
                    let idx = (j << 3) | i;
                    score[idx] += BOARD_SCORE[i * 8 + k];
                }
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }
    score
};

// 実行時初期化用のグローバル変数とOnce
static mut ROW_PLACEABLE: [u64; 65536] = [0; 65536];
static mut COL_PLACEABLE: [u64; 65536] = [0; 65536];
static mut DIAG_PLACEABLE_1: [u64; 65536] = [0; 65536];
static mut DIAG_PLACEABLE_2: [u64; 65536] = [0; 65536];

// turntableはflipするべき場所のbitを持ち、
// それぞれ(my_board & mask) << 11 | (opponent_board & mask) << 3 | placeで管理
static mut ROW_TURNTABLE: [u64; 524288] = [0; 524288];
static mut COL_TURNTABLE: [u64; 524288] = [0; 524288];
static mut DIAG_TURNTABLE_1: [u64; 524288] = [0; 524288];
static mut DIAG_TURNTABLE_2: [u64; 524288] = [0; 524288];
static INIT: Once = Once::new();

// 初期化関数
pub fn initialize_tables() {
    INIT.call_once(|| {
        unsafe {
            // SIGMOID_TABLE の初期化
            for i in 0..=64 {
                SIGMOID_TABLE[i] = 1.0 / (1.0 + (K_1 * (i as f32 - X_0)).exp());
            }

            // ROW_PLACEABLE の初期化
            for i in 0..256 {
                for j in 0..256 {
                    if i & j != 0 {
                        continue;
                    }
                    let mut data = 0u64;
                    
                    // 左から右へのスキャン
                    let mut placeable_1 = false;
                    let mut placeable_2 = false;
                    for k in 0..8 {
                        if !placeable_1 && (i & (1 << k) != 0) {
                            placeable_1 = true;
                        } else if placeable_1 && (j & (1 << k) != 0) {
                            placeable_2 = true;
                        } else if placeable_1 && placeable_2 && (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                            data |= 1 << k;
                        }
                        if (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                            placeable_1 = false;
                            placeable_2 = false;
                        }
                        if i & (1 << k) != 0 {
                            placeable_2 = false;
                        }
                    }
                    
                    // 右から左へのスキャン
                    placeable_1 = false;
                    placeable_2 = false;
                    for k in (0..8).rev() {
                        if !placeable_1 && (i & (1 << k) != 0) {
                            placeable_1 = true;
                        } else if placeable_1 && (j & (1 << k) != 0) {
                            placeable_2 = true;
                        } else if placeable_1 && placeable_2 && (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                            data |= 1 << k;
                        }
                        if (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                            placeable_1 = false;
                            placeable_2 = false;
                        }
                        if i & (1 << k) != 0 {
                            placeable_2 = false;
                        }
                    }
                    
                    ROW_PLACEABLE[(i << 8) | j] = data;
                }
            }

            // COL_PLACEABLE の初期化
            for i in 0..256 {
                for j in 0..256 {
                    if i & j != 0 {
                        continue;
                    }
                    let mut data = 0u64;
                    
                    // 上から下へのスキャン
                    let mut placeable_1 = false;
                    let mut placeable_2 = false;
                    for k in 0..8 {
                        if !placeable_1 && (i & (1 << k) != 0) {
                            placeable_1 = true;
                        } else if placeable_1 && (j & (1 << k) != 0) {
                            placeable_2 = true;
                        } else if placeable_1 && placeable_2 && (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                            data |= 1 << (k * 8);
                        }
                        if (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                            placeable_1 = false;
                            placeable_2 = false;
                        }
                        if i & (1 << k) != 0 {
                            placeable_2 = false;
                        }
                    }
                    
                    // 下から上へのスキャン
                    placeable_1 = false;
                    placeable_2 = false;
                    for k in (0..8).rev() {
                        if !placeable_1 && (i & (1 << k) != 0) {
                            placeable_1 = true;
                        } else if placeable_1 && (j & (1 << k) != 0) {
                            placeable_2 = true;
                        } else if placeable_1 && placeable_2 && (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                            data |= 1 << (k * 8);
                        }
                        if (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                            placeable_1 = false;
                            placeable_2 = false;
                        }
                        if i & (1 << k) != 0 {
                            placeable_2 = false;
                        }
                    }
                    
                    COL_PLACEABLE[(i << 8) | j] = data;
                }
            }

            // DIAG_PLACEABLE_1 の初期化
            for i in 0..256 {
                for j in 0..256 {
                    if i & j != 0 {
                        continue;
                    }
                    let mut data = 0u64;
                    
                    // 双方向スキャン
                    for direction in [false, true] {
                        let mut placeable_1 = false;
                        let mut placeable_2 = false;
                        let range: Box<dyn Iterator<Item = usize>> = if direction {
                            Box::new(0..8)
                        } else {
                            Box::new((0..8).rev())
                        };
                        
                        for k in range {
                            if !placeable_1 && (i & (1 << k) != 0) {
                                placeable_1 = true;
                            } else if placeable_1 && (j & (1 << k) != 0) {
                                placeable_2 = true;
                            } else if placeable_1 && placeable_2 && (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                                data |= 1 << (k * 7);
                            }
                            if (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                                placeable_1 = false;
                                placeable_2 = false;
                            }
                            if i & (1 << k) != 0 {
                                placeable_2 = false;
                            }
                        }
                    }
                    
                    DIAG_PLACEABLE_1[(i << 8) | j] = data;
                }
            }

            // DIAG_PLACEABLE_2 の初期化
            for i in 0..256 {
                for j in 0..256 {
                    if i & j != 0 {
                        continue;
                    }
                    let mut data = 0u64;
                    
                    // 双方向スキャン
                    for direction in [false, true] {
                        let mut placeable_1 = false;
                        let mut placeable_2 = false;
                        let range: Box<dyn Iterator<Item = usize>> = if direction {
                            Box::new(0..8)
                        } else {
                            Box::new((0..8).rev())
                        };
                        
                        for k in range {
                            if !placeable_1 && (i & (1 << k) != 0) {
                                placeable_1 = true;
                            } else if placeable_1 && (j & (1 << k) != 0) {
                                placeable_2 = true;
                            } else if placeable_1 && placeable_2 && (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                                data |= 1 << (k * 9);
                            }
                            if (i & (1 << k) == 0) && (j & (1 << k) == 0) {
                                placeable_1 = false;
                                placeable_2 = false;
                            }
                            if i & (1 << k) != 0 {
                                placeable_2 = false;
                            }
                        }
                    }
                    
                    DIAG_PLACEABLE_2[(i << 8) | j] = data;
                }
            }
        
            // ROW_TURNTABLE の初期化
            for i in 0..256 {
                for j in 0..256 {
                    if i & j != 0 {
                        continue;
                    }
                    
                    for k in 0..8 {
                        let mut data = 0u64;
                        if ROW_PLACEABLE[(i << 8) | j] & (1 << k) == 0 {
                            continue;
                        }

                        let mut l = k - 1 as isize;
                        while l >= 0 && (j & (1 << l) != 0){
                            l -= 1;
                        }
                        if l != -1 && i & (1 << l) != 0 {
                            l += 1;
                            while l != k {
                                data |= 1 << l;
                                l += 1;
                            }
                        }

                        l = k + 1;
                        while l < 8 && (j & (1 << l) != 0){
                            l += 1;
                        }
                        if l != 8 && i & (1 << l) != 0 {
                            l -= 1;
                            while l != k {
                                data |= 1 << l;
                                l -= 1;
                            }
                        }

                        ROW_TURNTABLE[(i << 11) as usize | (j << 3) as usize | k as usize] = data;
                    }
                }
            }

            // COL_TURNTABLE の初期化
            for i in 0..256 {
                for j in 0..256 {
                    if i & j != 0 {
                        continue;
                    }
                    
                    for k in 0..8 {
                        let mut data = 0u64;
                        if ROW_PLACEABLE[(i << 8) | j] & (1 << k) == 0 {
                            continue;
                        }

                        let mut l = k - 1 as isize;
                        while l >= 0 && (j & (1 << l) != 0){
                            l -= 1;
                        }
                        if l != -1 && i & (1 << l) != 0 {
                            l += 1;
                            while l != k {
                                data |= 1 << (l * 8);
                                l += 1;
                            }
                        }

                        l = k + 1;
                        while l < 8 && (j & (1 << l) != 0){
                            l += 1;
                        }
                        if l != 8 && i & (1 << l) != 0 {
                            l -= 1;
                            while l != k {
                                data |= 1 << (l * 8);
                                l -= 1;
                            }
                        }

                        COL_TURNTABLE[(i << 11) as usize | (j << 3) as usize | k as usize] = data;
                    }
                }
            }

            // DIAG_TURNTABLE_1 の初期化
            for i in 0..256 {
                for j in 0..256 {
                    if i & j != 0 {
                        continue;
                    }
                    
                    for k in 0..8 {
                        let mut data = 0u64;
                        if ROW_PLACEABLE[(i << 8) | j] & (1 << k) == 0 {
                            continue;
                        }

                        let mut l = k - 1 as isize;
                        while l >= 0 && (j & (1 << l) != 0){
                            l -= 1;
                        }
                        if l != -1 && i & (1 << l) != 0 {
                            l += 1;
                            while l != k {
                                data |= 1 << (l * 7);
                                l += 1;
                            }
                        }

                        l = k + 1;
                        while l < 8 && (j & (1 << l) != 0){
                            l += 1;
                        }
                        if l != 8 && i & (1 << l) != 0 {
                            l -= 1;
                            while l != k {
                                data |= 1 << (l * 7);
                                l -= 1;
                            }
                        }

                        DIAG_TURNTABLE_1[(i << 11) as usize | (j << 3) as usize | k as usize] = data;
                    }
                }
            }

            // DIAG_TURNTABLE_2 の初期化
            for i in 0..256 {
                for j in 0..256 {
                    if i & j != 0 {
                        continue;
                    }
                    
                    for k in 0..8 {
                        let mut data = 0u64;
                        if ROW_PLACEABLE[(i << 8) | j] & (1 << k) == 0 {
                            continue;
                        }

                        let mut l = k - 1 as isize;
                        while l >= 0 && (j & (1 << l) != 0){
                            l -= 1;
                        }
                        if l != -1 && i & (1 << l) != 0 {
                            l += 1;
                            while l != k {
                                data |= 1 << (l * 9);
                                l += 1;
                            }
                        }

                        l = k + 1;
                        while l < 8 && (j & (1 << l) != 0){
                            l += 1;
                        }
                        if l != 8 && i & (1 << l) != 0 {
                            l -= 1;
                            while l != k {
                                data |= 1 << (l * 9);
                                l -= 1;
                            }
                        }

                        DIAG_TURNTABLE_2[(i << 11) as usize | (j << 3) as usize | k as usize] = data;
                    }
                }
            }



        }
    });
}

// シグモイドテーブルにアクセスする関数
pub fn get_sigmoid_table() -> &'static [f32; 65] {
    initialize_tables();
    unsafe { &SIGMOID_TABLE }
}

// テーブルにアクセスする関数
fn get_row_placeable_table() -> &'static [u64; 65536] {
    initialize_tables();
    unsafe { &ROW_PLACEABLE }
}

fn get_col_placeable_table() -> &'static [u64; 65536] {
    initialize_tables();
    unsafe { &COL_PLACEABLE }
}

fn get_diag_placeable_1_table() -> &'static [u64; 65536] {
    initialize_tables();
    unsafe { &DIAG_PLACEABLE_1 }
}

fn get_diag_placeable_2_table() -> &'static [u64; 65536] {
    initialize_tables();
    unsafe { &DIAG_PLACEABLE_2 }
}

fn get_row_turntable() -> &'static [u64; 524288] {
    initialize_tables();
    unsafe { &ROW_TURNTABLE }
}

fn get_col_turntable() -> &'static [u64; 524288] {
    initialize_tables();
    unsafe { &COL_TURNTABLE }
}

fn get_diag_turntable_1() -> &'static [u64; 524288] {
    initialize_tables();
    unsafe { &DIAG_TURNTABLE_1 }
}

fn get_diag_turntable_2() -> &'static [u64; 524288] {
    initialize_tables();
    unsafe { &DIAG_TURNTABLE_2 }
}

// x行目のplaceableを返す
fn get_row_placeable(board: &play::Board, row_idx: usize) -> u64{
    let my_t = (board.my_board >> (row_idx * 8)) & 0xFF;
    let op_t = (board.opponent_board >> (row_idx * 8)) & 0xFF;
    return get_row_placeable_table()[((my_t << 8) | op_t) as usize] << (row_idx * 8);
}

fn get_col_placeable(board: &play::Board, col_idx: usize) -> u64{
	let my_t = (board.my_board & COL_MASK[col_idx]) >> col_idx;
	let op_t = (board.opponent_board & COL_MASK[col_idx]) >> col_idx;

	const MASK1 : u64 = 0x0100010001000100;
	const MASK2 : u64 = 0x0001000100010001;
	const MASK3 : u64 = 0x0003000000030000;
	const MASK4 : u64 = 0x0000000300000003;
	const MASK5 : u64 = 0x0000000f00000000;
	const MASK6 : u64 = 0x000000000000000f;
	fn shrink_bit_8(data: u64) -> u64 {
		let t1 = ((data & MASK1) >> 7) | (data & MASK2);
		let t2 = ((t1 & MASK3) >> 14) | (t1 & MASK4);
		let t3 = ((t2 & MASK5) >> 28) | (t2 & MASK6);
		return t3;
	}

    return (get_col_placeable_table()[((shrink_bit_8(my_t) << 8) | shrink_bit_8(op_t)) as usize] << col_idx) & COL_MASK[col_idx];
}

// x番目の斜のplaceableを返す
// x \in [0, 14)で、
// 0: 0
// 1: 1, 8
// 2: 2, 9, 16 ...
// のようにindexを(x, y)とした時、x + yで管理
fn get_diag1_placeable(board: &play::Board, diag1_idx: usize) -> u64{
	let my_t = (board.my_board & DIAG_MASK_1[diag1_idx]) >> diag1_idx;
	let op_t = (board.opponent_board & DIAG_MASK_1[diag1_idx]) >> diag1_idx;

	const MASK1 : u64 = 0b10000000000000100000000000001000000000000010000000;
	const MASK2 : u64 = 0b00000001000000000000010000000000000100000000000001;
	const MASK3 : u64 = 0b00000011000000000000000000000000001100000000000000;
	const MASK4 : u64 = 0b00000000000000000000110000000000000000000000000011;
	const MASK5 : u64 = 0b00000000000000000011110000000000000000000000000000;
	const MASK6 : u64 = 0b00000000000000000000000000000000000000000000001111;
	
	fn shrink_bit_7(data: u64) -> u64 {
		let t1 = ((data & MASK1) >> 6) | (data & MASK2);
		let t2 = ((t1 & MASK3) >> 12) | (t1 & MASK4);
		let t3 = ((t2 & MASK5) >> 24) | (t2 & MASK6);
		return t3;
	}

    return (get_diag_placeable_1_table()[((shrink_bit_7(my_t) << 8) | shrink_bit_7(op_t)) as usize] << diag1_idx) & DIAG_MASK_1[diag1_idx];
}

// x番目の斜のplaceableを返す
// x \in [0, 14)で、
// 0: 7
// 1: 6, 15
// 2: 5, 14, 23
// のようにindexを(x, y)とした時、7 + x - yで管理
fn get_diag2_placeable(board: &play::Board, diag2_idx: usize) -> u64{
	let my_t = if diag2_idx < 8 
        {(board.my_board & DIAG_MASK_2[diag2_idx]) >> (7 - diag2_idx)}
    else
        {(board.my_board & DIAG_MASK_2[diag2_idx]) << (diag2_idx - 7)};
	let op_t = if diag2_idx < 8 
        {(board.opponent_board & DIAG_MASK_2[diag2_idx]) >> (7 - diag2_idx)}
    else
        {(board.opponent_board & DIAG_MASK_2[diag2_idx]) << (diag2_idx - 7)};

	const MASK1 : u64 = 0b1000000000000000001000000000000000001000000000000000001000000000;
	const MASK2 : u64 = 0b0000000001000000000000000001000000000000000001000000000000000001;
	const MASK3 : u64 = 0b0000000011000000000000000000000000000000000011000000000000000000;
	const MASK4 : u64 = 0b0000000000000000000000000011000000000000000000000000000000000011;
	const MASK5 : u64 = 0b0000000000000000000000001111000000000000000000000000000000000000;
	const MASK6 : u64 = 0b0000000000000000000000000000000000000000000000000000000000001111;
	
	fn shrink_bit_9(data: u64) -> u64 {
		let t1 = ((data & MASK1) >> 8) | (data & MASK2);
		let t2 = ((t1 & MASK3) >> 16) | (t1 & MASK4);
		let t3 = ((t2 & MASK5) >> 32) | (t2 & MASK6);
		return t3;
	}

    let t = get_diag_placeable_2_table()[((shrink_bit_9(my_t) << 8) | shrink_bit_9(op_t)) as usize];
    return (if diag2_idx < 8 {
        t << (7 - diag2_idx)
    } else {
        t >> (diag2_idx - 7)
    }) & DIAG_MASK_2[diag2_idx];
}

// 全てのplaceableを取得
pub fn get_placeable(board: &play::Board) -> u64 {
    let mut placeable = 0u64;
    placeable |= get_row_placeable(board, 0);
    placeable |= get_row_placeable(board, 1);
    placeable |= get_row_placeable(board, 2);
    placeable |= get_row_placeable(board, 3);
    placeable |= get_row_placeable(board, 4);
    placeable |= get_row_placeable(board, 5);
    placeable |= get_row_placeable(board, 6);
    placeable |= get_row_placeable(board, 7);

    placeable |= get_col_placeable(board, 0);
    placeable |= get_col_placeable(board, 1);
    placeable |= get_col_placeable(board, 2);
    placeable |= get_col_placeable(board, 3);
    placeable |= get_col_placeable(board, 4);
    placeable |= get_col_placeable(board, 5);
    placeable |= get_col_placeable(board, 6);
    placeable |= get_col_placeable(board, 7);

    placeable |= get_diag1_placeable(board, 2);
    placeable |= get_diag1_placeable(board, 3);
    placeable |= get_diag1_placeable(board, 4);
    placeable |= get_diag1_placeable(board, 5);
    placeable |= get_diag1_placeable(board, 6);
    placeable |= get_diag1_placeable(board, 7);
    placeable |= get_diag1_placeable(board, 8);
    placeable |= get_diag1_placeable(board, 9);
    placeable |= get_diag1_placeable(board, 10);
    placeable |= get_diag1_placeable(board, 11);
    placeable |= get_diag1_placeable(board, 12);

    placeable |= get_diag2_placeable(board, 2);
    placeable |= get_diag2_placeable(board, 3);
    placeable |= get_diag2_placeable(board, 4);
    placeable |= get_diag2_placeable(board, 5);
    placeable |= get_diag2_placeable(board, 6);
    placeable |= get_diag2_placeable(board, 7);
    placeable |= get_diag2_placeable(board, 8);
    placeable |= get_diag2_placeable(board, 9);
    placeable |= get_diag2_placeable(board, 10);
    placeable |= get_diag2_placeable(board, 11);
    placeable |= get_diag2_placeable(board, 12);
    return placeable;
}

// (x, y)のrow flipを返す
fn get_row_flip(board: &play::Board, x: usize, y: usize) -> u64{
    let my_t = (board.my_board >> (x << 3)) & 0xFF;
    let op_t = (board.opponent_board >> (x << 3)) & 0xFF;
    return get_row_turntable()[(my_t << 11) as usize | (op_t << 3) as usize | y] << (x << 3);
}

fn get_col_flip(board: &play::Board, x: usize, y: usize) -> u64{
	let my_t = (board.my_board & COL_MASK[y]) >> y;
	let op_t = (board.opponent_board & COL_MASK[y]) >> y;

	const MASK1 : u64 = 0x0100010001000100;
	const MASK2 : u64 = 0x0001000100010001;
	const MASK3 : u64 = 0x0003000000030000;
	const MASK4 : u64 = 0x0000000300000003;
	const MASK5 : u64 = 0x0000000f00000000;
	const MASK6 : u64 = 0x000000000000000f;
    
    #[inline(always)]
	fn shrink_bit_8(data: u64) -> u64 {
		let t1 = ((data & MASK1) >> 7) | (data & MASK2);
		let t2 = ((t1 & MASK3) >> 14) | (t1 & MASK4);
		let t3 = ((t2 & MASK5) >> 28) | (t2 & MASK6);
		return t3;
	}

    return (get_col_turntable()[(shrink_bit_8(my_t) << 11) as usize | (shrink_bit_8(op_t) << 3) as usize | x] << y) & COL_MASK[y];
}

// x番目の斜のplaceableを返す
// x \in [0, 14)で、
// 0: 0
// 1: 1, 8
// 2: 2, 9, 16 ...
// のようにindexを(x, y)とした時、x + yで管理
fn get_diag_flip_1(board: &play::Board, x : usize, y : usize) -> u64{
    let diag1_idx = x + y;
	let my_t = (board.my_board & DIAG_MASK_1[diag1_idx]) >> diag1_idx;
	let op_t = (board.opponent_board & DIAG_MASK_1[diag1_idx]) >> diag1_idx;

	const MASK1 : u64 = 0b10000000000000100000000000001000000000000010000000;
	const MASK2 : u64 = 0b00000001000000000000010000000000000100000000000001;
	const MASK3 : u64 = 0b00000011000000000000000000000000001100000000000000;
	const MASK4 : u64 = 0b00000000000000000000110000000000000000000000000011;
	const MASK5 : u64 = 0b00000000000000000011110000000000000000000000000000;
	const MASK6 : u64 = 0b00000000000000000000000000000000000000000000001111;
	
    #[inline(always)]
	fn shrink_bit_7(data: u64) -> u64 {
		let t1 = ((data & MASK1) >> 6) | (data & MASK2);
		let t2 = ((t1 & MASK3) >> 12) | (t1 & MASK4);
		let t3 = ((t2 & MASK5) >> 24) | (t2 & MASK6);
		return t3;
	}

    return (get_diag_turntable_1()[(shrink_bit_7(my_t) << 11) as usize | (shrink_bit_7(op_t) << 3) as usize | x] << diag1_idx) & DIAG_MASK_1[diag1_idx];
}

// x番目の斜のplaceableを返す
// x \in [0, 14)で、
// 0: 7
// 1: 6, 15
// 2: 5, 14, 23
// のようにindexを(x, y)とした時、7 + x - yで管理
fn get_diag_flip_2(board: &play::Board, x : usize, y : usize) -> u64{
    let diag2_idx = 7 + x - y;
	let my_t = if diag2_idx < 8 
        {(board.my_board & DIAG_MASK_2[diag2_idx]) >> (7 - diag2_idx)}
    else
        {(board.my_board & DIAG_MASK_2[diag2_idx]) << (diag2_idx - 7)};
	let op_t = if diag2_idx < 8 
        {(board.opponent_board & DIAG_MASK_2[diag2_idx]) >> (7 - diag2_idx)}
    else
        {(board.opponent_board & DIAG_MASK_2[diag2_idx]) << (diag2_idx - 7)};

	const MASK1 : u64 = 0b1000000000000000001000000000000000001000000000000000001000000000;
	const MASK2 : u64 = 0b0000000001000000000000000001000000000000000001000000000000000001;
	const MASK3 : u64 = 0b0000000011000000000000000000000000000000000011000000000000000000;
	const MASK4 : u64 = 0b0000000000000000000000000011000000000000000000000000000000000011;
	const MASK5 : u64 = 0b0000000000000000000000001111000000000000000000000000000000000000;
	const MASK6 : u64 = 0b0000000000000000000000000000000000000000000000000000000000001111;
	
    #[inline(always)]
	fn shrink_bit_9(data: u64) -> u64 {
		let t1 = ((data & MASK1) >> 8) | (data & MASK2);
		let t2 = ((t1 & MASK3) >> 16) | (t1 & MASK4);
		let t3 = ((t2 & MASK5) >> 32) | (t2 & MASK6);
		return t3;
	}

    let t = get_diag_turntable_2()[(shrink_bit_9(my_t) << 11) as usize | (shrink_bit_9(op_t) << 3) as usize | x];
    return (if diag2_idx < 8 {
        t << (7 - diag2_idx)
    } else {
        t >> (diag2_idx - 7)
    }) & DIAG_MASK_2[diag2_idx];
}

pub fn get_flipper(board: &play::Board, i: usize) -> u64 {
    let x = i >> 3;
    let y = i & 7;
    let mut flipper = 0u64;
    flipper |= get_row_flip(board, x, y);
    flipper |= get_col_flip(board, x, y);
    flipper |= get_diag_flip_1(board, x, y);
    flipper |= get_diag_flip_2(board, x, y);
    return flipper;
}

// グローバルカウンター
static EVAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
static NODE_COUNTER: AtomicUsize = AtomicUsize::new(0);

// バックグラウンド思考専用カウンター
static BACKGROUND_NODE_COUNTER: AtomicUsize = AtomicUsize::new(0);

// カウンター操作関数
pub fn increment_eval_count() {
    EVAL_COUNTER.fetch_add(1, Ordering::Relaxed);
}

pub fn increment_node_count() {
    NODE_COUNTER.fetch_add(1, Ordering::Relaxed);
}

pub fn increment_background_node_count() -> usize {
    BACKGROUND_NODE_COUNTER.fetch_add(1, Ordering::Relaxed) + 1
}

pub fn get_eval_count() -> usize {
    EVAL_COUNTER.load(Ordering::Relaxed)
}

pub fn get_node_count() -> usize {
    NODE_COUNTER.load(Ordering::Relaxed)
}

pub fn get_background_node_count() -> usize {
    BACKGROUND_NODE_COUNTER.load(Ordering::Relaxed)
}

pub fn reset_counters() {
    EVAL_COUNTER.store(0, Ordering::Relaxed);
    NODE_COUNTER.store(0, Ordering::Relaxed);
}

pub fn reset_background_counter() {
    BACKGROUND_NODE_COUNTER.store(0, Ordering::Relaxed);
}

// (自分, 相手)の石の平均自由度を計算する
pub fn calc_freedom(board: &play::Board) -> (f32, f32) {
    let all_stone = board.my_board | board.opponent_board;
    let shift_1 = !all_stone << 1;
    let shift_8 = !all_stone << 8;
    let shift_7 = !all_stone << 7;
    let shift_9 = !all_stone << 9;
    let shift_n1 = !all_stone >> 1;
    let shift_n8 = !all_stone >> 8;
    let shift_n7 = !all_stone >> 7;
    let shift_n9 = !all_stone >> 9;

    let helper = |t: u64| -> f32 {
        ((t & shift_1).count_ones() as f32 +
        (t & shift_8).count_ones() as f32 +
        (t & shift_7).count_ones() as f32 +
        (t & shift_9).count_ones() as f32 +
        (t & shift_n1).count_ones() as f32 +
        (t & shift_n8).count_ones() as f32 +
        (t & shift_n7).count_ones() as f32 +
        (t & shift_n9).count_ones() as f32) / t.count_ones() as f32
    };
    let my_freedom = helper(board.my_board);
    let opponent_freedom = helper(board.opponent_board);

    return (my_freedom, opponent_freedom);
}

// パターンマッチ用
// const PATTERN_1 = 0b11111111_01000010_00000000_00000000_00000000_00000000_00000000_00000000;
// const PATTERN_2 = 0b10000000_11000000_10000000_10000000_10000000_10000000_11000000_10000000;
// const PATTERN_3 = 0b00000000_00000000_00000000_00000000_00000000_00000000_01000010_11111111;
// const PATTERN_4 = 0b00000001_00000011_00000001_00000001_00000001_00000001_00000011_00000001;


fn print_board_u64(board : u64){
	for i in 0..8 {
		println!("{:08b}", (board >> ((7 - i) * 8)) & 0xFF);
	}
	println!("\n");
}

fn print_board(board : &play::Board){
    for i in 0..8{
        for j in 0..8{
            if board.my_board & (1 << ((7 - i) * 8 + (7 - j))) != 0 {
                print!("O");
            } else if board.opponent_board & (1 << ((7 - i) * 8 + (7 - j))) != 0 {
                print!("X");
            } else {
                print!(".");
            }
        }
        println!();
    }
}

#[derive(Copy, Clone)]
pub struct CacheElement {
    pub my_board: u64,
    pub opponent_board: u64,
    pub depth: u8,
    pub next_move: u8,
    pub value: f32,
    pub exact: bool,
    pub complete: bool,
    pub lower_bound: bool,
    pub upper_bound: bool
}

impl CacheElement {
    pub const fn new() -> Self {
        CacheElement {
            my_board: 0,
            opponent_board: 0,
            depth: 0,
            value: 0.0,
            next_move: 255,
            exact: false,
            complete: false,
            lower_bound: false,
            upper_bound: false
        }
    }
}

pub struct Cache {
    elements: Box<[CacheElement]>  // Boxを使用してスタックオーバーフロー防止
}

impl Cache {
    pub fn new() -> Self {
        const SIZE : usize = 1 << 22;
        let elements = (0..SIZE)
            .map(|_| CacheElement::new())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        
        Cache {elements}
    }

    // ハッシュ関数、安全だがあまり高速でないことに注意
    // 掛け算をxorに変更したが、安全性?
    pub fn hash(&self, board : &play::Board) -> usize{
        let mut key1 = board.my_board;
        key1 = (key1 ^ (key1 >> 30)) ^ 0xbf58476d1ce4e5b9;
        key1 = (key1 ^ (key1 >> 27)) ^ 0x94d049bb133111eb;
        key1 ^= key1 >> 31;
        let mut key2 = board.opponent_board;
        key2 = (key2 ^ (key2 >> 30)) ^ 0xbf58476d1ce4e5b9;
        key2 = (key2 ^ (key2 >> 27)) ^ 0x94d049bb133111eb;
        key2 ^= key2 >> 31;
        return ((key1 ^ key2) & 0x3FFFFF) as usize;
    }

    // getメソッドの修正 - 借用を返すように変更
    pub fn get(&self, board: &play::Board) -> Option<&CacheElement> {
        let index = self.hash(board);
        let element = &self.elements[index];
        if element.my_board == board.my_board && element.opponent_board == board.opponent_board {
            Some(element)
        } else {
            None
        }
    }

    pub fn set(&mut self, board: &play::Board, depth: u8, next_move: u8, value: f32, exact: bool, complete: bool, lower_bound: bool, upper_bound: bool) {
        let index = self.hash(board);
        let element = &mut self.elements[index];
        if element.my_board == board.my_board && element.opponent_board == board.opponent_board && depth < element.depth {
            return;
        }
        element.my_board = board.my_board;
        element.opponent_board = board.opponent_board;
        element.depth = depth;
        element.next_move = next_move;
        element.value = value;
        element.exact = exact;
        element.complete = complete;
        element.lower_bound = lower_bound;
        element.upper_bound = upper_bound;
    }
}
static CACHE: LazyLock<std::sync::Mutex<Cache>> = LazyLock::new(|| {
    std::sync::Mutex::new(Cache::new())
});
pub fn get_cache() -> std::sync::MutexGuard<'static, Cache> {
    CACHE.lock().unwrap()
}

use hashbrown::hash_map as base;
static BOOK: LazyLock<std::sync::Mutex<base::HashMap<play::Board, u8>>> = LazyLock::new(|| {
    std::sync::Mutex::new(base::HashMap::new())
});

static INIT_BOOK: Once = Once::new();
pub fn init_book() {
    INIT_BOOK.call_once(|| {
        let mut file = File::open("src/book.bin").expect("Failed to open book.bin");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read book.bin");

        let mut i = 0;
        let mut j = 0;
        fn succ(i: usize, j: usize) -> (usize, usize) {
            if j == 7 {
                (i + 1, 0)
            }
            else {
                (i, j + 1)
            }
        }
        
        let mut book = BOOK.lock().unwrap();
        
        loop {
            let mut seen = 0;
            let mut board = play::Board{
                my_board: 0,
                opponent_board: 0
            };

            while seen < 64 {
                let b1 = (buffer[i] >> j) & 1 != 0;
                (i, j) = succ(i, j);
                let b2 = (buffer[i] >> j) & 1 != 0;
                (i, j) = succ(i, j);
                match (b1, b2) {
                    (false, false) => {
                        seen += 1;
                    }

                    (true, false) => {
                        board.my_board |= 1 << seen;
                        seen += 1;
                    }

                    (false, true) => {
                        board.opponent_board |= 1 << seen;
                        seen += 1;
                    }

                    (true, true) => {
                        let mut t: u8 = 0;
                        for k in 0..5 {
                            t |= ((buffer[i] >> j) & 1) << k;
                            (i, j) = succ(i, j);
                        }
                        seen += t as usize;
                    }
                }
            }

            let mut mv: u8 = 0;
            for k in 0..6 {
                mv |= ((buffer[i] >> j) & 1) << k;
                (i, j) = succ(i, j);
            }

            unsafe{
                book.insert(board, mv);
            }
            //println!("Inserted board:\n{}\n, move: {}", board, mv);

            if i >= buffer.len() - 1 {
                break;
            }
        }
    });
}

fn get_eq_board(board: &play::Board) -> [play::Board; 8] {
    fn turn_board(board: &play::Board) -> play::Board {
		let mut new_board = play::Board {
			my_board: 0,
			opponent_board: 0,
		};
		for i in 0..64 {
			let x = i % 8;
			let y = i / 8;
			if board.check_mine(x, y) {
				new_board.my_board |= 1 << ((7 - x) * 8 + y);
			} else if board.check_opponent(x, y) {
				new_board.opponent_board |= 1 << ((7 - x) * 8 + y);
			}
		}
		new_board
	}

	fn flip_board(board: &play::Board) -> play::Board {
		let mut new_board = play::Board {
			my_board: 0,
			opponent_board: 0,
		};
		for i in 0..64 {
			let x = i % 8;
			let y = i / 8;
			if board.check_mine(x, y) {
				new_board.my_board |= 1 << (y * 8 + (7 - x));
			} else if board.check_opponent(x, y) {
				new_board.opponent_board |= 1 << (y * 8 + (7 - x));
			}
		}
		new_board
	}

	let mut eq_boards = [play::Board {
		my_board: 0,
		opponent_board: 0,
	}; 8];
	eq_boards[0] = board.clone();
	eq_boards[1] = turn_board(&eq_boards[0]);
	eq_boards[2] = turn_board(&eq_boards[1]);
	eq_boards[3] = turn_board(&eq_boards[2]);
	eq_boards[4] = flip_board(&eq_boards[0]);
	eq_boards[5] = turn_board(&eq_boards[4]);
	eq_boards[6] = turn_board(&eq_boards[5]);
	eq_boards[7] = turn_board(&eq_boards[6]);

	eq_boards
}

pub fn lookup_book(input_board: &play::Board) -> Option<u8> {
    let book = BOOK.lock().unwrap();
    let mut board = input_board.clone();
    board.change_turn();

    fn conv(x: u8, y: u8) -> u8 {
        y * 8 + x
    }

    let eq_boards = get_eq_board(&board);
    for i in 0..8 {
        if let Some(&mv) = book.get(&eq_boards[i]) {
            let x = mv % 8;
            let y = mv / 8;

            return match i {
                0 => Some(conv(x, y)), 
                1 => Some(conv(7 - y, x)),
                2 => Some(conv(7 - x, 7 - y)),
                3 => Some(conv(y, 7 - x)),
                4 => Some(conv(7 - x, y)),
                5 => Some(conv(y, x)),
                6 => Some(conv(x, 7 - y)),
                7 => Some(conv(7 - y, 7 - x)),
                _ => unreachable!(),
            };
        }
    }

    None
}

use std::io::Write;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_operations() {
        // テーブルの初期化（最初の呼び出し時のみ実行される）
        initialize_tables();
        
        println!("=== テストボード1 ===");
        // mine : O, opponent : X, empty : .
        let test_board_1 = play::Board {
            my_board:       0b00000000_00000100_00001000_00110100_00000000_00000000_00000000_00000000,
            opponent_board: 0b00000000_00000000_00110000_00001000_00011100_00100000_00000000_00000000
        };
        
        print_board(&test_board_1);
        
        println!("Row placeable:");
        let mut data_row = 0u64;
        for i in 0..8 {
            data_row |= get_row_placeable(&test_board_1, i);
        }
        print_board_u64(data_row);

        println!("Col placeable:");
        let mut data_col = 0u64;
        for i in 0..8 {
            data_col |= get_col_placeable(&test_board_1, i);
        }
        print_board_u64(data_col);

        println!("Diag1 placeable:");
        let mut data_diag1 = 0u64;
        for i in 0..15 {
            data_diag1 |= get_diag1_placeable(&test_board_1, i);
        }
        print_board_u64(data_diag1);

        println!("Diag2 placeable:");
        let mut data_diag2 = 0u64;
        for i in 0..15 {
            data_diag2 |= get_diag2_placeable(&test_board_1, i);
        }
        print_board_u64(data_diag2);

        println!("All placeable:");
        print_board_u64(get_placeable(&test_board_1));

        println!("\n=== テストボード2 ===");
        let test_board_2 = play::Board {
            my_board:       0b00000000_00010000_10000000_01000000_00111000_00010100_00000000_00111000,
            opponent_board: 0b00000100_10000100_01111111_00111111_00000110_01101000_10111111_00000010
        };
        
        print_board(&test_board_2);
        
        println!("Row placeable:");
        let mut data_row = 0u64;
        for i in 0..8 {
            data_row |= get_row_placeable(&test_board_2, i);
        }
        print_board_u64(data_row);
        
        println!("Col placeable:");
        let mut data_col = 0u64;
        for i in 0..8 {
            data_col |= get_col_placeable(&test_board_2, i);
        }
        print_board_u64(data_col);
        
        println!("Diag1 placeable:");
        let mut data_diag1 = 0u64;
        for i in 0..15 {
            data_diag1 |= get_diag1_placeable(&test_board_2, i);
        }
        print_board_u64(data_diag1);
        
        println!("Diag2 placeable:");
        let mut data_diag2 = 0u64;
        for i in 0..15 {
            data_diag2 |= get_diag2_placeable(&test_board_2, i);
        }
        print_board_u64(data_diag2);

        println!("All placeable:");
        print_board_u64(get_placeable(&test_board_2));

        println!("=== フリップテスト ===");
        println!("Flips (0, 0):");
        print_board_u64(get_flipper(&test_board_2, 63));
        
        println!("Flips (1, 1):");
        print_board_u64(get_flipper(&test_board_2, 54));
        
        println!("Flips (4, 1):");
        print_board_u64(get_flipper(&test_board_2, 30));
        
        println!("Flips (0, 1) will fail:");
        print_board_u64(get_flipper(&test_board_2, 62));

        let test_board_3 = play::Board {
            my_board: 0x0000000810000000,
            opponent_board: 0x0000001008000000,
        };
        println!("Flips (2, 3):");
        print_board_u64(get_flipper(&test_board_3, 19));
    }

    #[test]
    fn test_book() {
        init_book();

        let book = BOOK.lock().unwrap();
        let mut file = std::fs::File::create("log.txt").expect("Failed to create log.txt");

        writeln!(file, "Book contents ({} entries):", book.len()).unwrap();
        for (board, &mv) in book.iter() {
            writeln!(file, "Board: {}", board).unwrap();
            writeln!(file, "Move: {} (x={}, y={})", mv, mv % 8, mv / 8).unwrap();
            writeln!(file, "---").unwrap();
        }

        println!("Book contents written to log.txt");
    }
}
