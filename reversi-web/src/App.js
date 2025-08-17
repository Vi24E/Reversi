/* eslint-env es2020 */
import './App.css';
import React, { useState, useEffect } from 'react';

// WebAssemblyモジュールをグローバル変数として管理
let wasmModule = null;
let wasmInitialized = false;

// アプリケーション開始時にWebAssemblyを即座に初期化開始
const wasmPromise = (async () => {
  try {
    console.log('Starting WASM initialization...');
    wasmModule = await import('fl-reversi-rs');
    
    if (typeof wasmModule.default === 'function') {
      await wasmModule.default();
      console.log('WASM default initialization completed');
    }
    
    // 初期化確認のテスト
    const initialBoard = "...........................WB......BW...........................";
    const testMask = wasmModule.get_valid_moves(initialBoard, false);
    console.log('WASM test successful, valid moves:', testMask);
    
    wasmInitialized = true;
    console.log('WASM fully ready');
    return true;
  } catch (error) {
    console.error('WASM initialization failed:', error);
    return false;
  }
})();

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
      return false;
    }
  }
}

function renderStoneFromBoard(board, row, col, turn) {
  const cell = board.getCell(row, col);
  
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
  
  // WebAssemblyが初期化されている場合のみ合法手を表示
  if (wasmInitialized && wasmModule) {
    try {
      const placeable = wasmModule.get_valid_moves(board.board_str, turn % 2 !== 0);
      if (placeable[row * 8 + col] === '1') {
        return (
          <div
            style={{
              width: 8,
              height: 8,
              borderRadius: '50%',
              background: '#999999b1',
              margin: 'auto',
              marginTop: 4,
            }}
          ></div>
        );
      }
    } catch (error) {
      console.warn('Error getting valid moves for display:', error);
    }
  }
  
  return null;
}

function App() {
  const size = 8;
  // 初期盤面（文字列形式）
  const initialBoard = "...........................WB......BW...........................";

  const [boardStr, setBoardStr] = useState(initialBoard);
  const [turn, setTurn] = useState(0);
  const [wasmLoaded, setWasmLoaded] = useState(false);
  const [wasmError, setWasmError] = useState(null);

  // WebAssemblyの初期化完了を待つ
  useEffect(() => {
    let isMounted = true;
    
    wasmPromise.then((success) => {
      if (isMounted) {
        if (success) {
          setWasmLoaded(true);
          setWasmError(null);
        } else {
          setWasmError('WebAssemblyの読み込みに失敗しました');
        }
      }
    });
    
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

    // WebAssemblyが未初期化の場合は何もしない
    if (!wasmLoaded) {
      console.log('WASM not ready yet');
      return;
    }

    // 合法手判定
    if (!board.checkLegalMove(row, col, turn % 2 !== 0)) {
      console.log('Invalid move at', row, col);
      return;
    }

    try {
      const currentTurn = turn % 2 === 0; // false: 黒, true: 白
      
      console.log('Calling WASM update_board...');
      const updatedBoardStr = wasmModule.update_board(boardStr, pos, currentTurn);
      
      console.log('WASM call successful, result:', updatedBoardStr);
      
      setBoardStr(updatedBoardStr);
      let next_turn = turn + 1;
      setTurn(next_turn);

      if (wasmModule.is_pass(updatedBoardStr, next_turn % 2 !== 0)) {
        setTurn(next_turn + 1);
        console.log('Pass detected');
      }

      console.log('Move executed via WASM');
    }
    catch (error) {
      console.error('Error during WASM move:', error);
    }
  };

  const guidePoints = [[2, 2], [2, 6], [6, 2], [6, 6]];
  const cellSize = 40;
  const borderSize = 2;

  // 石数を取得
  const blackStoneCount = board.getBlackStoneCount();
  const whiteStoneCount = board.getWhiteStoneCount();

  // WebAssemblyが読み込まれるまで待機
  if (!wasmLoaded) {
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
            {wasmError && (
              <div style={{ color: 'red', fontSize: '14px', marginTop: '10px' }}>
                {wasmError}
              </div>
            )}
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
        <div>状態: {wasmModule.get_result(boardStr)}</div>
        <div style={{ color: 'green' }}>
          WASM状態: ✅ 読み込み済み
        </div>
        <button onClick={handleReset}>リセット</button>
      </div>
      
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
