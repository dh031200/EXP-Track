#!/usr/bin/env python3
"""
Detect numbers using RapidOCR (Auto-scale version)
Automatically scales ROI coordinates based on image size
"""

import numpy as np
from PIL import Image, ImageDraw
from pathlib import Path
from rapidocr import RapidOCR


# ROI definitions as proportions (0.0 to 1.0) of image dimensions
# Based on reference size: 522x255
# Format: {cell_name: (x_min_ratio, x_max_ratio, y_min_ratio, y_max_ratio)}
ROI_PROPORTIONS = {
    # Row 0 (top row)
    'shift': (0.0000, 0.2490, 0.2510, 0.4902),  # x: 0-130, y: 64-125
    'ins':   (0.2490, 0.5000, 0.2510, 0.4902),  # x: 130-261, y: 64-125
    'home':  (0.5000, 0.7490, 0.2510, 0.4902),  # x: 261-391, y: 64-125
    'pup':   (0.7490, 0.9981, 0.2510, 0.4902),  # x: 391-521, y: 64-125
    # Row 1 (bottom row)
    'ctrl':  (0.0000, 0.2490, 0.7686, 0.9961),  # x: 0-130, y: 196-254
    'del':   (0.2490, 0.5000, 0.7686, 0.9961),  # x: 130-261, y: 196-254
    'end':   (0.5000, 0.7490, 0.7686, 0.9961),  # x: 261-391, y: 196-254
    'pdn':   (0.7490, 0.9981, 0.7686, 0.9961),  # x: 391-521, y: 196-254
}


def calculate_roi_coordinates(img_width: int, img_height: int) -> dict:
    """
    Calculate absolute ROI coordinates based on image dimensions

    Args:
        img_width: Image width in pixels
        img_height: Image height in pixels

    Returns:
        Dictionary of ROI definitions with absolute coordinates
    """
    roi_definitions = {}

    for cell_name, (x_min_ratio, x_max_ratio, y_min_ratio, y_max_ratio) in ROI_PROPORTIONS.items():
        roi_definitions[cell_name] = {
            'x_min': int(x_min_ratio * img_width),
            'x_max': int(x_max_ratio * img_width),
            'y_min': int(y_min_ratio * img_height),
            'y_max': int(y_max_ratio * img_height),
        }

    return roi_definitions


def ocr_roi(engine: RapidOCR, img_array: np.ndarray, roi: dict) -> str:
    """
    Extract text from a single ROI using RapidOCR
    Returns empty string if no text detected
    """
    # Crop ROI
    x_min, x_max = roi['x_min'], roi['x_max']
    y_min, y_max = roi['y_min'], roi['y_max']

    roi_img = img_array[y_min:y_max+1, x_min:x_max+1]

    # Run OCR
    result = engine(roi_img)

    # Extract text from result.txts tuple
    if result is None or not result.txts:
        return ""

    # Join all detected texts
    detected_text = ''.join(result.txts)

    # Remove any non-digit characters (safety)
    detected_text = ''.join(c for c in detected_text if c.isdigit())

    return detected_text


