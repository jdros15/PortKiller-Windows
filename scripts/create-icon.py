#!/usr/bin/env python3
"""
Create Windows .ico file from PNG image
Requires Pillow: pip install Pillow
"""

import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("Error: Pillow library not found")
    print("Install with: pip install Pillow")
    sys.exit(1)

def create_icon(png_path: str, ico_path: str):
    """Convert PNG to ICO with multiple sizes"""
    
    # Icon sizes to include (Windows standard)
    sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)]
    
    try:
        # Open the source PNG
        img = Image.open(png_path)
        
        # Convert to RGBA if needed
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        
        # Create list of resized images
        icon_images = []
        for size in sizes:
            resized = img.resize(size, Image.Resampling.LANCZOS)
            icon_images.append(resized)
        
        # Save as ICO
        icon_images[0].save(
            ico_path,
            format='ICO',
            sizes=sizes,
            append_images=icon_images[1:]
        )
        
        print(f"✅ Icon created successfully: {ico_path}")
        print(f"   Sizes included: {', '.join(f'{w}x{h}' for w, h in sizes)}")
        
        # Show file size
        ico_size = Path(ico_path).stat().st_size / 1024
        print(f"   File size: {ico_size:.1f} KB")
        
        return True
        
    except Exception as e:
        print(f"❌ Error creating icon: {e}")
        return False

def main():
    # Default paths
    project_root = Path(__file__).parent.parent
    png_path = project_root / "assets" / "app-logo-color.png"
    ico_path = project_root / "assets" / "app-icon.ico"
    
    # Allow custom paths from command line
    if len(sys.argv) > 1:
        png_path = Path(sys.argv[1])
    if len(sys.argv) > 2:
        ico_path = Path(sys.argv[2])
    
    # Validate input
    if not png_path.exists():
        print(f"❌ Error: PNG file not found: {png_path}")
        print("\nUsage: python create-icon.py [input.png] [output.ico]")
        sys.exit(1)
    
    # Create icon
    print(f"🎨 Creating Windows icon...")
    print(f"   Source: {png_path}")
    print(f"   Output: {ico_path}")
    print()
    
    if create_icon(str(png_path), str(ico_path)):
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()
