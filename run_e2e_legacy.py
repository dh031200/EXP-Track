#!/usr/bin/env python3
"""
End-to-end OCR pipeline for MapleStory inventory
1. Extract inventory regions from screenshots
2. Detect and recognize numbers in inventory slots
3. Save results and timing information
"""

import time
import json
import shutil
from pathlib import Path
from datetime import datetime
from extract_inventory_final import extract_inventory_region
from detect_numbers_roi import detect_numbers_in_image_roi


def run_e2e_pipeline():
    """Run complete end-to-end OCR pipeline"""

    # Setup directories
    sample_dir = Path("sample_images")
    output_dir = Path("filtered_output")
    template_dir = Path("nums_processed")
    results_dir = Path("results")

    output_dir.mkdir(exist_ok=True)
    results_dir.mkdir(exist_ok=True)

    # Timestamp for this run
    run_timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    # Create timestamped subdirectory for this run
    run_results_dir = results_dir / f"run_{run_timestamp}"
    run_results_dir.mkdir(exist_ok=True)

    # Create subdirectories for images
    cropped_dir = run_results_dir / "cropped"
    detected_dir = run_results_dir / "detected"
    cropped_dir.mkdir(exist_ok=True)
    detected_dir.mkdir(exist_ok=True)

    print("="*80)
    print("MapleStory Inventory OCR - End-to-End Pipeline (ROI-optimized)")
    print("="*80)
    print(f"Run timestamp: {run_timestamp}\n")

    # Get all input images
    images = sorted(sample_dir.glob("*.png"))
    images = [img for img in images if not any(x in img.name for x in
              ['_binary', '_morph', '_filtered', '_final', '_components', '_cross', '_inventory', '_triple'])]

    print(f"Found {len(images)} input images\n")

    # Results storage
    all_results = []

    # ===== STAGE 1: Extract Inventory Regions =====
    print("="*80)
    print("STAGE 1: Extracting Inventory Regions")
    print("="*80)

    stage1_start = time.time()

    extraction_results = {}

    for img_path in images:
        img_name = img_path.stem
        print(f"\n[1/2] {img_name}")

        extract_start = time.time()

        try:
            bbox, num_candidates = extract_inventory_region(
                str(img_path),
                output_dir=output_dir,
                debug=False
            )

            extract_time = time.time() - extract_start

            if bbox:
                left, top, right, bottom = bbox
                width, height = right - left + 1, bottom - top + 1
                print(f"  ✅ Extracted: ({left},{top}) to ({right},{bottom}) {width}x{height} [{extract_time:.3f}s]")

                # Copy cropped image to results
                cropped_img = output_dir / f"{img_name}_5_threshold_1.png"
                if cropped_img.exists():
                    shutil.copy(cropped_img, cropped_dir / f"{img_name}.png")

                extraction_results[img_name] = {
                    'success': True,
                    'bbox': bbox,
                    'time': extract_time
                }
            else:
                print(f"  ❌ Extraction failed [{extract_time:.3f}s]")
                extraction_results[img_name] = {
                    'success': False,
                    'time': extract_time
                }

        except Exception as e:
            extract_time = time.time() - extract_start
            print(f"  ❌ Error: {e} [{extract_time:.3f}s]")
            extraction_results[img_name] = {
                'success': False,
                'error': str(e),
                'time': extract_time
            }

    stage1_time = time.time() - stage1_start

    successful_extractions = sum(1 for r in extraction_results.values() if r['success'])
    print(f"\nStage 1 Complete: {successful_extractions}/{len(images)} successful [{stage1_time:.3f}s]")

    # ===== STAGE 2: Detect Numbers =====
    print("\n" + "="*80)
    print("STAGE 2: Detecting Numbers in Inventory Slots")
    print("="*80)

    stage2_start = time.time()

    # Get all threshold_1 images created in stage 1
    threshold_images = sorted(output_dir.glob("*_5_threshold_1.png"))

    for img_path in threshold_images:
        img_name = img_path.stem.replace('_5_threshold_1', '')
        print(f"\n[2/2] {img_name}")

        detect_start = time.time()

        try:
            detections, grid_results = detect_numbers_in_image_roi(
                str(img_path),
                template_dir,
                threshold=0.6,
                debug=True  # Enable visualization
            )

            detect_time = time.time() - detect_start

            print(f"  ✅ Detected {len(detections)} digits [{detect_time:.3f}s]")
            print(f"     Row 1: [shift:{grid_results.get('shift', '---'):>6s}] [ins:{grid_results.get('ins', '---'):>6s}] [home:{grid_results.get('home', '---'):>6s}] [pup:{grid_results.get('pup', '---'):>6s}]")
            print(f"     Row 2: [ctrl:{grid_results.get('ctrl', '---'):>6s}] [del:{grid_results.get('del', '---'):>6s}] [end:{grid_results.get('end', '---'):>6s}] [pdn:{grid_results.get('pdn', '---'):>6s}]")

            # Copy detected image to results
            # Note: img_name is already without _5_threshold_1 suffix
            detected_img = img_path.parent / f"{img_path.stem}_detected_roi.png"
            if detected_img.exists():
                shutil.copy(detected_img, detected_dir / f"{img_name}.png")

            # Store results
            extraction_info = extraction_results.get(img_name, {})

            all_results.append({
                'image': img_name,
                'timestamp': run_timestamp,
                'extraction': {
                    'success': extraction_info.get('success', False),
                    'bbox': extraction_info.get('bbox'),
                    'time': extraction_info.get('time', 0.0)
                },
                'detection': {
                    'success': True,
                    'num_detections': len(detections),
                    'grid': grid_results,
                    'time': detect_time
                },
                'total_time': extraction_info.get('time', 0.0) + detect_time
            })

        except Exception as e:
            detect_time = time.time() - detect_start
            print(f"  ❌ Error: {e} [{detect_time:.3f}s]")

            extraction_info = extraction_results.get(img_name, {})

            all_results.append({
                'image': img_name,
                'timestamp': run_timestamp,
                'extraction': {
                    'success': extraction_info.get('success', False),
                    'bbox': extraction_info.get('bbox'),
                    'time': extraction_info.get('time', 0.0)
                },
                'detection': {
                    'success': False,
                    'error': str(e),
                    'time': detect_time
                },
                'total_time': extraction_info.get('time', 0.0) + detect_time
            })

    stage2_time = time.time() - stage2_start

    successful_detections = sum(1 for r in all_results if r['detection']['success'])
    print(f"\nStage 2 Complete: {successful_detections}/{len(threshold_images)} successful [{stage2_time:.3f}s]")

    # ===== SUMMARY =====
    total_time = time.time() - stage1_start

    print("\n" + "="*80)
    print("END-TO-END PIPELINE SUMMARY")
    print("="*80)

    print(f"\nTotal images processed: {len(images)}")
    print(f"Successful extractions: {successful_extractions}/{len(images)}")
    print(f"Successful detections: {successful_detections}/{len(threshold_images)}")

    print(f"\nTiming:")
    print(f"  Stage 1 (Extraction): {stage1_time:.3f}s")
    print(f"  Stage 2 (Detection):  {stage2_time:.3f}s")
    print(f"  Total Pipeline:       {total_time:.3f}s")
    print(f"  Average per image:    {total_time/len(images):.3f}s")

    # ===== SAVE RESULTS =====
    results_file = run_results_dir / "ocr_results.json"

    results_json = {
        'run_info': {
            'timestamp': run_timestamp,
            'total_images': len(images),
            'successful_extractions': successful_extractions,
            'successful_detections': successful_detections,
            'stage1_time': stage1_time,
            'stage2_time': stage2_time,
            'total_time': total_time,
            'avg_time_per_image': total_time / len(images)
        },
        'results': all_results
    }

    with open(results_file, 'w', encoding='utf-8') as f:
        json.dump(results_json, f, indent=2, ensure_ascii=False)

    print(f"\n✅ Results saved to: {results_file}")

    # ===== SAVE SUMMARY CSV =====
    csv_file = run_results_dir / "ocr_summary.csv"

    with open(csv_file, 'w', encoding='utf-8') as f:
        # Header
        f.write("image,extraction_success,detection_success,num_detections,")
        f.write("shift,ins,home,pup,ctrl,del,end,pdn,")
        f.write("extraction_time,detection_time,total_time\n")

        # Data rows
        for result in all_results:
            img = result['image']
            ext_success = 'Y' if result['extraction']['success'] else 'N'
            det_success = 'Y' if result['detection']['success'] else 'N'
            num_det = result['detection'].get('num_detections', 0)

            grid = result['detection'].get('grid', {})
            shift = grid.get('shift', '')
            ins = grid.get('ins', '')
            home = grid.get('home', '')
            pup = grid.get('pup', '')
            ctrl = grid.get('ctrl', '')
            del_ = grid.get('del', '')
            end = grid.get('end', '')
            pdn = grid.get('pdn', '')

            ext_time = result['extraction']['time']
            det_time = result['detection']['time']
            total_time = result['total_time']

            f.write(f"{img},{ext_success},{det_success},{num_det},")
            f.write(f"{shift},{ins},{home},{pup},{ctrl},{del_},{end},{pdn},")
            f.write(f"{ext_time:.3f},{det_time:.3f},{total_time:.3f}\n")

    print(f"✅ Summary CSV saved to: {csv_file}")

    # ===== DETAILED RESULTS =====
    print("\n" + "="*80)
    print("DETAILED RESULTS")
    print("="*80)

    for result in all_results:
        img = result['image']
        grid = result['detection'].get('grid', {})
        total_time = result['total_time']
        num_det = result['detection'].get('num_detections', 0)

        print(f"\n{img:50s} ({num_det:2d} digits, {total_time:.3f}s)")
        if result['detection']['success']:
            print(f"  Row 1: [shift:{grid.get('shift', '---'):>6s}] [ins:{grid.get('ins', '---'):>6s}] [home:{grid.get('home', '---'):>6s}] [pup:{grid.get('pup', '---'):>6s}]")
            print(f"  Row 2: [ctrl:{grid.get('ctrl', '---'):>6s}] [del:{grid.get('del', '---'):>6s}] [end:{grid.get('end', '---'):>6s}] [pdn:{grid.get('pdn', '---'):>6s}]")
        else:
            print(f"  ❌ Detection failed: {result['detection'].get('error', 'Unknown error')}")

    print("\n" + "="*80)
    print("OUTPUT FILES")
    print("="*80)
    print(f"\nResults directory: {run_results_dir}/")
    print(f"\n  📁 cropped/           - Cropped inventory images (522x255)")
    print(f"  📁 detected/          - Detection visualizations with ROI boxes")
    print(f"  📄 ocr_results.json   - Full JSON results")
    print(f"  📄 ocr_summary.csv    - Summary CSV")

    print(f"\nIntermediate files (filtered_output/):")
    print(f"  *_0_greyscale.png       - Greyscale images")
    print(f"  *_1_binary.png          - Binary thresholded")
    print(f"  *_2_morph.png           - Morphology applied")
    print(f"  *_3_filtered.png        - Filtered regions")
    print(f"  *_4_final.png           - Visualizations")
    print(f"  *_5_threshold_1.png     - OCR input (522x255)")
    print(f"  *_detected_roi.png      - Detection results")

    print("\n" + "="*80)
    print("PIPELINE COMPLETE")
    print("="*80)

    return results_json


if __name__ == "__main__":
    results = run_e2e_pipeline()
