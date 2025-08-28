/* eslint-env es2020 */
import './App.css';
import React, {useState, useEffect} from 'react';
import {GameEngine} from './GameEngine';

let wasmModule = null;

// WASM初期化
const wasmPromise = (async () => {
    try {
        console.log('Starting WASM initialization...');
        wasmModule = await import('fl-reversi-rs');

        if (typeof wasmModule.default === 'function') {
            await wasmModule.default();
        }

		wasmModule.initialize();

        console.log('WASM ready');
        return true;
    }
	catch (error) {
        console.error('WASM initialization failed:', error);
        return false;
    }
})();

// 石の描画
function RenderStone({type, isValidMove, isLastMove}) {
    if (type === 1) {
        return (
            <div
                className="BlackStone"
                style={{
					boxShadow: isLastMove ? '6px 4px 4px #033d14ff' : 'none',
                }}
            />
        );
    }
    if (type === 2) {
        return (
            <div
                className="WhiteStone"
                style={{
					boxShadow: isLastMove ? '6px 4px 4px #033d14ff' : 'none',
                }}
            />
        );
    }
    if (isValidMove) {
        return (
            <div className="PlaceableDot" />
        );
    }
    return null;
}

// ゲーム情報
function GameInfo({gameEngine}) {
    const blackCount = gameEngine.getBlackStoneCount();
    const whiteCount = gameEngine.getWhiteStoneCount();
    const currentPlayer = gameEngine.getCurrentTurn() % 2 === 0 ? '●' : '○';

    return (
        <div className="GameInfo">
            <div className="GameInfoBackground">
                {/* 現在の手番 */}
                <span className="GameInfoText"> 
					Next: {currentPlayer}
				</span>

                <div className="GameInfoSeparator"></div>

                {/* 黒石の数 */}
                <span className="GameInfoText">
                    ● {blackCount}
                </span>

                <div className="GameInfoSeparator"></div>

                {/* 白石の数 */}
                <span className="GameInfoText">
                    ○ {whiteCount}
                </span>
            </div>
        </div>
    );
}

// ゲーム操作ボタンコンポーネント
function GameControls({gameEngine, onReset, onUndo, onRedo, onShowMenu, onDownloadLog}) {
    const canUndo = gameEngine.undoable();
    const canRedo = gameEngine.redoable();

    return (
        <div style={{ marginTop: '20px', display: 'flex', gap: '10px', height: '35px', alignItems: 'center', justifyContent: 'center' }}>
            {/* Undo */}
            <button
                onClick={onUndo}
                disabled={!canUndo}
                className={canUndo ? 'ButtonEditEnabled' : 'ButtonEditDisabled'}
            >
                ←
            </button>

            {/* Redo */}
            <button
                onClick={onRedo}
                disabled={!canRedo}
                className={canRedo ? 'ButtonEditEnabled' : 'ButtonEditDisabled'}
            >
                →
            </button>

            {/* RESET */}
            <button
                onClick={onReset}
                className="ButtonReset"
            >
                RESET
            </button>

            {/* メニュー */}
            <button
                onClick={onShowMenu}
                className="ButtonMenu"
            >
                MENU
            </button>

            {/* 棋譜コピー */}
            <button
                onClick={onDownloadLog}
                className="ButtonRecord"
            >
                RECORD
            </button>
        </div>
    );
}

