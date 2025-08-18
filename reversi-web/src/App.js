/* eslint-env es2020 */
import './App.css';
import React,{useState,useEffect} from 'react';
import {GameEngine} from './GameEngine';

// WebAssemblyモジュールをグローバル変数として管理
let wasmModule = null;

// アプリケーション開始時にWebAssemblyを即座に初期化開始
const wasmPromise = (async () => {
	try {
		console.log('Starting WASM initialization...');
		wasmModule = await import('fl-reversi-rs');

		if (typeof wasmModule.default === 'function') {
			await wasmModule.default();
		}

		// 初期化テスト
		const testBoard = '...........................WB......BW...........................';
		wasmModule.get_valid_moves(testBoard,false);

		console.log('WASM fully ready');
		return true;
	} catch (error) {
		console.error('WASM initialization failed:',error);
		return false;
	}
})();

// 石を描画するコンポーネント
function Stone({type,isValidMove}) {
	if (type === 1) {
		return (
			<div
				style={{
					width: 32,
					height: 32,
					borderRadius: '50%',
					background: 'black',
					margin: 'auto',
				}}
			/>
		);
	}
	if (type === 2) {
		return (
			<div
				style={{
					width: 32,
					height: 32,
					borderRadius: '50%',
					background: 'white',
					margin: 'auto',
				}}
			/>
		);
	}
	if (isValidMove) {
		return (
			<div
				style={{
					width: 8,
					height: 8,
					borderRadius: '50%',
					background: '#999999b1',
					margin: 'auto',
				}}
			/>
		);
	}
	return null;
}

// ゲーム情報を表示するコンポーネント
function GameInfo({gameEngine,passMessage,onReset}) {
	const blackCount = gameEngine.getBlackStoneCount();
	const whiteCount = gameEngine.getWhiteStoneCount();
	const currentPlayer = gameEngine.getCurrentTurn() % 2 === 0 ? '黒' : '白';
	const gameState = gameEngine.getGameState();

	return (
		<div>
			<div>現在のプレイヤー: {currentPlayer}</div>
			<div>黒石: {blackCount}個</div>
			<div>白石: {whiteCount}個</div>
			<div>ターン数: {gameEngine.getCurrentTurn()}</div>
			<div>状態: {gameState}</div>

			<button onClick={onReset}>リセット</button>
		</div>
	);
}