def detect_numbers_in_image_roi(image_path: str, template_dir: Path = None, threshold: float = 0.7, debug: bool = True):
    """
    Detect all numbers in inventory image using RapidOCR
    Auto-scales ROI coordinates based on image size

    Args:
        image_path: Path to input image
        template_dir: Not used (kept for compatibility)
        threshold: Not used (kept for compatibility)
        debug: If True, save visualization

    Returns:
        all_detections: List of detection info (for compatibility, returns empty list)
        grid_results: Dict mapping cell name to detected number string
    """
    # Initialize RapidOCR
    engine = RapidOCR()

    # Load image (keep RGB for better OCR)
    img = Image.open(image_path)
    img_array = np.array(img)

    img_width, img_height = img.size

    print(f"Image size: {img_width}x{img_height}")

    # Calculate ROI coordinates for this image size
    roi_definitions = calculate_roi_coordinates(img_width, img_height)

    print(f"Auto-scaled ROI coordinates:")
    for cell_name, roi in list(roi_definitions.items())[:2]:  # Show first 2 as example
        print(f"  {cell_name}: x[{roi['x_min']}-{roi['x_max']}] y[{roi['y_min']}-{roi['y_max']}]")
    print(f"  ... (8 total ROIs)")

    print(f"\nProcessing {len(roi_definitions)} ROIs with RapidOCR...\n")

    # Process each ROI
    grid_results = {}
    detection_info = []  # Store for visualization

    for cell_name, roi in roi_definitions.items():
        detected_number = ocr_roi(engine, img_array, roi)

        if detected_number:
            grid_results[cell_name] = detected_number
            print(f"  [{cell_name:6s}] → {detected_number}")

            # Store detection info for visualization
            detection_info.append({
                'cell': cell_name,
                'text': detected_number,
                'roi': roi
            })
        else:
            print(f"  [{cell_name:6s}] → ---")

    print(f"\nTotal detected cells: {len(grid_results)}/{len(roi_definitions)}")

    # Visualize if requested
    if debug:
        img_rgb = img.convert('RGB')
        draw = ImageDraw.Draw(img_rgb)

        # Draw all ROI boxes
        for cell_name, roi in roi_definitions.items():
            x_min, x_max = roi['x_min'], roi['x_max']
            y_min, y_max = roi['y_min'], roi['y_max']

            # Blue box for all ROIs
            draw.rectangle([x_min, y_min, x_max, y_max], outline='blue', width=1)
            draw.text((x_min+2, y_min+2), cell_name, fill='blue')

        # Draw detection results in green
        for info in detection_info:
            cell_name = info['cell']
            text = info['text']
            roi = info['roi']

            x_min, x_max = roi['x_min'], roi['x_max']
            y_min, y_max = roi['y_min'], roi['y_max']

            # Green overlay for detected ROIs
            draw.rectangle([x_min, y_min, x_max, y_max], outline='lime', width=2)

            # Center text
            center_x = (x_min + x_max) // 2
            center_y = (y_min + y_max) // 2
            draw.text((center_x - 10, center_y - 10), text, fill='lime', font=None)

        # Save visualization
        output_path = Path(image_path).parent / f"{Path(image_path).stem}_autoscale_detected.png"
        img_rgb.save(output_path)
        print(f"\nSaved visualization to: {output_path}")

        # Save results to JSON
        import json
        coords_path = Path(image_path).parent / f"{Path(image_path).stem}_autoscale_results.json"
        with open(coords_path, 'w') as f:
            json.dump({
                'image': Path(image_path).name,
                'method': 'RapidOCR (Auto-scale)',
                'image_size': f"{img_width}x{img_height}",
                'roi_coordinates': roi_definitions,
                'total_detected_cells': len(grid_results),
                'grid_results': grid_results,
            }, f, indent=2)
        print(f"Saved results to: {coords_path}")

    # Return empty list for all_detections (compatibility with run_e2e.py)
    return [], grid_results


def main():
    import sys

    if len(sys.argv) < 2:
        print("Usage: python detect_numbers_rapidocr_autoscale.py <image_path>")
        sys.exit(1)

    image_path = sys.argv[1]

    print("="*60)
    print("Number Detection using RapidOCR (Auto-scale Method)")
    print("="*60)

    detections, grid_results = detect_numbers_in_image_roi(image_path, debug=True)

    print("\n" + "="*60)
    print("GRID RESULTS (4x2 Inventory Layout)")
    print("="*60)

    # Display grid layout
    grid_names = [
        ['shift', 'ins', 'home', 'pup'],
        ['ctrl', 'del', 'end', 'pdn']
    ]

    for row_idx, row_names in enumerate(grid_names):
        row_values = []
        for cell_name in row_names:
            value = grid_results.get(cell_name, '---')
            row_values.append(f"[{cell_name}:{value:>6s}]")
        print(f"Row {row_idx+1}: " + " ".join(row_values))

    print("\nDetailed values:")
    for cell_name in ['shift', 'ins', 'home', 'pup', 'ctrl', 'del', 'end', 'pdn']:
        value = grid_results.get(cell_name, 'N/A')
        print(f"  {cell_name:6s} = {value}")


if __name__ == "__main__":
    main()
