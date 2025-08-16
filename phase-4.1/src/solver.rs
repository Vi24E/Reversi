use crate::play::{Board, TimeManager};
use itertools::Itertools;
use std::io::{self, Write};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::hash::{BuildHasher, Hasher};

#[derive(Default)]
pub struct IdentityHasher {
    value: u64,
}

impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, bytes: &[u8]) {
        // u64のバイト列をそのまま使用（恒等写像）
        if bytes.len() >= 8 {
            self.value = u64::from_ne_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7]
            ]);
        }
    }

    fn write_u64(&mut self, i: u64) {
        // phase-2のハッシュ関数を適用する場合
        let mut key = i;
        key = (key ^ (key >> 30)) ^ 0xbf58476d1ce4e5b9;
        key = (key ^ (key >> 27)) ^ 0x94d049bb133111eb;
        key ^= key >> 31;
        self.value = key;
    }
}

#[derive(Default)]
pub struct MyHash;

impl BuildHasher for MyHash {
    type Hasher = IdentityHasher;

    fn build_hasher(&self) -> Self::Hasher {
        IdentityHasher::default()
    }
}

// カスタムハッシャーを使用するHashMap型
type Cache = HashMap<(u64, u64), (i8, u8), MyHash>;

static CACHE_SOLVER: LazyLock<std::sync::Mutex<Cache>> = LazyLock::new(|| {
    std::sync::Mutex::new(HashMap::with_hasher(MyHash))
});

pub fn get_cache() -> std::sync::MutexGuard<'static, Cache> {
    CACHE_SOLVER.lock().unwrap()
}

pub fn solve(board: &Board, time_manager: &TimeManager) -> (i8, u8) {
	let moves = board.get_valid_moves();
	unsafe {
		return _solve(board, moves, time_manager);
	}
}

fn _solve(board: &Board, moves: u64, time_manager: &TimeManager) -> (i8, u8) {
	if let Some(p) = get_cache().get(&(board.my_board, board.opponent_board)) {
		return *p;
	}

	if time_manager.should_stop() {
        return (-2, 64);
    }

    if moves == 0 {
        let mut t = board.clone();
        t.change_turn();
		let new_moves = t.get_valid_moves();
        if new_moves == 0 {
            let my_count = t.my_board.count_ones();
            let op_count = t.opponent_board.count_ones();
            if my_count > op_count {
                get_cache().insert((board.my_board, board.opponent_board), (1, 64));
                return (1, 64);
            }
            else if my_count < op_count {
                get_cache().insert((board.my_board, board.opponent_board), (-1, 64));
                return (-1, 64);
            }
            else {
                get_cache().insert((board.my_board, board.opponent_board), (0, 64));
                return (0, 64);
            }
        }
        let (res, _) = _solve(&t, new_moves, time_manager);

		get_cache().insert((board.my_board, board.opponent_board), (-res, 64));
        return (-res, 64);
    }

	let count_stones = board.my_board.count_ones() + board.opponent_board.count_ones();
	let mut ordered_moves: Vec<(u8, u8, Board, u64)> = {
		let mut vec = Vec::new();
		let mut left = moves;
		while left != 0 {
			let m = left.trailing_zeros() as u8;
			left &= !(1 << m);
			let mut t = board.clone();
			t.do_move(m);
			t.change_turn();
			let next_moves = t.get_valid_moves();
			let eval = next_moves.count_ones() as u8;
			vec.push((eval, m, t, next_moves));
		}
		vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
		vec
	};
	
	let mut best_move = 64;
	for (_, mv, next_board, next_moves) in ordered_moves {
		let (res, _res_move) = {
			// 61で1個開き
			if count_stones <= 59 {
				_solve(&next_board, next_moves, time_manager)
			}
			else {
				_solve_leaf(&next_board, next_moves, time_manager)
			}
		};
		match res {
			1 => {
				continue;
			}
			-1 => {
				get_cache().insert((board.my_board, board.opponent_board), (1, mv));
				return (1, mv);
			}
			0 => {
				best_move = mv;
			}
			-2 => {
				return (-2, 64);
			}
			_ => unreachable!(),
		}
	}

	if best_move == 64 {
		get_cache().insert((board.my_board, board.opponent_board), (-1, 64));
		(-1, 64)
	}
	else {
		get_cache().insert((board.my_board, board.opponent_board), (0, best_move));
		(0, best_move)
	}
}


