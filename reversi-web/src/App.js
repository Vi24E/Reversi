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

		wasmModule.initialize();

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
function Stone({type, isValidMove, isLastMove}) {
    if (type === 1) {
        return (
            <div
                style={{
                    width: 32,
                    height: 32,
                    borderRadius: '50%',
                    background: 'black',
                    margin: 'auto',
					boxShadow: isLastMove ? '6px 4px 4px #033d14ff' : 'none',
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
					boxShadow: isLastMove ? '6px 4px 4px #033d14ff' : 'none',
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
function GameInfo({gameEngine}) {
    const blackCount = gameEngine.getBlackStoneCount();
    const whiteCount = gameEngine.getWhiteStoneCount();
    const currentPlayer = gameEngine.getCurrentTurn() % 2 === 0 ? '●' : '○';

    return (
        <div style={{ 
            width: '100%',
            maxWidth: '320px', // ボードと同じ幅に合わせる
            margin: '0 auto 15px auto', // 中央寄せ
        }}>
            <div style={{
                background: '#e0f1ff',
                color: 'black',
                padding: '8px 16px',
                fontSize: '16px',
                fontWeight: 'bold',
                textAlign: 'center',
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                gap: '20px',
                border: '2px solid #000000',
                borderRadius: '0', // リボン風にするため角を丸くしない
            }}>
                {/* 現在の手番 */}
                <span>Next: {currentPlayer}</span>
                
                {/* 区切り線 */}
                <div style={{
                    width: '2px',
                    height: '20px',
                    background: '#000000',
                    opacity: 0.7
                }}></div>
                
                {/* 黒石の数 */}
                <span style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                    ● {blackCount}
                </span>
                
                {/* 区切り線 */}
                <div style={{
                    width: '2px',
                    height: '20px',
                    background: '#000000',
                    opacity: 0.7
                }}></div>
                
                {/* 白石の数 */}
                <span style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                    ○ {whiteCount}
                </span>
            </div>
        </div>
    );
}

// ゲーム操作ボタンコンポーネント
function GameControls({gameEngine, onReset, onUndo, onRedo, onShowMenu}) {
    const canUndo = gameEngine.undoable();
    const canRedo = gameEngine.redoable();

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

    return (
        <div style={{ marginTop: '20px', display: 'flex', gap: '10px', alignItems: 'center', justifyContent: 'center' }}>
            {/* Undoボタン */}
            <button
                onClick={onUndo}
                disabled={!canUndo}
                style={{
                    padding: '8px 12px',
                    fontSize: '16px',
                    backgroundColor: canUndo ? '#17a2b8' : '#6c757d',
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
                    backgroundColor: canRedo ? '#17a2b8' : '#6c757d',
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

            {/* ログ管理ボタン */}
            <button
                onClick={handleDownloadLog}
                style={{
                    padding: '8px 16px',
                    fontSize: '14px',
                    backgroundColor: '#17a2b8',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer'
                }}
                title="ログをダウンロード"
            >
                📥 棋譜
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
    const [isAiThinking, setIsAiThinking] = useState(false); // AI思考中フラグ
    const [isEditing, setIsEditing] = useState(false); // 編集中フラグ
    
    // 新しい設定用のstate
    const [blackAiLevel, setBlackAiLevel] = useState(5); // 黒AIレベル (1-10)
    const [whiteAiLevel, setWhiteAiLevel] = useState(5); // 白AIレベル (1-10)

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

    // AIの手番を監視して自動で手を打つ
    useEffect(() => {
        if (!gameEngine || showMenu || gameEngine.isGameFinished() || passMessage !== '' || isAiThinking || isEditing) {
            return;
        }

		console.log('AIの手番を監視中...');

        const currentPlayerType = gameEngine.getCurrentPlayerType();
        
        if (!isAiThinking && currentPlayerType === 'ai') {
            // AIの番の場合、少し遅延をつけてからAIに手を打たせる
            setIsAiThinking(true);
            
            const aiMoveTimer = setTimeout(() => {
                try {
                    const result = gameEngine.getAIMove(1000); // GameEngineにAIの手を打つメソッドが必要
					let row = Math.floor(result / 8);
					let col = result % 8;
					console.log(`AI recommends move: (${row}, ${col})`);
					gameEngine.makeMove(row, col);

                    forceUpdate({});
                } catch (error) {
                    console.error('AI move failed:', error);
                } finally {
                    setIsAiThinking(false);
                }
            }, 1200); // 1秒の思考時間

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

    // メニューオーバーレイ部分を修正
    const renderMenu = () => (
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
                    minWidth: '350px',
                    maxHeight: '80vh',
                    overflowY: 'auto'
                }}
            >

                {/* プレイヤー設定セクション */}
                <div style={{ marginBottom: '25px' }}>
                    <h4 style={{ margin: '0 0 15px 0', fontSize: '16px', borderBottom: '1px solid #555', paddingBottom: '8px' }}>
                        プレイヤー設定
                    </h4>
                    
                    {/* 黒プレイヤー設定 */}
                    <div style={{ marginBottom: '20px' }}>
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

                        {/* 黒AI レベル設定 */}
                        <div style={{ 
                            marginLeft: '39px',
                            opacity: gameEngine.playerModes.black === 'ai' ? 1 : 0.4,
                            pointerEvents: gameEngine.playerModes.black === 'ai' ? 'auto' : 'none',
                            transition: 'opacity 0.3s ease'
                        }}>
                            <div style={{ 
                                fontSize: '14px', 
                                marginBottom: '8px',
                                display: 'flex',
                                justifyContent: 'space-between',
                                alignItems: 'center'
                            }}>
                                <span>AI Level:</span>
                                <span style={{ 
                                    backgroundColor: '#333', 
                                    padding: '2px 8px', 
                                    borderRadius: '10px',
                                    fontSize: '12px',
                                    minWidth: '25px',
                                    textAlign: 'center'
                                }}>
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
                                style={{
                                    width: '100%',
                                    height: '4px',
                                    borderRadius: '2px',
                                    background: gameEngine.playerModes.black === 'ai' ? '#555' : '#333',
                                    outline: 'none',
                                    cursor: gameEngine.playerModes.black === 'ai' ? 'pointer' : 'not-allowed'
                                }}
                            />
                            <div style={{
                                display: 'flex',
                                justifyContent: 'space-between',
                                fontSize: '10px',
                                color: '#999',
                                marginTop: '3px'
                            }}>
                                <span>弱</span>
                                <span>強</span>
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
                            <div style={{
                                width: 24,
                                height: 24,
                                borderRadius: '50%',
                                background: 'white',
                                border: '1px solid #666'
                            }}></div>
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

                        {/* 白AI レベル設定 */}
                        <div style={{ 
                            marginLeft: '39px',
                            opacity: gameEngine.playerModes.white === 'ai' ? 1 : 0.4,
                            pointerEvents: gameEngine.playerModes.white === 'ai' ? 'auto' : 'none',
                            transition: 'opacity 0.3s ease'
                        }}>
                            <div style={{ 
                                fontSize: '14px', 
                                marginBottom: '8px',
                                display: 'flex',
                                justifyContent: 'space-between',
                                alignItems: 'center'
                            }}>
                                <span>AI Level:</span>
                                <span style={{ 
                                    backgroundColor: '#333', 
                                    padding: '2px 8px', 
                                    borderRadius: '10px',
                                    fontSize: '12px',
                                    minWidth: '25px',
                                    textAlign: 'center'
                                }}>
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
                                style={{
                                    width: '100%',
                                    height: '4px',
                                    borderRadius: '2px',
                                    background: gameEngine.playerModes.white === 'ai' ? '#555' : '#333',
                                    outline: 'none',
                                    cursor: gameEngine.playerModes.white === 'ai' ? 'pointer' : 'not-allowed'
                                }}
                            />
                            <div style={{
                                display: 'flex',
                                justifyContent: 'space-between',
                                fontSize: '10px',
                                color: '#999',
                                marginTop: '3px'
                            }}>
                                <span>弱</span>
                                <span>強</span>
                            </div>
                        </div>
                    </div>
                </div>

                {/* ボタン */}
                <div style={{
                    display: 'flex',
                    gap: '10px',
                    justifyContent: 'center',
                    paddingTop: '15px',
                    borderTop: '1px solid #555'
                }}>
                    <button
                        onClick={() => {
                            console.log('Game Settings Applied:', {
                                boardSize,
                                blackAiLevel: gameEngine.playerModes.black === 'ai' ? blackAiLevel : 'N/A',
                                whiteAiLevel: gameEngine.playerModes.white === 'ai' ? whiteAiLevel : 'N/A'
                            });
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
    );

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
                                                cursor: (gameFinished || isPassActive || showMenu || isAiThinking || currentPlayerType !== 'human') ? 'not-allowed' : 'pointer',
                                                opacity: (gameFinished || isPassActive || showMenu || isAiThinking) ? 0.3 : 1,
                                                filter: (gameFinished || isPassActive || showMenu || isAiThinking) ? 'grayscale(30%)' : 'none',
                                            }}
                                            onClick={() => handleCellClick(row, col)}
                                        >
                                            <Stone type={cellType} isValidMove={isValidMove} isLastMove={isLastMove} />
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
                            opacity: (gameFinished || isPassActive || showMenu || isAiThinking) ? 0.3 : 1,
                        }}
                    />
                ))}

                {/* AI思考中オーバーレイ */}
                {isAiThinking && (
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
                            backgroundColor: 'rgba(0, 0, 0, 0.5)',
                            pointerEvents: 'none',
                        }}
                    >
                        <div
                            style={{
                                color: 'white',
                                fontSize: '32px',
                                fontWeight: 'bold',
                                textShadow: '2px 2px 4px rgba(0, 0, 0, 0.8)',
                                letterSpacing: '4px',
                                animation: 'pulse 1.5s infinite'
                            }}
                        >
                            Thinking...
                        </div>
                    </div>
                )}

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
                {showMenu && renderMenu()}

                {/* ゲーム終了オーバーレイ */}
                {gameFinished && !showMenu && (
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
				/>
			</div>
		</div>
	);
}

export default App;

/*

*/