import './App.css';
import React, { useState } from 'react';

class InnerBoard {
  constructor(my_board = 0n, opponent_board = 0n) {
    this.my_board = BigInt(my_board);
    this.opponent_board = BigInt(opponent_board);
  }

  // 指定座標(row, col)に自分の石があるか
  isMyStone(row, col) {
    const pos = BigInt(row * 8 + col);
    return ((this.my_board >> pos) & 1n) === 1n;
  }

  // 指定座標(row, col)に相手の石があるか
  isOpponentStone(row, col) {
    const pos = BigInt(row * 8 + col);
    return ((this.opponent_board >> pos) & 1n) === 1n;
  }

  // 0:空, 1:自分, 2:相手
  getCell(row, col) {
    if (this.isMyStone(row, col)) return 1;
    if (this.isOpponentStone(row, col)) return 2;
    return 0;
  }
}

function renderStoneFromBoard(board, row, col) {
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
  if (cell === 2) {
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
  return null;
}

function App() {
  const size = 8;
  // 初期盤面
  const initialMyBoard = 0x0000001008000000n;
  const initialOpponentBoard = 0x0000000810000000n;

  const [myBoard, setMyBoard] = useState(initialMyBoard);
  const [opponentBoard, setOpponentBoard] = useState(initialOpponentBoard);

  const [turn, setTurn] = useState(0);

  const board = new InnerBoard(myBoard, opponentBoard);
  
  // マスをクリックしたら自分の石を置く（合法手判定なし）
  const handleCellClick = (row, col) => {
    const pos = BigInt(row * 8 + col);
    // 既に石がある場合は何もしない
    if (board.getCell(row, col) !== 0) return;
    if (turn % 2 === 0) {
      setMyBoard(myBoard | (1n << pos));
    }
    else {
      setOpponentBoard(opponentBoard | (1n << pos));
    }
    setTurn(turn + 1);
  };

  const guidePoints = [
    [2, 2],
    [2, 6],
    [6, 2],
    [6, 6],
  ];
  const cellSize = 40;
  const borderSize = 2;

  return (
    <div className="App">
      <h1>Reversi</h1>
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
                    {renderStoneFromBoard(board, row, col)}
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
