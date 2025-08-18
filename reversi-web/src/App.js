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
    const gameState = gameEngine.getGameState();

    return (
        <div style={{ marginBottom: '15px' }}>
            <div>現在のプレイヤー: {currentPlayer}</div>
            <div>黒石: {blackCount}個</div>
            <div>白石: {whiteCount}個</div>
            <div>ターン数: {gameEngine.getCurrentTurn()}</div>
            <div>状態: {gameState}</div>
        </div>
    );
}

// ゲーム操作ボタンコンポーネント
function GameControls({gameEngine, onReset, onUndo, onRedo}) {
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
        gameEngine.setPlayerMode('human', 'human');
        setPassMessage('');
        forceUpdate({});
    };

    // Undo処理
    const handleUndo = () => {
        if (gameEngine.undoable()) {
            gameEngine.undoBoard();
            setPassMessage(''); // パスメッセージをクリア
            forceUpdate({});
        }
    };

    // Redo処理
    const handleRedo = () => {
        if (gameEngine.redoable()) {
            gameEngine.redoBoard();
            setPassMessage(''); // パスメッセージをクリア
            forceUpdate({});
        }
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
                                                cursor: (gameFinished || isPassActive || gameEngine.getCurrentPlayerType() !== 'human') ? 'not-allowed' : 'pointer',
                                                opacity: (gameFinished || isPassActive) ? 0.3 : 1,
                                                filter: (gameFinished || isPassActive) ? 'grayscale(30%)' : 'none',
                                            }}
                                            onClick={() => handleCellClick(row, col)}
                                        >
                                            <Stone type={cellType} isValidMove={isValidMove && gameEngine.getCurrentPlayerType() == 'human'} />
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
            />
        </div>
    );
}

export default App;