fn _solve_leaf(board: &Board, moves: u64, time_manager: &TimeManager) -> (i8, u8) {
	if time_manager.should_stop() {
        return (-2, 64);
    }

    if moves == 0 {
        let mut t = board.clone();
        t.change_turn();
		let new_moves = t.get_valid_moves();
        if new_moves == 0 {
            let my_count = t.my_board.count_ones();
            let op_count = t.opponent_board.count_ones();
            if my_count > op_count {
                return (1, 64);
            }
            else if my_count < op_count {
                return (-1, 64);
            }
            else {
                return (0, 64);
            }
        }
        let (res, _) = _solve(&t, new_moves, time_manager);

        return (-res, 64);
    }

	let count_stones = board.my_board.count_ones() + board.opponent_board.count_ones();
	let mut ordered_moves: Vec<(u8, Board, u64)> = {
		let mut vec = Vec::new();
		let mut left = moves;
		while left != 0 {
			let m = left.trailing_zeros() as u8;
			left &= !(1 << m);
			let mut t = board.clone();
			t.do_move(m);
			t.change_turn();
			let next_moves = t.get_valid_moves();
			vec.push((m, t, next_moves));
		}
		vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
		vec
	};

	
	let mut best_move = 64;
	for (mv, next_board, next_moves) in ordered_moves {
		let (res, _res_move) = {
			// 61で1個開き
			if count_stones <= 61 {
				_solve_leaf(&next_board, next_moves, time_manager)
			}
			else {
				_solve_leaf_one(&next_board, next_moves)
			}
		};
		match res {
			1 => {
				continue;
			}
			-1 => {
				return (1, mv);
			}
			0 => {
				best_move = mv;
			}
			-2 => {
				return (-2, 64);
			}
			_ => unreachable!(),
		}
	}

	if best_move == 64 {
		(-1, 64)
	}
	else {
		(0, best_move)
	}
}


fn _solve_leaf_one(board: &Board, moves: u64) -> (i8, u8) {
	fn judge(board: &Board) -> i8 {
		let my_count = board.my_board.count_ones();
		let op_count = board.opponent_board.count_ones();
		if my_count > op_count {
			return 1;
		}
		else if my_count < op_count {
			return -1;
		}
		else {
			return 0;
		}
	}

	if moves != 0 {
		let mut b = board.clone();
		let mv = moves.trailing_zeros() as u8;
		b.do_move(mv);
		return (judge(&b), mv);
	}
	else {
		let mut b = board.clone();
		b.change_turn();
		// need refactor
		let new_move = b.get_valid_moves();
		if new_move == 0 {
			return (judge(board), 64);
		}
		let mv = new_move.trailing_zeros() as u8;
		b.do_move(mv);
		return (-judge(&b), mv);
	}
}



fn str_to_board(s: &String) -> Option<Board> {
    let mut my_board = 0u64;
    let mut opponent_board = 0u64;

    for i in 0..64 {
        let c = s.chars().nth(i)?;
		// 'X'が手番側、
		// 'O'が相手側
		// 自分の実装と逆のため、注意!
        match c {
            'X' => my_board |= 1 << i,
            'O' => opponent_board |= 1 << i,
            _ => continue,
        }
    }

	Some(Board {
		my_board,
		opponent_board,
	})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver() {
        /*
		OOOX-OX--XXX-O--XXXXXO----XOXXO-OOOOOOOO-OOOXX------XOOO-----O-X 0 (24)
		---X-OX---XX-O-XOO-XXOXX-OOXXOXX--XOXOX--XXXOXXX--OOOOOO----OOO- -6 (23)
		XO-O---XOO-O-XXXXOXOXOOX-OXOOXOX-OXOOOOOO-XXOOOO---XOOOOXXXXXXXX -2 (13)
		------XXXXXXXXX--XOOXXX-XOOOXXXXOOOOOOOXO-XO-OOXOOO-O-O-XOX--O-- -4 (19)
		OOO-OO-X-OXOOOXXXXOOOOXXXXOXXOOX-OXXOOOOO-XOOOOO--XX-OOX-OX--OOO 2 (11)
		-O--XO----O-OXO-XXOOOOOOXXOOXXOOXXOXOOX--XOXOO---OOXX-O-OOO-X--- 2 (20)
		---O-X--O-OOXX--XOXOXXXO-XOOOXX-XOXOXO-XOXXXXXX-XOXXOO--XXXX-OOO 0 (16)

		 */

		fn test(s: &'static str){
			let time_manager = TimeManager::new(100000);
			let board = str_to_board(&s.to_string()).unwrap();
			let start = std::time::Instant::now();
			let (res, mv) = solve(&board, &time_manager);
			let duration = start.elapsed();
			eprintln!("Solve took: {:?}", duration);
			eprintln!("Result: {}, Move: {}", res, mv);
		}

		//test("-O--XO----O-OXO-XXOOOOOOXXOOXXOOXXOXOOX--XOXOO---OOXX-O-OOO-X---"); //20
		// test("XO-O---XOO-O-XXXXOXOXOOX-OXOOXOX-OXOOOOOO-XXOOOO---XOOOOXXXXXXXX"); //13
		// test("---O-X--O-OOXX--XOXOXXXO-XOOOXX-XOXOXO-XOXXXXXX-XOXXOO--XXXX-OOO"); //16
		// test("-O--------OXXXX-OXXOXXXX-OXOOOO-OOXXOOO-OXXOOOOOO-XXOOO-O-XXXXX-"); //17
		// test("XXO-----OXOX----OXXO-XO-OXOXOO--OXOXOO--OOOOXXXXOOOOOOO---OOOOOO"); //18 =
		// test("---OOOOO---XXOOO-O--XOXOOOOOOXO---XOOXO-OXOOOOO-X-OOOOO---OOOXXX"); //18 =
		// test("---O--X--OOOOOXO---XOXX-OOXXOOXX-O-OOXXXXOOOXXXX-OOOOOO--O-XOO-O"); //18 =
		// test("XXX-OX-OX-XXXXXXOOOXOOXX-OXOXOX-OXOOOX--OOOOOOX-OOOO----O-X-----"); //18 -
		// test("OXX------XOXXX-XXXXOX-X-OXOXOXOO-OXXXOOOO-XXOOXO--OXOOOO-OX--O--"); //18 +
		// test("OXX------XOXXX-XXXXOX-X-OXOXOXOO-OXXXOOOO-XXOOXO--OXOOOO-OX--O--"); //18 +
		test("✓··X····O·XXX···OOXXXX✓OOOOXOOOOOOXOXXOOOOOOOXOOXOOOO✓X✓OO✓XO✓··");
		test("XXXXXXX·XXXXXO✓OXOXOXOOOXOOXOOOOXOXOOOOOXXXOOOOOXOXOOOO✓OOOOOOOO"); //18 -
    }
}



