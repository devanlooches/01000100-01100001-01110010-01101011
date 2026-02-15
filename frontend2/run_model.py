import sys
import os

# Write to both file and stderr to debug
with open('run_model_debug.log', 'w') as f:
    f.write("[run_model.py] Script started!\n")

sys.stderr.write("[run_model.py] Script started!\n")
sys.stderr.flush()

import warnings
import numpy as np
import tensorflow as tf
import keras

def inverse_transform(data):
    # 1. Reverse normalization: (data / 1.5) - 1.0  ->  (data + 1.0) * 1.5
    data = (data + 1.0) * 1.5
    
    # 2. Reverse log10: np.log10(x)  ->  10 ** x
    data = np.power(10, data)
    
    # 3. Reverse the shift: x + 1.0 + 1e-5  ->  x - 1.0 - 1e-5
    data = data - 1.0 - 1e-5
    
    return data

# Define custom loss function
@keras.saving.register_keras_serializable()
class DiceAndMAE(keras.losses.Loss):
    def __init__(self, alpha=0.5, smooth=1e-6, **kwargs):
        super().__init__()
        self.alpha = alpha  # Weight for Dice (0.0 to 1.0)
        self.smooth = smooth
        self.mae = keras.losses.MeanAbsoluteError()

    def call(self, y_true, y_pred):
        # 1. Calculate Dice Loss (Shape)
        y_true_f = tf.reshape(y_true, [-1])
        y_pred_f = tf.reshape(y_pred, [-1])
        intersection = tf.reduce_sum(y_true_f * y_pred_f)
        sum_true = tf.reduce_sum(tf.square(y_true_f))
        sum_pred = tf.reduce_sum(tf.square(y_pred_f))
        dice_loss = 1.0 - (2. * intersection + self.smooth) / (sum_true + sum_pred + self.smooth)

        # 2. Calculate MAE (Physics/Intensity)
        # We use MAE instead of MSE to avoid exploding gradients from the bright halo centers
        mae_loss = self.mae(y_true, y_pred)

        # 3. Combine
        # alpha controls the trade-off. 
        # 0.5 means equal attention to shape and intensity.
        return (self.alpha * dice_loss) + ((1 - self.alpha) * mae_loss)


@keras.saving.register_keras_serializable()
class TverskyAndMAE(keras.losses.Loss):
    def __init__(self, alpha=0.5, beta=2.0, smooth=1e-6, **kwargs):
        super().__init__()
        self.alpha = alpha
        self.beta = beta
        self.smooth = smooth
        self.mae = keras.losses.MeanAbsoluteError()
    
    def call(self, y_true, y_pred):
        # Threshold to [0, 1] for Tversky computation
        # Using a soft threshold around 0
        y_true_binary = tf.nn.sigmoid(y_true * 4)  # Smooth threshold centered at 0
        y_pred_binary = tf.nn.sigmoid(y_pred * 4)
        
        y_true_f = tf.reshape(y_true_binary, [-1])
        y_pred_f = tf.reshape(y_pred_binary, [-1])
        
        tp = tf.reduce_sum(y_true_f * y_pred_f)
        fp = tf.reduce_sum(y_pred_f * (1 - y_true_f))
        fn = tf.reduce_sum(y_true_f * (1 - y_pred_f))
        
        # Tversky index
        tversky = (tp + self.smooth) / (tp + self.alpha*fp + self.beta*fn + self.smooth)
        tversky_loss = 1.0 - tversky
        
        # MAE on original continuous values
        mae_loss = self.mae(y_true, y_pred)
        
        return 0.9 * tversky_loss + 0.1 * mae_loss

# Disable Python buffering so prints appear immediately
os.environ['PYTHONUNBUFFERED'] = '1'

with open('run_model_debug.log', 'a') as f:
    f.write(f"[run_model.py] Python version: {sys.version}\n")

sys.stderr.write(f"[run_model.py] Python version: {sys.version}\n")
sys.stderr.flush()

with open('run_model_debug.log', 'a') as f:
    f.write("[run_model.py] Loading user_input.npy...\n")

sys.stderr.write("[run_model.py] Loading user_input.npy...\n")
sys.stderr.flush()
input_data = np.load("user_input.npy")

# Print the loaded array for debugging
sys.stderr.write(f"[run_model.py] Loaded array:\n{input_data}\n")
sys.stderr.flush()

# input_data = list(input_data.values())
    
# grid = np.full((64, 64, 64), -1.0)

# # Fill the grid with the proper density values
# for entry in input_data:
#     density, x, y, z = entry
#     grid[int(x), int(y), int(z)] = density

# input_data = grid

with open('run_model_debug.log', 'a') as f:
    f.write(f"[run_model.py] Loaded input with shape: {input_data.shape}, dtype: {input_data.dtype}\n")

sys.stderr.write(f"[run_model.py] Loaded input with shape: {input_data.shape}, dtype: {input_data.dtype}\n")
sys.stderr.flush()

sys.stderr.write("[run_model.py] Expanding dimensions...\n")
sys.stderr.flush()
input_data = input_data[..., np.newaxis]
input_data = tf.expand_dims(input_data, axis=0)
sys.stderr.write(f"[run_model.py] Input reshaped to: {input_data.shape}\n")
sys.stderr.flush()

sys.stderr.write("[run_model.py] Loading Keras model...\n")
sys.stderr.flush()
warnings.filterwarnings('ignore') # Ignore all the warning messages in this tutorial
model = tf.keras.models.load_model('model_final_newloss.keras', custom_objects={'DiceAndMAE': DiceAndMAE})
sys.stderr.write("[run_model.py] Keras model loaded successfully\n")
sys.stderr.flush()

sys.stderr.write("[run_model.py] Running inference...\n")
sys.stderr.flush()
output = model.predict(input_data)[0].squeeze()

output = inverse_transform(output)
sys.stderr.write(f"[run_model.py] Inference complete! Output shape: {output.shape}, dtype: {output.dtype}\n")
sys.stderr.flush()

sys.stderr.write("[run_model.py] Saving output to output.npy...\n")
sys.stderr.flush()
np.save('output.npy', output)
sys.stderr.write("[run_model.py] Output saved successfully!\n")
sys.stderr.flush()

sys.stderr.write("[run_model.py] Script completed!\n")
sys.stderr.flush()