// メインアプリケーション
function App() {
	const [gameEngine,setGameEngine] = useState(null);
	const [wasmLoaded,setWasmLoaded] = useState(false);
	const [wasmError,setWasmError] = useState(null);
	const [passMessage,setPassMessage] = useState('');
	const [,forceUpdate] = useState({}); // 強制再描画用

	// WebAssemblyの初期化を待つ
	useEffect(() => {
		let isMounted = true;

		wasmPromise.then(success => {
			if (isMounted) {
				if (success) {
					setGameEngine(new GameEngine(wasmModule));
					setWasmLoaded(true);
					setWasmError(null);
				} else {
					setWasmError('WebAssemblyの読み込みに失敗しました');
				}
			}
		});

		return () => {
			isMounted = false;
		};
	},[]);

	// ローディング画面
	if (!wasmLoaded || !gameEngine) {
		return (
			<div className="App">
				<h1>Reversi</h1>
				<div
					style={{
						display: 'flex',
						justifyContent: 'center',
						alignItems: 'center',
						height: '200px',
						fontSize: '18px',
					}}
				>
					<div>
						<div>⏳ WebAssemblyを読み込み中...</div>
						{wasmError && (
							<div
								style={{
									color: 'red',
									fontSize: '14px',
									marginTop: '10px',
								}}
							>
								{wasmError}
							</div>
						)}
					</div>
				</div>
			</div>
		);
	}

	// マスをクリックしたときの処理
	const handleCellClick = (row, col) => {
		// ゲーム終了時またはパス中はクリック無効
		if (gameEngine.isGameFinished() || passMessage !== '') {
			return;
		}

		const result = gameEngine.makeMove(row, col);

		if (gameEngine.isGameFinished()) {
			forceUpdate({});
			return;
		}

		if (result.success) {
			// パスメッセージを表示
			if (result.passMessage) {
				setPassMessage(result.passMessage);

				// 500ミリ秒後にメッセージをクリア
				setTimeout(() => setPassMessage(''), 700);
			} else {
				setPassMessage('');
			}

			// 強制的に再描画
			forceUpdate({});
		}
	};

	// リセット処理
	const handleReset = () => {
		gameEngine.reset();
		setPassMessage('');
		forceUpdate({});
	};

	// ボードの描画
	const renderBoard = () => {
		const size = 8;
		const cellSize = 40;
		const borderSize = 2;
		const gameFinished = gameEngine.isGameFinished();
		const validMovesStr = gameEngine.getValidMoves();
		const isPassActive = passMessage !== ''; // パスメッセージがある時はパス状態

		return (
			<div style={{display: 'inline-block', position: 'relative'}}>
				<table style={{borderCollapse: 'collapse'}}>
					<tbody>
						{[...Array(size)].map((_, row) => (
							<tr key={row}>
								{[...Array(size)].map((_, col) => {
									const cellType = gameEngine.getCell(row, col);
									const isValidMove = !gameFinished && !isPassActive && validMovesStr[row * 8 + col] === '1';

									return (
										<td
											key={col}
											style={{
												width: cellSize,
												height: cellSize,
												border: '2px solid black',
												background: '#0a7d2c',
												padding: 0,
												position: 'relative',
												cursor: (gameFinished || isPassActive) ? 'not-allowed' : 'pointer',
												opacity: (gameFinished || isPassActive) ? 0.3 : 1,
												filter: (gameFinished || isPassActive) ? 'grayscale(30%)' : 'none',
												//transition: "all 0.2s ease"
											}}
											onClick={() => handleCellClick(row, col)}
										>
											<Stone type={cellType} isValidMove={isValidMove} />
										</td>
									);
								})}
							</tr>
						))}
					</tbody>
				</table>

				{/* ガイドポイント */}
				{[
					[2, 2],
					[2, 6],
					[6, 2],
					[6, 6],
				].map(([r, c], i) => (
					<div
						key={i}
						style={{
							position: 'absolute',
							left: c * (cellSize + borderSize) + borderSize / 2 - 4,
							top: r * (cellSize + borderSize) + borderSize / 2 - 4,
							width: 8,
							height: 8,
							background: 'black',
							borderRadius: '100%',
							pointerEvents: 'none',
							opacity: (gameFinished || isPassActive) ? 0.3 : 1,
						}}
					/>
				))}

				{/* PASSオーバーレイ */}
				{isPassActive && (
					<div
						style={{
							position: 'absolute',
							top: 0,
							left: 0,
							width: '100%',
							height: '100%',
							display: 'flex',
							alignItems: 'center',
							justifyContent: 'center',
							backgroundColor: 'rgba(0, 0, 0, 0.7)', // 半透明の黒いオーバーレイ
							pointerEvents: 'none', // クリックを無効化
						}}
					>
						<div
							style={{
								color: 'white',
								fontSize: '48px',
								fontWeight: 'bold',
								textShadow: '2px 2px 4px rgba(0, 0, 0, 0.8)',
								letterSpacing: '8px',
							}}
						>
							PASS
						</div>
					</div>
				)}

				{gameEngine.isGameFinished() && (
					<div
						style={{
							position: 'absolute',
							top: 0,
							left: 0,
							width: '100%',
							height: '100%',
							display: 'flex',
							alignItems: 'center',
							justifyContent: 'center',
							backgroundColor: 'rgba(0, 0, 0, 0.7)', // 半透明の黒いオーバーレイ
							pointerEvents: 'none', // クリックを無効化
						}}
					>
						<div
							style={{
								color: 'white',
								fontSize: '48px',
								fontWeight: 'bold',
								textShadow: '2px 2px 4px rgba(0, 0, 0, 0.8)',
								letterSpacing: '8px',
								textAlign: 'left',
							}}
						>
							White: {gameEngine.getWhiteStoneCount()} <br />
							Black: {gameEngine.getBlackStoneCount()} <br />
							<br />
							{gameEngine.getGameState() === 'BlackWin' ? 'Black Win' : 'White Win'}
						</div>
					</div>
				)}
			</div>
		);
	};

	return (
		<div className="App">
			<h1>Reversi</h1>
			<GameInfo gameEngine={gameEngine} passMessage={passMessage} onReset={handleReset} />
			{renderBoard()}
		</div>
	);
}

export default App;
