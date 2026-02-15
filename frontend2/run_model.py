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
model = tf.keras.models.load_model('model_final.keras', custom_objects={'DiceAndMAE': DiceAndMAE})
sys.stderr.write("[run_model.py] Keras model loaded successfully\n")
sys.stderr.flush()

sys.stderr.write("[run_model.py] Running inference...\n")
sys.stderr.flush()
output = model(input_data).numpy().squeeze()
sys.stderr.write(f"[run_model.py] Inference complete! Output shape: {output.shape}, dtype: {output.dtype}\n")
sys.stderr.flush()

sys.stderr.write("[run_model.py] Saving output to output.npy...\n")
sys.stderr.flush()
np.save('output.npy', output)
sys.stderr.write("[run_model.py] Output saved successfully!\n")
sys.stderr.flush()

sys.stderr.write("[run_model.py] Script completed!\n")
sys.stderr.flush()