// メインアプリケーション
function App() {
    const [gameEngine, setGameEngine] = useState(null);
    const [wasmLoaded, setWasmLoaded] = useState(false);
    const [wasmError, setWasmError] = useState(null);
    const [passMessage, setPassMessage] = useState('');
    const [, forceUpdate] = useState({}); // 強制再描画
    const [showMenu, setShowMenu] = useState(true); // 初回はメニュー表示
    const [isAiThinking, setIsAiThinking] = useState(false); // AI思考中フラグ
    const [isEditing, setIsEditing] = useState(false); // 編集中フラグ
    const [blackAiLevel, setBlackAiLevel] = useState(5); // 黒AIレベル (1-10)
    const [whiteAiLevel, setWhiteAiLevel] = useState(5); // 白AIレベル (1-10)

    useEffect(() => {
        let isMounted = true;

        wasmPromise.then(success => {
            if (isMounted) {
                if (success) {
                    const engine = new GameEngine(wasmModule);
                    engine.setPlayerMode('human', 'human');
                    setGameEngine(engine);
                    setWasmLoaded(true);
                    setWasmError(null);
                }
				else {
                    setWasmError('WebAssemblyの読み込みに失敗しました');
                }
            }
        });

        return () => {
            isMounted = false;
        };
    }, []);

    // AIの手番を監視
    useEffect(() => {
        if (!gameEngine || showMenu || gameEngine.isGameFinished() || passMessage !== '' || isAiThinking || isEditing) {
            return;
        }

        const currentPlayerType = gameEngine.getCurrentPlayerType();
        
        if (!isAiThinking && currentPlayerType === 'ai') {
            setIsAiThinking(true);
            
            const aiMoveTimer = setTimeout(() => {
                try {
					const level = gameEngine.getCurrentPlayer() ? whiteAiLevel : blackAiLevel;
                    const result = gameEngine.getAIMove(1000, level);
					let row = Math.floor(result / 8);
					let col = result % 8;
					console.log(`AI recommends move: (${row}, ${col})`);
					gameEngine.makeMove(row, col);

                    forceUpdate({});
                }
				catch (error) {
                    console.error('AI move failed:', error);
                } 
				finally {
                    setIsAiThinking(false);
                }
            }, 1200);

            return () => {
                clearTimeout(aiMoveTimer);
                setIsAiThinking(false);
            };
        }
    }, [gameEngine, showMenu, passMessage, gameEngine?.getCurrentTurn(), isEditing]);

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
                        <div>⏳ Waiting for initializing WASM...</div>
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

    // メニューキャンセル
    const handleMenuCancel = () => {
        setShowMenu(false);
    };

    // マスをクリックしたときの処理
    const handleCellClick = (row, col) => {
		if (isEditing) {
			setIsEditing(false);
		}

        // メニュー表示中またはAI思考中はクリック無効
        if (showMenu || isAiThinking) {
            return;
        }

        // ゲーム終了時またはパス中はクリック無効
        if (gameEngine.isGameFinished() || passMessage !== '') {
            return;
        }

        // AIのターンの場合はクリック無効
        if (gameEngine.getCurrentPlayerType() !== 'human') {
            return;
        }

        const result = gameEngine.makeMove(row, col);

        if (gameEngine.isGameFinished()) {
            forceUpdate({});
            return;
        }

        if (result.success) {
            if (result.passMessage) {
                setPassMessage(result.passMessage);
                setTimeout(() => setPassMessage(''), 700);
            } else {
                setPassMessage('');
            }
            forceUpdate({});
        }
    };

    // リセット処理
    const handleReset = () => {
        setIsAiThinking(false);
		setIsEditing(false);
        gameEngine.reset();
        setPassMessage('');
        forceUpdate({});
    };

    // Undo処理
    const handleUndo = () => {
		setIsEditing(true);
        if (gameEngine.undoable() && !isAiThinking) {
            setIsAiThinking(false);
            gameEngine.undoBoard();
            setPassMessage('');
            forceUpdate({});
        }
    };

    // Redo処理
    const handleRedo = () => {
		setIsEditing(true);
        if (gameEngine.redoable() && !isAiThinking) {
            setIsAiThinking(false);
            gameEngine.redoBoard();
            setPassMessage('');
            forceUpdate({});
        }
    };

    // メニュー表示
    const handleShowMenu = () => {
        setShowMenu(true);
    };

    // メニューオーバーレイ
    const renderMenu = () => (
        <div className="MenuOverlay">
            <div className="MenuBackground">

                {/* プレイヤー設定セクション */}
                <div style={{ marginBottom: '25px' }}>
                    <h4 className="MenuTitle">
                        Settings
                    </h4>
                    
                    {/* 黒プレイヤー設定 */}
                    <div style={{ marginBottom: '20px' }}>
                        <div style={{
                            display: 'flex',
                            alignItems: 'center',
                            marginBottom: '15px',
                            gap: '15px'
                        }}>
                            <div className="MenuBlackStone"></div>
                            <div className="MenuPlayerButtonEdge">
                                <button
                                    onClick={() => {
                                        const newBlackMode = 'human';
                                        gameEngine.setPlayerMode(newBlackMode, gameEngine.playerModes.white);
                                        forceUpdate({});
                                    }}
                                    className={gameEngine.playerModes.black === 'human' ? 'MenuPlayerButtonEnabled' : 'MenuPlayerButtonDisabled'}
                                >
                                    Human
                                </button>
                                <button
                                    onClick={() => {
                                        const newBlackMode = 'ai';
                                        gameEngine.setPlayerMode(newBlackMode, gameEngine.playerModes.white);
                                        forceUpdate({});
                                    }}
                                    className={gameEngine.playerModes.black === 'ai' ? 'MenuPlayerButtonEnabled' : 'MenuPlayerButtonDisabled'}
                                >
                                    AI
                                </button>
                            </div>
                        </div>

                        {/* 黒AI レベル設定 */}
                        <div className={gameEngine.playerModes.black === 'ai' ? 'MenuLevelEnabled' : 'MenuLevelDisabled'}>
                            <div className="MenuLevelText">
                                <span>AI Level:</span>
                                <span className="MenuLevelValue">
                                    {blackAiLevel}
                                </span>
                            </div>
                            <input
                                type="range"
                                min="1"
                                max="10"
                                step="1"
                                value={blackAiLevel}
                                onChange={(e) => setBlackAiLevel(parseInt(e.target.value))}
                                disabled={gameEngine.playerModes.black !== 'ai'}
                                className={gameEngine.playerModes.black === 'ai' ? 'MenuSliderEnabled' : 'MenuSliderDisabled'}
                            />
                            <div className="MenuSubtext">
                                <span>Weak</span>
                                <span>Strong</span>
                            </div>
                        </div>
                    </div>

                    {/* 白プレイヤー設定 */}
                    <div style={{ marginBottom: '20px' }}>
                        <div style={{
                            display: 'flex',
                            alignItems: 'center',
                            marginBottom: '15px',
                            gap: '15px'
                        }}>
                            <div className="MenuWhiteStone"></div>
                            <div className="MenuPlayerButtonEdge">
                                <button
                                    onClick={() => {
                                        const newWhiteMode = 'human';
                                        gameEngine.setPlayerMode(gameEngine.playerModes.black, newWhiteMode);
                                        forceUpdate({});
                                    }}
                                    className={gameEngine.playerModes.white === 'human' ? 'MenuPlayerButtonEnabled' : 'MenuPlayerButtonDisabled'}
                                >
                                    Human
                                </button>
                                <button
                                    onClick={() => {
                                        const newWhiteMode = 'ai';
                                        gameEngine.setPlayerMode(gameEngine.playerModes.black, newWhiteMode);
                                        forceUpdate({});
                                    }}
                                    className={gameEngine.playerModes.white === 'ai' ? 'MenuPlayerButtonEnabled' : 'MenuPlayerButtonDisabled'}
                                >
                                    AI
                                </button>
                            </div>
                        </div>

                        {/* 白AI レベル設定 */}
                        <div className={gameEngine.playerModes.white === 'ai' ? 'MenuLevelEnabled' : 'MenuLevelDisabled'}>
                            <div className="MenuLevelText">
                                <span>AI Level:</span>
                                <span className="MenuLevelValue">
                                    {whiteAiLevel}
                                </span>
                            </div>
                            <input
                                type="range"
                                min="1"
                                max="10"
                                step="1"
                                value={whiteAiLevel}
                                onChange={(e) => setWhiteAiLevel(parseInt(e.target.value))}
                                disabled={gameEngine.playerModes.white !== 'ai'}
                                className={gameEngine.playerModes.white === 'ai' ? 'MenuSliderEnabled' : 'MenuSliderDisabled'}
                            />
                            <div className="MenuSubtext">
                                <span>Weak</span>
                                <span>Strong</span>
                            </div>
                        </div>
                    </div>
                </div>

                {/* ボタン */}
                <div className="MenuTailButton">
                    <button
                        onClick={() => {
                            gameEngine.reset();
                            setPassMessage('');
                            setShowMenu(false);
							setIsEditing(false);
                            forceUpdate({});
                        }}
                        className="MenuStartButton"
                    >
                        Start
                    </button>
                    <button
                        onClick={handleMenuCancel}
                        className="MenuCloseButton"
                    >
                        Close
                    </button>
                </div>
            </div>
        </div>
    );

	// 棋譜ダウンロード機能
    const handleDownloadLog = () => {
        let result = gameEngine.getKif();
		if (result) {
			navigator.clipboard.writeText(result).then(() => {
				console.log('棋譜がクリップボードにコピーされました');
			}).catch(err => {
				console.error('クリップボードへのコピーに失敗しました:', err);
			});
		}
    };

    // ボードの描画
    const renderBoard = () => {
        const size = 8;
        const cellSize = 40;
        const borderSize = 2;
        const gameFinished = gameEngine.isGameFinished();
        const validMovesStr = gameEngine.getValidMoves();
        const isPassActive = passMessage !== '';
        const currentPlayerType = gameEngine.getCurrentPlayerType();

        return (
            <div style={{display: 'inline-block', position: 'relative'}}>
                <table style={{borderCollapse: 'collapse'}}>
                    <tbody>
                        {[...Array(size)].map((_, row) => (
                            <tr key={row}>
                                {[...Array(size)].map((_, col) => {
                                    const cellType = gameEngine.getCell(row, col);
                                    const isValidMove = !gameFinished && !isPassActive && !showMenu && !isAiThinking && currentPlayerType === 'human' && validMovesStr[row * 8 + col] === '1';
									const isLastMove = (gameEngine.getLastMove() === row * 8 + col);

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
                                                cursor: (gameFinished || isPassActive || showMenu) ? 'not-allowed' : (isAiThinking ? 'wait' : 'pointer')
                                            }}
                                            onClick={() => handleCellClick(row, col)}
                                        >
                                            <RenderStone type={cellType} isValidMove={isValidMove} isLastMove={isLastMove} />
                                        </td>
                                    );
                                })}
                            </tr>
                        ))}
                    </tbody>
                </table>

                {/* ガイドポイント */}
                {[
                    [2, 2], [2, 6], [6, 2], [6, 6],
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
                        }}
                    />
                ))}

                {/* AI思考中オーバーレイ */}
                {isAiThinking && (
                    <div className="AIThinkingOverlay">
                        <div className="AIThinkingMessage">
                            Thinking...
                        </div>
                    </div>
                )}

                {/* PASSオーバーレイ */}
                {passMessage && (
                    <div className="PassOverlay">
                        <div className="Pass">
                            PASS
                        </div>
                    </div>
                )}

                {/* メニューオーバーレイ */}
                {showMenu && renderMenu()}

                {/* ゲーム終了オーバーレイ */}
                {gameFinished && !showMenu && (
                    <div className="GameFinishedOverlay">
                        <div className="GameFinishedMessage">
                            White: {gameEngine.getWhiteStoneCount()} <br />
                            Black: {gameEngine.getBlackStoneCount()} <br />
                            <br />
                            {gameEngine.getGameState() === 'BlackWin' ? 'Black Win' : 
                                gameEngine.getGameState() === 'WhiteWin' ? 'White Win' : 'Draw'}
                        </div>
                    </div>
                )}
            </div>
        );
    };

	return (
		<div className="App">
			<h1>Reversi</h1>
			
			{/* オセロボード */}
			<div style={{ marginBottom: '2px' }}>
				{renderBoard()}
			</div>

			{/* ゲーム情報*/}
			<div style={{ marginBottom: '-15px' }}>
				<GameInfo 
					gameEngine={gameEngine} 
				/>
			</div>

			{/* ゲーム操作ボタン */}
			<div style={{ marginBottom: '3px' }}>
				<GameControls
					gameEngine={gameEngine}
					onReset={handleReset}
					onUndo={handleUndo}
					onRedo={handleRedo}
					onShowMenu={handleShowMenu}
					onDownloadLog={handleDownloadLog}
				/>
			</div>
		</div>
	);
}

export default App;