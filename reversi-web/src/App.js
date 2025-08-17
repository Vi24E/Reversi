/* eslint-env es2020 */
import './App.css';
import React, { useState, useEffect } from 'react';

// WebAssemblyモジュールをグローバル変数として管理
let wasmModule = null;
let wasmInitialized = false;

class InnerBoard {
  constructor(board_str = "...........................WB......BW...........................") {
    this.board_str = board_str;
  }

  // 指定座標(row, col)に黒石があるか
  isBlackStone(row, col) {
    const pos = row * 8 + col;
    return this.board_str[pos] === 'B';
  }

  // 指定座標(row, col)に白石があるか
  isWhiteStone(row, col) {
    const pos = row * 8 + col;
    return this.board_str[pos] === 'W';
  }

  // 0:空, 1:黒石, 2:白石
  getCell(row, col) {
    if (this.isBlackStone(row, col)) return 1;
    if (this.isWhiteStone(row, col)) return 2;
    return 0;
  }

  // 黒石の数を数える
  getBlackStoneCount() {
    return (this.board_str.match(/B/g) || []).length;
  }

  // 白石の数を数える
  getWhiteStoneCount() {
    return (this.board_str.match(/W/g) || []).length;
  }

  checkLegalMove(row, col, turn) {
    try {
      const place = row * 8 + col;
      
      // 既に石がある場合は無効
      if (this.board_str[place] !== '.') return false;
      
      const validMovesStr = wasmModule.get_valid_moves(this.board_str, turn);
      return validMovesStr[place] === '1';
    }
    catch (error) {
      console.error('Error in checkLegalMove:', error);
      return true; // エラーの場合は許可
    }
  }
}

function renderStoneFromBoard(board, row, col, turn) {
  const cell = board.getCell(row, col);
  const placeable = wasmModule.get_valid_moves(board.board_str, turn % 2 !== 0);
  if (cell === 1) {
    return (
      <div
        style={{
          width: 32,
          height: 32,
          borderRadius: '50%',
          background: 'black',
          margin: 'auto',
          marginTop: 4,
        }}
      ></div>
    );
  }
  else if (cell === 2) {
    return (
      <div
        style={{
          width: 32,
          height: 32,
          borderRadius: '50%',
          background: 'white',
          margin: 'auto',
          marginTop: 4,
          border: '1px solid #888',
        }}
      ></div>
    );
  }
  else if (placeable[row * 8 + col] === '1') {
    return (
      <div
        style={{
          width: 32,
          height: 32,
          borderRadius: '50%',
          background: 'gray',
          margin: 'auto',
          marginTop: 4,
        }}
      ></div>
    );
  }
  return null;
}

// WebAssemblyの初期化を行う関数（修正版）
async function initializeWasm() {
  if (wasmInitialized) return true;
  
  try {
    console.log('Initializing WASM module...');
    wasmModule = await import('fl-reversi-rs');
    
    // default関数で初期化（最も一般的な方法）
    if (typeof wasmModule.default === 'function') {
      await wasmModule.default();
      console.log('WASM default initialization completed');
    }
    
    // 初期化確認のテスト
    const initialBoard = "...........................WB......BW...........................";

    console.log('Testing with board:', initialBoard);
    const testMask = wasmModule.get_valid_moves(initialBoard, false);
    console.log('WASM test successful, valid moves:', testMask);
    
    wasmInitialized = true;
    console.log('WASM fully ready');
    return true;
    
  } catch (error) {
    console.error('WASM initialization failed:', error);
    return false;
  }
}

