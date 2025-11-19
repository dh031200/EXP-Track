import numpy as np
import onnxruntime as ort
from PIL import Image, ImageOps
import os

# Paths
image_path = r"C:/Users/user/.gemini/antigravity/brain/f263bcaa-008e-4581-87eb-e34107ec34c2/uploaded_image_1763528739090.png"
model_path = r"d:\wjh1065\Projects\EXP-Track\python_ocr_server\models\number_classifier.onnx"
output_dir = r"C:/Users/user/.gemini/antigravity/brain/f263bcaa-008e-4581-87eb-e34107ec34c2/simple_test"

os.makedirs(output_dir, exist_ok=True)

def predict_with_config(session, original_img, slots, expected, crop_percent=0.45, threshold=180, invert=False):
    config_name = f"ResizeFirst_T={threshold}_Inv={invert}"
    print(f"\nTesting: {config_name}")
    
    correct_count = 0
    input_name = session.get_inputs()[0].name
    
    # Grid dimensions
    w, h = original_img.size
    slot_w = w / 4
    slot_h = h / 2
    
    for name, row, col in slots:
        # 1. Crop Slot
        x = int(slot_w * col)
        y = int(slot_h * row)
        slot_img = original_img.crop((x, y, x + int(slot_w), y + int(slot_h)))
        
        # 2. Crop bottom percentage
        width, height = slot_img.size
        crop_h_px = int(height * crop_percent)
        crop_y = height - crop_h_px
        number_region_img = slot_img.crop((0, crop_y, width, height))
        
        # 3. Resize to 92x43 (Bilinear) - This smooths out noise
        resized = number_region_img.resize((92, 43), Image.Resampling.BILINEAR)
        
        # 4. Binary Threshold
        gray = resized.convert('L')
        # If pixel > threshold -> 255 (White), else 0 (Black)
        # This assumes White Text on Dark BG.
        binary = gray.point(lambda p: 255 if p > threshold else 0)
        
        if invert:
            binary = ImageOps.invert(binary)
            
        # Save debug image
        if threshold == 180 and invert == False:
             binary.save(os.path.join(output_dir, f"debug_{name}_resize_first.png"))

        # 5. Prepare for model (Resize to 224x224)
        img_for_model = binary.convert('RGB')
        img_final = img_for_model.resize((224, 224), Image.Resampling.BILINEAR)
        
        img_data = np.array(img_final, dtype=np.float32)
        img_data = img_data / 255.0
        mean = np.array([0.485, 0.456, 0.406], dtype=np.float32)
        std = np.array([0.229, 0.224, 0.225], dtype=np.float32)
        img_data = (img_data - mean) / std
        img_data = img_data.transpose(2, 0, 1)
        img_data = np.expand_dims(img_data, axis=0)

        # 6. Predict
        outputs = session.run(None, {input_name: img_data})
        predicted_idx = np.argmax(outputs[0])

        is_correct = predicted_idx == expected[name]
        status = "✅" if is_correct else f"❌ (Exp: {expected[name]})"
        if is_correct: correct_count += 1
        
        print(f"{name:<10} | {predicted_idx:<10} | {status}")

    print(f"Accuracy: {correct_count}/8")
    return correct_count, config_name

def run_tests():
    if not os.path.exists(image_path):
        print(f"Error: Image not found at {image_path}")
        return

    original_img = Image.open(image_path)

    slots = [
        ("Shift", 0, 0), ("Ins", 0, 1), ("Hm", 0, 2), ("Pup", 0, 3), 
        ("Ctrl", 1, 0), ("Del", 1, 1), ("End", 1, 2), ("Pdn", 1, 3)  
    ]

    expected = {
        "Shift": 136, "Ins": 40, "Hm": 44, "Pup": 105,
        "Ctrl": 42, "Del": 574, "End": 614, "Pdn": 172
    }

    try:
        session = ort.InferenceSession(model_path)
    except Exception as e:
        print(f"Failed to load model: {e}")
        return

    # Test variations
    configs = [
        (0.45, 150, False),
        (0.45, 180, False),
        (0.45, 200, False),
        (0.45, 150, True), # Inverted
        (0.45, 180, True), # Inverted
    ]

    best_acc = -1
    best_config_name = ""

    for crop, thresh, inv in configs:
        acc, name = predict_with_config(session, original_img, slots, expected, crop, thresh, inv)
        if acc > best_acc:
            best_acc = acc
            best_config_name = name
    
    print(f"\n🏆 Best Configuration: {best_config_name} (Accuracy: {best_acc}/8)")

if __name__ == "__main__":
    run_tests()
