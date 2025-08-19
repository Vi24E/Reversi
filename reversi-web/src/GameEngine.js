// オセロの処理を担当するクラス
export class GameEngine {
	constructor(wasmModule) {
		this.wasmModule = wasmModule;
		this.initialBoard = '...........................WB......BW...........................';
		this.BoardHistory = [];
		this.CurrentBoardIdx = 0;
		// ゆくゆくはNilに変更
		this.playerModes = {
			black : 'human',
			white : 'human'
		};
		this.reset();
	}

	// ゲームをリセット
	reset() {
		this.boardStr = this.initialBoard;
		this.BoardHistory = [{board: this.initialBoard, turn: 0}];
		this.CurrentBoardIdx = 0;
		this.turn = 0;
	}

	// 現在の盤面状態を取得
	getCurrentBoard() {
		return this.boardStr;
	}

	// 現在のターン数を取得
	getCurrentTurn() {
		return this.turn;
	}

	setPlayerMode(BlackMode, WhiteMode){
		this.playerModes = {
			black : BlackMode,
			white : WhiteMode
		};
	}

	// 現在のプレイヤー（false: 黒, true: 白）
	getCurrentPlayer() {
		return this.turn % 2 !== 0;
	}

	getCurrentPlayerType() {
		return this.turn % 2 === 0 ? this.playerModes.black : this.playerModes.white;
	}

	getOpponentPlayerType() {
		return this.turn % 2 === 0 ? this.playerModes.white : this.playerModes.black;
	}

	// 指定座標の石の状態を取得（0:空, 1:黒石, 2:白石）
	getCell(row, col) {
		const pos = row * 8 + col;
		const char = this.boardStr[pos];
		if (char === 'B') return 1;
		if (char === 'W') return 2;
		return 0;
	}

	// 黒石の数を取得
	getBlackStoneCount() {
		return (this.boardStr.match(/B/g) || []).length;
	}

	// 白石の数を取得
	getWhiteStoneCount() {
		return (this.boardStr.match(/W/g) || []).length;
	}

	// 指定座標が合法手かどうかを判定
	isValidMove(row, col) {
		try {
			const place = row * 8 + col;

			// 既に石がある場合は無効
			if (this.boardStr[place] !== '.') return false;

			const validMovesStr = this.wasmModule.get_valid_moves(this.boardStr, this.getCurrentPlayer());
			return validMovesStr[place] === '1';
		} catch (error) {
			console.error('Error in isValidMove:', error);
			return false;
		}
	}

	// 有効手のビットマスク文字列を取得
	getValidMoves() {
		try {
			return this.wasmModule.get_valid_moves(this.boardStr, this.getCurrentPlayer());
		} catch (error) {
			console.error('Error getting valid moves:', error);
			return '0'.repeat(64); // エラー時は空の有効手
		}
	}

	// 直前の盤面に戻す
	undoBoard() {
		if (this.CurrentBoardIdx > 0) {
			this.CurrentBoardIdx -= 1;
			this.boardStr = this.BoardHistory[this.CurrentBoardIdx].board;
			this.turn = this.BoardHistory[this.CurrentBoardIdx].turn;
		}
	}

	// 直前の盤面に戻せるか
	undoable() {
		return this.CurrentBoardIdx > 0;
	}

	// 直後の盤面に進める
	redoBoard() {
		if (this.CurrentBoardIdx < this.BoardHistory.length - 1) {
			this.CurrentBoardIdx += 1;
			this.boardStr = this.BoardHistory[this.CurrentBoardIdx].board;
			this.turn = this.BoardHistory[this.CurrentBoardIdx].turn;
		}
	}

	// 直後の盤面に進めるか
	redoable() {
		return this.CurrentBoardIdx < this.BoardHistory.length - 1;
	}

	// 手を実行
	makeMove(row, col) {
		if (!this.isValidMove(row, col)) {
			return {success: false, message: '無効な手です'};
		}

		try {
			const pos = row * 8 + col;
			const currentPlayer = this.getCurrentPlayer();

			// WebAssemblyで盤面を更新
			const updatedBoardStr = this.wasmModule.update_board(this.boardStr, pos, !currentPlayer);
			this.boardStr = updatedBoardStr;
			this.turn += 1;

			// パス処理
			const passMessage = this.handlePass();

			if (this.BoardHistory.length !== this.CurrentBoardIdx + 1){
				if (this.BoardHistory[this.CurrentBoardIdx + 1].board === updatedBoardStr){
					this.CurrentBoardIdx += 1;
				}
				else {
					this.BoardHistory = this.BoardHistory.slice(0, this.CurrentBoardIdx + 1);
					this.BoardHistory.push({board: updatedBoardStr, turn: this.turn});
					this.CurrentBoardIdx += 1;
				}
			}
			else{
				this.BoardHistory.push({board: updatedBoardStr, turn: this.turn});
				this.CurrentBoardIdx += 1;
			}

			return {
				success: true,
				passMessage: passMessage,
				gameState: this.getGameState(),
			};
		} catch (error) {
			console.error('Error during move:', error);
			return {success: false, message: 'エラーが発生しました'};
		}
	}

	// パス処理（連続パス対応）
	handlePass() {
		let passMessage = '';
		const nextPlayer = this.getCurrentPlayer();

		if (this.wasmModule.is_pass(this.boardStr, nextPlayer)) {
			const playerName = nextPlayer ? '白' : '黒';
			passMessage = `${playerName}はパスです`;
			this.turn += 1;
		}
		return passMessage;
	}

	// ゲーム状態を取得
	/*
	WhiteWin
	BlackWin
	Draw
	Ongoing
	*/
	getGameState() {
		try {
			return this.wasmModule.get_result(this.boardStr);
		} catch (error) {
			console.error('Error getting game state:', error);
			return 'Ongoing';
		}
	}

	// ゲームが終了しているかどうか
	isGameFinished() {
		const state = this.getGameState();
		return state !== 'Ongoing';
	}

	// AI手を取得（将来の実装用）
	getAIMove(timeMs = 1000) {
		try {
			return this.wasmModule.get_ai_move(this.boardStr, this.getCurrentPlayer(), timeMs);
		}
		catch (error) {
			console.error('Error getting AI move:', error);
			return 64; // パスを返す
		}
	}
}