function App() {
  const size = 8;
  // 初期盤面（文字列形式）
  const initialBoard = "...........................WB......BW...........................";

  const [boardStr, setBoardStr] = useState(initialBoard);
  const [turn, setTurn] = useState(0);
  const [wasmLoaded, setWasmLoaded] = useState(false);
  const [wasmError, setWasmError] = useState(null);

  // アプリケーション起動時にWebAssemblyを初期化
  useEffect(() => {
    let isMounted = true;
    
    const loadWasm = async () => {
      const success = await initializeWasm();
      if (isMounted) {
        if (success) {
          setWasmLoaded(true);
          setWasmError(null);
        } else {
          setWasmError('WebAssemblyの読み込みに失敗しました');
        }
      }
    };
    
    loadWasm();
    
    // クリーンアップ
    return () => {
      isMounted = false;
    };
  }, []);

  const board = new InnerBoard(boardStr);
  
  // リセット機能
  const handleReset = () => {
    setBoardStr(initialBoard);
    setTurn(0);
  };

  // マスをクリックしたときの処理
  const handleCellClick = (row, col) => {
    const pos = row * 8 + col;

    // 合法手判定
    if (!board.checkLegalMove(row, col, turn % 2 !== 0)) {
      console.log('Invalid move at', row, col);
      return;
    }

    // WebAssemblyが利用可能かつ関数が存在する場合のみ使用
    if (wasmLoaded && wasmModule && typeof wasmModule.update_board === 'function') {
      try {
        const currentTurn = turn % 2 === 0; // false: 黒, true: 白
        
        console.log('Calling WASM update_board...');
        const updatedBoardStr = wasmModule.update_board(boardStr, pos, currentTurn);
        
        console.log('WASM call successful, result:', updatedBoardStr);
        
        // 新しい盤面状態を設定
        setBoardStr(updatedBoardStr);
        setTurn(turn + 1);
        
        console.log('Move executed via WASM');
      }
      catch (error) {
        console.error('Error during WASM move:', error);
        fallbackMove(row, col);
      }
    }
    else {
      console.log('WASM not available, using fallback');
      fallbackMove(row, col);
    }
  };

  // フォールバック：シンプルな石置き
  const fallbackMove = (row, col) => {
    const pos = row * 8 + col;
    const currentTurn = turn % 2 === 0; // false: 黒, true: 白
    
    let newBoardStr = boardStr.split('');
    newBoardStr[pos] = currentTurn ? 'W' : 'B';
    
    setBoardStr(newBoardStr.join(''));
    setTurn(turn + 1);
    console.log('Move executed via fallback');
  };

  const guidePoints = [
    [2, 2],
    [2, 6],
    [6, 2],
    [6, 6],
  ];
  const cellSize = 40;
  const borderSize = 2;

  // 石数を取得
  const blackStoneCount = board.getBlackStoneCount();
  const whiteStoneCount = board.getWhiteStoneCount();

  // ローディング画面
  if (!wasmLoaded && !wasmError) {
    return (
      <div className="App">
        <h1>Reversi</h1>
        <div style={{ 
          display: 'flex', 
          justifyContent: 'center', 
          alignItems: 'center', 
          height: '200px',
          fontSize: '18px' 
        }}>
          <div>
            <div>⏳ WebAssemblyを読み込み中...</div>
            <div style={{ fontSize: '14px', color: '#666', marginTop: '10px' }}>
              初回起動時は少し時間がかかります
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="App">
      <h1>Reversi</h1>
      
      {/* ゲーム情報表示 */}
      <div>
        <div>現在のプレイヤー: {turn % 2 === 0 ? '黒' : '白'}</div>
        <div>黒石: {blackStoneCount}個</div>
        <div>白石: {whiteStoneCount}個</div>
        <div>ターン数: {turn}</div>
        <div>状態: {
          (() => {
            try {
              if (wasmModule && typeof wasmModule.get_result === 'function') {
                return wasmModule.get_result(boardStr);
              }
              return '読み込み中...';
            } catch (error) {
              console.error('get_result error:', error);
              return 'エラー';
            }
          })()
        }</div>
        <div style={{ color: wasmLoaded ? 'green' : 'red' }}>
          WASM状態: {wasmLoaded ? '✅ 読み込み済み' : wasmError ? '❌ エラー' : '⏳ 読み込み中...'}
        </div>
        {wasmError && (
          <div style={{ color: 'red', fontSize: '12px' }}>
            {wasmError}
          </div>
        )}
        <button onClick={handleReset}>リセット</button>
      </div>
      
      {/* デバッグ用：盤面の文字列表示 */}
      <div style={{ fontSize: '10px', fontFamily: 'monospace', marginBottom: '10px' }}>
        {boardStr.split('').map((char, i) => (
          <span key={i} style={{ color: char === 'B' ? 'black' : char === 'W' ? 'blue' : 'gray' }}>
            {char}
            {(i + 1) % 8 === 0 ? '\n' : ''}
          </span>
        ))}
      </div>
      
      <div style={{ display: 'inline-block', position: 'relative' }}>
        <table style={{ borderCollapse: 'collapse' }}>
          <tbody>
            {[...Array(size)].map((_, row) => (
              <tr key={row}>
                {[...Array(size)].map((_, col) => (
                  <td
                    key={col}
                    style={{
                      width: cellSize,
                      height: cellSize,
                      border: '2px solid black',
                      background: '#0a7d2c',
                      padding: 0,
                      position: 'relative',
                      cursor: 'pointer',
                      opacity: wasmLoaded ? 1 : 0.7,
                    }}
                    onClick={() => handleCellClick(row, col)}
                  >
                    {renderStoneFromBoard(board, row, col, turn)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
        
        {/* 枠線の交点に小さな黒丸 */}
        {guidePoints.map(([r, c], i) => (
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
          ></div>
        ))}
      </div>
    </div>
  );
}

export default App;
