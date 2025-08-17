use crate::play::Board;
use std::sync::LazyLock;

// パターン定義（eval.rsと同じ）
static PATTERNS: &[&[u8]] = &[
    // Group 0: edge + 2X
    &[0, 1, 2, 3, 4, 5, 6, 7, 9, 14],
    &[0, 8, 9, 16, 24, 32, 40, 48, 49, 56],
    &[7, 14, 15, 23, 31, 39, 47, 54, 55, 63],
    &[49, 54, 56, 57, 58, 59, 60, 61, 62, 63],

    // Group 1: hor2
    &[8, 9, 10, 11, 12, 13, 14, 15],
    &[48, 49, 50, 51, 52, 53, 54, 55],
    &[1, 9, 17, 25, 33, 41, 49, 57],
    &[6, 14, 22, 30, 38, 46, 54, 62],

    // Group 2: hor3
    &[16, 17, 18, 19, 20, 21, 22, 23],
    &[40, 41, 42, 43, 44, 45, 46, 47],
    &[2, 10, 18, 26, 34, 42, 50, 58],
    &[5, 13, 21, 29, 37, 45, 53, 61],

    // Group 3: hor4
    &[24, 25, 26, 27, 28, 29, 30, 31],
    &[32, 33, 34, 35, 36, 37, 38, 39],
    &[3, 11, 19, 27, 35, 43, 51, 59],
    &[4, 12, 20, 28, 36, 44, 52, 60],

    // Group 4: diag4 corner
    &[0, 1, 2, 3, 8, 9, 10, 16, 17, 24],
    &[4, 5, 6, 7, 13, 14, 15, 22, 23, 31],
    &[32, 40, 41, 48, 49, 50, 56, 57, 58, 59],
    &[39, 46, 47, 53, 54, 55, 60, 61, 62, 63],

    // Group 5: diag5 + 3X
    &[4, 9, 11, 14, 18, 25, 32, 49],
    &[14, 31, 38, 45, 49, 52, 54, 59],
    &[3, 9, 12, 14, 21, 30, 39, 54],
    &[9, 24, 33, 42, 49, 51, 54, 60],

    // Group 6: diag6-C-corner
    &[5, 6, 7, 12, 19, 26, 33, 40, 48, 56],
    &[7, 15, 23, 30, 37, 44, 51, 56, 57, 58],
    &[0, 1, 2, 11, 20, 29, 38, 47, 55, 63],
    &[0, 8, 16, 25, 34, 43, 52, 61, 62, 63],

    // Group 7: diag7-corner
    &[6, 7, 13, 20, 27, 34, 41, 48, 56],
    &[7, 15, 22, 29, 36, 43, 50, 56, 57],
    &[0, 1, 10, 19, 28, 37, 46, 55, 63],
    &[0, 8, 17, 26, 35, 44, 53, 62, 63],

    // Group 8: diag8 + 2C
    &[0, 1, 8, 9, 18, 27, 36, 45, 54, 63],
    &[0, 9, 18, 27, 36, 45, 54, 55, 62, 63],
    &[6, 7, 14, 15, 21, 28, 35, 42, 49, 56],
    &[7, 14, 21, 28, 35, 42, 48, 49, 56, 57],

    // Group 9: 33-corner
    &[0, 1, 2, 8, 9, 10, 16, 17, 18],
    &[5, 6, 7, 13, 14, 15, 21, 22, 23],
    &[40, 41, 42, 48, 49, 50, 56, 57, 58],
    &[45, 46, 47, 53, 54, 55, 61, 62, 63],

    // Group 10: wing-corner
    &[0, 1, 2, 3, 4, 8, 9, 16, 24, 32],
    &[3, 4, 5, 6, 7, 14, 15, 23, 31, 39],
    &[24, 32, 40, 48, 49, 56, 57, 58, 59, 60],
    &[31, 39, 47, 54, 55, 59, 60, 61, 62, 63],

    // Group 11: 24-midedge + corner
    &[0, 2, 3, 4, 5, 7, 10, 11, 12, 13],
    &[0, 16, 17, 24, 25, 32, 33, 40, 41, 56],
    &[7, 22, 23, 30, 31, 38, 39, 46, 47, 63],
    &[50, 51, 52, 53, 56, 58, 59, 60, 61, 63],
     
    // Group 12: flint
    &[0, 1, 8, 9, 10, 11, 17, 18, 25, 27],
    &[6, 7, 12, 13, 14, 15, 21, 22, 28, 30],
    &[33, 35, 41, 42, 48, 49, 50, 51, 56, 57],
    &[36, 38, 45, 46, 52, 53, 54, 55, 62, 63]
];

