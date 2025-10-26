# Test Fixtures

This directory contains sample images for OCR testing.

## Sample Images

### Level ROI
- **File**: `level_126.png`
- **Content**: "LV. 126"
- **Format**: White/Orange text on black background
- **Size**: Small ROI capture from game UI

### EXP ROI
- **File**: `exp_5509611_1276.png`
- **Content**: "EXP. 5509611[12.76%]"
- **Format**: Numbers with brackets and percentage symbol
- **Size**: Small ROI capture from game UI

### Map ROI
- **File**: `map_korean.png`
- **Content**: "히든스트리트 작은 난파선" (Korean text)
- **Format**: Korean text on light background
- **Size**: Small ROI capture from game UI

## Usage

These images are used in OCR integration tests to verify:
1. Tesseract engine configuration
2. Korean language support
3. Text parsing accuracy
4. End-to-end OCR workflow

## Adding New Test Images

When adding new test images:
1. Use descriptive filenames (e.g., `level_XXX.png`, `exp_XXX_YYY.png`)
2. Keep images small (ROI size only)
3. Use PNG format for clarity
4. Document expected OCR output in test code