/*
(0..64) case:
Solve took: 116.171458ms
Result: -1, Move: 64
Solve took: 1.278940625s
Result: 0, Move: 24
Solve took: 11.6922265s
Result: 0, Move: 63
Solve took: 23.788968667s
Result: 0, Move: 38
Solve took: 16.205126708s
Result: 0, Move: 57
Solve took: 32.204844667s
Result: 0, Move: 58
Solve took: 2.625159958s
Result: -1, Move: 64

trailing_zeros case:
Solve took: 102.716459ms
Result: -1, Move: 64
Solve took: 1.002814s
Result: 0, Move: 24
Solve took: 10.760402833s
Result: 0, Move: 63
Solve took: 20.663799542s
Result: 0, Move: 38
Solve took: 13.440456708s
Result: 0, Move: 57
Solve took: 27.272013625s
Result: 0, Move: 58
Solve took: 2.1596595s
Result: -1, Move: 64

with unrolled_loops:
Solve took: 91.199125ms
Result: -1, Move: 64
Solve took: 833.919833ms
Result: 0, Move: 24
Solve took: 8.588502584s
Result: 0, Move: 63
Solve took: 17.456547834s
Result: 0, Move: 38
Solve took: 11.26203725s
Result: 0, Move: 57
Solve took: 22.607979333s
Result: 0, Move: 58
Solve took: 1.704176125s
Result: -1, Move: 64

with specialized small case: 58
Solve took: 91.928083ms
Result: -1, Move: 64
Solve took: 293.624084ms
Result: 1, Move: 24
Solve took: 7.813963625s
Result: 0, Move: 63
Solve took: 15.353969s
Result: 0, Move: 38
Solve took: 9.485016791s
Result: 0, Move: 57
Solve took: 23.117414417s
Result: 0, Move: 58
Solve took: 1.691424875s
Result: -1, Move: 64
Solve took: 10.454721s
Result: 1, Move: 63
Solve took: 1.167µs
Result: 1, Move: 63

with specialized small case: 59
Solve took: 80.909208ms
Result: -1, Move: 64
Solve took: 289.859333ms
Result: 1, Move: 24
Solve took: 8.46460425s
Result: 0, Move: 63
Solve took: 14.234936542s
Result: 0, Move: 38
Solve took: 10.113279958s
Result: 0, Move: 57
Solve took: 23.19773575s
Result: 0, Move: 58
Solve took: 1.542678s
Result: -1, Move: 64
Solve took: 9.345129583s
Result: 1, Move: 63
Solve took: 1.375µs
Result: 1, Move: 63

Solve took: 74.43125ms
Result: -1, Move: 64
Solve took: 289.505458ms
Result: 1, Move: 24
Solve took: 8.419978125s
Result: 0, Move: 63
Solve took: 14.68861375s
Result: 0, Move: 38
Solve took: 10.293281917s
Result: 0, Move: 57
Solve took: 23.28176775s
Result: 0, Move: 58
Solve took: 1.549420041s
Result: -1, Move: 64
Solve took: 9.226593958s
Result: 1, Move: 63
Solve took: 1.584µs
Result: 1, Move: 63

*/