# extract_weights.py
import onnx
import numpy as np

def extract_weights_from_onnx(model_path):
    model = onnx.load(model_path)
    
    weights = {}
    for initializer in model.graph.initializer:
        # Tensorから重みデータを取得
        tensor_data = np.frombuffer(initializer.raw_data, dtype=np.float32)
        weights[initializer.name] = tensor_data.reshape(initializer.dims)
    
    return weights

# 重みを抽出
model_path = "src/othello_model_pattern_alt.onnx"
weights = extract_weights_from_onnx(model_path)

# 重みをバイナリファイルとして保存
for name, weight in weights.items():
    filename = f"src/{name.replace('/', '_')}.bin"
    weight.astype(np.float32).tobytes()
    with open(filename, 'wb') as f:
        f.write(weight.astype(np.float32).tobytes())
    print(f"Saved {name} to {filename}, shape: {weight.shape}")