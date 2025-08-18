/* eslint-env es2020 */
import './App.css';
import React, {useState, useEffect} from 'react';
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
        wasmModule.get_valid_moves(testBoard, false);

        console.log('WASM fully ready');
        return true;
    } catch (error) {
        console.error('WASM initialization failed:', error);
        return false;
    }
})();

// 石を描画するコンポーネント
function Stone({type, isValidMove}) {
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
function GameInfo({gameEngine, passMessage}) {
    const blackCount = gameEngine.getBlackStoneCount();
    const whiteCount = gameEngine.getWhiteStoneCount();
    const currentPlayer = gameEngine.getCurrentTurn() % 2 === 0 ? '黒' : '白';
    const currentPlayerType = gameEngine.getCurrentPlayerType();
    const gameState = gameEngine.getGameState();

    return (
        <div style={{ marginBottom: '15px' }}>
            <div>現在のプレイヤー: {currentPlayer} ({currentPlayerType === 'human' ? '人間' : 'AI'})</div>
            <div>黒石: {blackCount}個</div>
            <div>白石: {whiteCount}個</div>
            <div>ターン数: {gameEngine.getCurrentTurn()}</div>
            <div>状態: {gameState}</div>

            {passMessage && (
                <div style={{
                    color: 'orange',
                    fontSize: '18px',
                    fontWeight: 'bold',
                    padding: '10px',
                    backgroundColor: '#fff3cd',
                    border: '1px solid #ffeaa7',
                    borderRadius: '5px',
                    marginTop: '10px',
                    textAlign: 'center'
                }}>
                    {passMessage}
                </div>
            )}
        </div>
    );
}

// ゲーム操作ボタンコンポーネント
function GameControls({gameEngine, onReset, onUndo, onRedo, onShowMenu}) {
    const canUndo = gameEngine.undoable();
    const canRedo = gameEngine.redoable();

    return (
        <div style={{ marginTop: '20px', display: 'flex', gap: '10px', alignItems: 'center', justifyContent: 'center' }}>
            {/* Undoボタン */}
            <button
                onClick={onUndo}
                disabled={!canUndo}
                style={{
                    padding: '8px 12px',
                    fontSize: '16px',
                    backgroundColor: canUndo ? '#007bff' : '#6c757d',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: canUndo ? 'pointer' : 'not-allowed',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '5px'
                }}
                title="元に戻す"
            >
                ←
            </button>

            {/* Redoボタン */}
            <button
                onClick={onRedo}
                disabled={!canRedo}
                style={{
                    padding: '8px 12px',
                    fontSize: '16px',
                    backgroundColor: canRedo ? '#007bff' : '#6c757d',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: canRedo ? 'pointer' : 'not-allowed',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '5px'
                }}
                title="やり直し"
            >
                →
            </button>

            {/* リセットボタン */}
            <button
                onClick={onReset}
                style={{
                    padding: '8px 16px',
                    fontSize: '14px',
                    backgroundColor: '#28a745',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer'
                }}
            >
                リセット
            </button>

            {/* メニューボタン */}
            <button
                onClick={onShowMenu}
                style={{
                    padding: '8px 16px',
                    fontSize: '14px',
                    backgroundColor: '#6c757d',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer'
                }}
            >
                メニュー
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
    const [, forceUpdate] = useState({}); // 強制再描画用
    const [showMenu, setShowMenu] = useState(true); // 初回はメニュー表示

    // WebAssemblyの初期化を待つ
    useEffect(() => {
        let isMounted = true;

        wasmPromise.then(success => {
            if (isMounted) {
                if (success) {
                    // GameEngineを初期化
                    const engine = new GameEngine(wasmModule);
                    engine.setPlayerMode('human', 'human'); // デフォルト設定
                    setGameEngine(engine);
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
    }, []);

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

    // メニューキャンセル
    const handleMenuCancel = () => {
        setShowMenu(false);
    };

    // マスをクリックしたときの処理
    const handleCellClick = (row, col) => {
        // メニュー表示中はクリック無効
        if (showMenu) {
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
        gameEngine.reset();
        setPassMessage('');
        forceUpdate({});
    };

    // Undo処理
    const handleUndo = () => {
        if (gameEngine.undoable()) {
            gameEngine.undoBoard();
            setPassMessage('');
            forceUpdate({});
        }
    };

    // Redo処理
    const handleRedo = () => {
        if (gameEngine.redoable()) {
            gameEngine.redoBoard();
            setPassMessage('');
            forceUpdate({});
        }
    };

    // メニュー表示
    const handleShowMenu = () => {
        setShowMenu(true);
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
                                    const isValidMove = !gameFinished && !isPassActive && !showMenu && currentPlayerType === 'human' && validMovesStr[row * 8 + col] === '1';

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
                                                cursor: (gameFinished || isPassActive || showMenu || currentPlayerType !== 'human') ? 'not-allowed' : 'pointer',
                                                opacity: (gameFinished || isPassActive || showMenu) ? 0.3 : 1,
                                                filter: (gameFinished || isPassActive || showMenu) ? 'grayscale(30%)' : 'none',
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
                            opacity: (gameFinished || isPassActive || showMenu) ? 0.3 : 1,
                        }}
                    />
                ))}

                {/* PASSオーバーレイ */}
                {passMessage && (
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
                            backgroundColor: 'rgba(0, 0, 0, 0.7)',
                            pointerEvents: 'none',
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

                {/* メニューオーバーレイ */}
                {showMenu && (
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
                            backgroundColor: 'rgba(0, 0, 0, 0.7)',
                            pointerEvents: 'none',
                        }}
                    >
                        <div
                            style={{
                                backgroundColor: '#1a1a1a',
                                border: '2px solid #666',
                                borderRadius: '10px',
                                padding: '25px',
                                color: 'white',
                                boxShadow: '0 4px 16px rgba(0, 0, 0, 0.8)',
                                pointerEvents: 'auto',
                                minWidth: '300px'
                            }}
                        >
                            <h3 style={{ margin: '0 0 20px 0', fontSize: '20px', textAlign: 'center' }}>
                                プレイヤー設定
                            </h3>

                            {/* 設定セクション */}
                            <div style={{ marginBottom: '20px' }}>
                                {/* 黒プレイヤー設定 */}
                                <div style={{
                                    display: 'flex',
                                    alignItems: 'center',
                                    marginBottom: '15px',
                                    gap: '15px'
                                }}>
                                    <div style={{
                                        width: 24,
                                        height: 24,
                                        borderRadius: '50%',
                                        background: 'black',
                                        border: '1px solid #999'
                                    }}></div>
                                    <span style={{ fontSize: '16px', minWidth: '40px' }}>黒:</span>
                                    <div style={{
                                        display: 'flex',
                                        border: '1px solid #555',
                                        borderRadius: '15px',
                                        overflow: 'hidden',
                                        marginLeft: 'auto'
                                    }}>
                                        <button
                                            onClick={() => {
                                                const newBlackMode = 'human';
                                                gameEngine.setPlayerMode(newBlackMode, gameEngine.playerModes.white);
                                                forceUpdate({});
                                            }}
                                            style={{
                                                padding: '6px 12px',
                                                border: 'none',
                                                backgroundColor: gameEngine.playerModes.black === 'human' ? '#007bff' : '#333',
                                                color: gameEngine.playerModes.black === 'human' ? '#fff' : '#ccc',
                                                cursor: 'pointer',
                                                fontSize: '14px'
                                            }}
                                        >
                                            人間
                                        </button>
                                        <button
                                            onClick={() => {
                                                const newBlackMode = 'ai';
                                                gameEngine.setPlayerMode(newBlackMode, gameEngine.playerModes.white);
                                                forceUpdate({});
                                            }}
                                            style={{
                                                padding: '6px 12px',
                                                border: 'none',
                                                backgroundColor: gameEngine.playerModes.black === 'ai' ? '#007bff' : '#333',
                                                color: gameEngine.playerModes.black === 'ai' ? '#fff' : '#ccc',
                                                cursor: 'pointer',
                                                fontSize: '14px'
                                            }}
                                        >
                                            AI
                                        </button>
                                    </div>
                                </div>

                                {/* 白プレイヤー設定 */}
                                <div style={{
                                    display: 'flex',
                                    alignItems: 'center',
                                    gap: '15px'
                                }}>
                                    <div style={{
                                        width: 24,
                                        height: 24,
                                        borderRadius: '50%',
                                        background: 'white',
                                        border: '1px solid #666'
                                    }}></div>
                                    <span style={{ fontSize: '16px', minWidth: '40px' }}>白:</span>
                                    <div style={{
                                        display: 'flex',
                                        border: '1px solid #555',
                                        borderRadius: '15px',
                                        overflow: 'hidden',
                                        marginLeft: 'auto'
                                    }}>
                                        <button
                                            onClick={() => {
                                                const newWhiteMode = 'human';
                                                gameEngine.setPlayerMode(gameEngine.playerModes.black, newWhiteMode);
                                                forceUpdate({});
                                            }}
                                            style={{
                                                padding: '6px 12px',
                                                border: 'none',
                                                backgroundColor: gameEngine.playerModes.white === 'human' ? '#007bff' : '#333',
                                                color: gameEngine.playerModes.white === 'human' ? '#fff' : '#ccc',
                                                cursor: 'pointer',
                                                fontSize: '14px'
                                            }}
                                        >
                                            人間
                                        </button>
                                        <button
                                            onClick={() => {
                                                const newWhiteMode = 'ai';
                                                gameEngine.setPlayerMode(gameEngine.playerModes.black, newWhiteMode);
                                                forceUpdate({});
                                            }}
                                            style={{
                                                padding: '6px 12px',
                                                border: 'none',
                                                backgroundColor: gameEngine.playerModes.white === 'ai' ? '#007bff' : '#333',
                                                color: gameEngine.playerModes.white === 'ai' ? '#fff' : '#ccc',
                                                cursor: 'pointer',
                                                fontSize: '14px'
                                            }}
                                        >
                                            AI
                                        </button>
                                    </div>
                                </div>
                            </div>

                            {/* ボタン */}
                            <div style={{
                                display: 'flex',
                                gap: '10px',
                                justifyContent: 'center'
                            }}>
                                <button
                                    onClick={() => {
                                        gameEngine.reset();
                                        setPassMessage('');
                                        setShowMenu(false);
                                        forceUpdate({});
                                    }}
                                    style={{
                                        padding: '10px 20px',
                                        fontSize: '14px',
                                        fontWeight: 'bold',
                                        backgroundColor: '#28a745',
                                        color: 'white',
                                        border: 'none',
                                        borderRadius: '6px',
                                        cursor: 'pointer'
                                    }}
                                >
                                    Start
                                </button>
                                <button
                                    onClick={handleMenuCancel}
                                    style={{
                                        padding: '10px 20px',
                                        fontSize: '14px',
                                        fontWeight: 'bold',
                                        backgroundColor: '#6c757d',
                                        color: 'white',
                                        border: 'none',
                                        borderRadius: '6px',
                                        cursor: 'pointer'
                                    }}
                                >
                                    Close
                                </button>
                            </div>
                        </div>
                    </div>
                )}

                {gameFinished && (
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
                            backgroundColor: 'rgba(0, 0, 0, 0.7)',
                            pointerEvents: 'none',
                        }}
                    >
                        <div
                            style={{
                                color: 'white',
                                fontSize: '48px',
                                fontWeight: 'bold',
                                textShadow: '2px 2px 4px rgba(0, 0, 0, 0.8)',
                                letterSpacing: '8px',
                                textAlign: 'center',
                            }}
                        >
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
            
            {/* ゲーム情報（ボードの上） */}
            <GameInfo 
                gameEngine={gameEngine} 
                passMessage={passMessage}
            />
            
            {/* オセロボード */}
            {renderBoard()}
            
            {/* ゲーム操作ボタン（ボードの下） */}
            <GameControls
                gameEngine={gameEngine}
                onReset={handleReset}
                onUndo={handleUndo}
                onRedo={handleRedo}
                onShowMenu={handleShowMenu}
            />
        </div>
    );
}

export default App;