const PATTERN_MASK: [u64; 52] = [
    0x42ff,
    0x103010101010301,
    0x80c080808080c080,
    0xff42000000000000,
    0xff00,
    0xff000000000000,
    0x202020202020202,
    0x4040404040404040,
    0xff0000,
    0xff0000000000,
    0x404040404040404,
    0x2020202020202020,
    0xff000000,
    0xff00000000,
    0x808080808080808,
    0x1010101010101010,
    0x103070f,
    0x80c0e0f0,
    0xf07030100000000,
    0xf0e0c08000000000,
    0x2000102044a10,
    0x852204080004000,
    0x40008040205208,
    0x104a040201000200,
    0x1010102040810e0,
    0x708102040808080,
    0x8080804020100807,
    0xe010080402010101,
    0x1010204081020c0,
    0x304081020408080,
    0x8080402010080403,
    0xc020100804020101,
    0x8040201008040303,
    0xc0c0201008040201,
    0x10204081020c0c0,
    0x303040810204080,
    0x70707,
    0xe0e0e0,
    0x707070000000000,
    0xe0e0e00000000000,
    0x10101031f,
    0x808080c0f8,
    0x1f03010101000000,
    0xf8c0808080000000,
    0x3cbd,
    0x100030303030001,
    0x8000c0c0c0c00080,
    0xbd3c000000000000,
    0xa060f03,
    0x5060f0c0,
    0x30f060a00000000,
    0xc0f0605000000000,
];

// 重みデータを埋め込み
const EMBEDDING_0_WEIGHTS: &[u8] = include_bytes!("embedding_layers.0.weight.bin");
const EMBEDDING_1_WEIGHTS: &[u8] = include_bytes!("embedding_layers.1.weight.bin");
const EMBEDDING_2_WEIGHTS: &[u8] = include_bytes!("embedding_layers.2.weight.bin");
const DENSE_0_WEIGHTS: &[u8] = include_bytes!("dense_layers.0.weight.bin");
const DENSE_0_BIAS: &[u8] = include_bytes!("dense_layers.0.bias.bin");
const DENSE_2_WEIGHTS: &[u8] = include_bytes!("dense_layers.2.weight.bin");
const DENSE_2_BIAS: &[u8] = include_bytes!("dense_layers.2.bias.bin");

// ビット操作用のルックアップテーブル（eval.rsと同じロジック）
static BIT_TO_TERNARY_TABLES: LazyLock<Vec<Vec<i64>>> = LazyLock::new(|| {
    let mut tables = Vec::new();
    
    for (pattern_id, pattern) in PATTERNS.iter().enumerate() {
        let pattern_len = pattern.len();
        let table_size = 1usize << (pattern_len * 2);
        let mut conversion_table = vec![0i64; table_size];
        
        for my_bits in 0..(1 << pattern_len) {
            for opp_bits in 0..(1 << pattern_len) {
                if (my_bits & opp_bits) == 0 {
                    let combined_index = (my_bits << pattern_len) | opp_bits;
                    if combined_index < conversion_table.len() {
                        let mut ternary_index = 0i64;
                        let mut power = 1i64;
                        
                        for i in 0..pattern_len {
                            let bit_pos = 1 << i;
                            let ternary_value = if (my_bits & bit_pos) != 0 {
                                1
                            } else if (opp_bits & bit_pos) != 0 {
                                2
                            } else {
                                0
                            };
                            
                            ternary_index += ternary_value * power;
                            power *= 3;
                        }
                        
                        conversion_table[combined_index] = ternary_index;
                    }
                }
            }
        }
        
        tables.push(conversion_table);
    }
    
    tables
});

// WebAssembly対応のニューラルネットワーク
pub struct EmbeddingNeuralNetwork {
    // 埋め込み層の重み
    embedding_0: Vec<f32>, // (6561, 8)
    embedding_1: Vec<f32>, // (19683, 8)  
    embedding_2: Vec<f32>, // (59049, 8)
    
    // 全結合層の重みとバイアス
    dense_0_weights: Vec<f32>, // (128, 416)
    dense_0_bias: Vec<f32>,    // (128,)
    dense_2_weights: Vec<f32>, // (1, 128)
    dense_2_bias: f32,         // (1,)
}

impl EmbeddingNeuralNetwork {
    pub fn new() -> Self {
        Self {
            embedding_0: Self::bytes_to_f32_vec(EMBEDDING_0_WEIGHTS),
            embedding_1: Self::bytes_to_f32_vec(EMBEDDING_1_WEIGHTS),
            embedding_2: Self::bytes_to_f32_vec(EMBEDDING_2_WEIGHTS),
            dense_0_weights: Self::bytes_to_f32_vec(DENSE_0_WEIGHTS),
            dense_0_bias: Self::bytes_to_f32_vec(DENSE_0_BIAS),
            dense_2_weights: Self::bytes_to_f32_vec(DENSE_2_WEIGHTS),
            dense_2_bias: Self::bytes_to_f32_vec(DENSE_2_BIAS)[0],
        }
    }

    fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }

    pub fn forward(&self, pattern_indices: &[i64]) -> f32 {
        // パターンを3つのグループに分ける（元のモデルの構造に合わせて）
        let mut embedded_features = Vec::with_capacity(416); // 52 * 8 = 416
        
        // 各パターンを埋め込み
        for (i, &pattern_idx) in pattern_indices.iter().enumerate() {
            let pattern_idx = pattern_idx as usize;
            
            // パターンのサイズに基づいて適切な埋め込み層を選択
            let pattern_size = PATTERNS[i].len();
            let embedding_values = if pattern_size <= 8 {
                // 小さなパターン用の埋め込み
                if pattern_idx < 6561 {
                    &self.embedding_0[pattern_idx * 8..(pattern_idx + 1) * 8]
                } else {
                    &[0.0; 8] // インデックスが範囲外の場合はゼロパディング
                }
            } else if pattern_size <= 10 {
                // 中サイズパターン用の埋め込み
                if pattern_idx < 19683 {
                    &self.embedding_1[pattern_idx * 8..(pattern_idx + 1) * 8]
                } else {
                    &[0.0; 8]
                }
            } else {
                // 大きなパターン用の埋め込み
                if pattern_idx < 59049 {
                    &self.embedding_2[pattern_idx * 8..(pattern_idx + 1) * 8]
                } else {
                    &[0.0; 8]
                }
            };
            
            embedded_features.extend_from_slice(embedding_values);
        }

        // 全結合層1（隠れ層）の計算
        let mut hidden = vec![0.0f32; 128];
        for i in 0..128 {
            let mut sum = self.dense_0_bias[i];
            for j in 0..416 {
                sum += embedded_features[j] * self.dense_0_weights[i * 416 + j];
            }
            hidden[i] = sum.max(0.0); // ReLU活性化関数
        }

        // 出力層の計算
        let mut output = self.dense_2_bias;
        for i in 0..128 {
            output += hidden[i] * self.dense_2_weights[i];
        }

        output
    }
}

// WebAssembly対応の評価関数
pub struct EvalFunction {
    neural_net: EmbeddingNeuralNetwork,
}

impl EvalFunction {
    pub fn new() -> Self {
        Self {
            neural_net: EmbeddingNeuralNetwork::new(),
        }
    }

    pub fn eval(&self, board: &Board) -> f32 {
        // パターンインデックスを計算（eval.rsと同じロジック）
        let pattern_indices: Vec<i64> = PATTERNS.iter().enumerate().map(|(pattern_id, pattern)| {
            let mask = PATTERN_MASK[pattern_id];
            
            let my_masked = board.my_board & mask;
            let opp_masked = board.opponent_board & mask;
            
            let my_compressed = self.compress_bits(my_masked, pattern);
            let opp_compressed = self.compress_bits(opp_masked, pattern);
            
            let pattern_len = pattern.len();
            let combined_index = (my_compressed << pattern_len) | opp_compressed;
            
            if combined_index < BIT_TO_TERNARY_TABLES[pattern_id].len() {
                BIT_TO_TERNARY_TABLES[pattern_id][combined_index]
            } else {
                self.direct_ternary_calculation(pattern, board)
            }
        }).collect();

        // ニューラルネットワークで評価
        self.neural_net.forward(&pattern_indices)
    }

    fn compress_bits(&self, bits: u64, pattern: &[u8]) -> usize {
        let mut compressed = 0usize;
        let mut compressed_pos = 0;
        
        for &pos in pattern.iter() {
            if (bits >> pos) & 1 != 0 {
                compressed |= 1 << compressed_pos;
            }
            compressed_pos += 1;
        }
        
        compressed
    }

    fn direct_ternary_calculation(&self, pattern: &[u8], board: &Board) -> i64 {
        let mut index = 0i64;
        let mut power = 1i64;
        
        for &pos in pattern.iter() {
            let ternary_value = if (board.my_board >> pos) & 1 != 0 {
                1
            } else if (board.opponent_board >> pos) & 1 != 0 {
                2
            } else {
                0
            };
            
            index += ternary_value * power;
            power *= 3;
        }
        
        index
    }
}

// グローバルな評価関数インスタンス
pub static EVAL_FUNCTION: LazyLock<EvalFunction> = LazyLock::new(|| {
    EvalFunction::new()
});
