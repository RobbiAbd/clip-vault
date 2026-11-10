#!/bin/bash
# Create minimal valid PNG files as placeholders

# Minimal 1x1 transparent PNG (base64 encoded)
BASE64_PNG="iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="

echo "$BASE64_PNG" | base64 -d > 32x32.png
echo "$BASE64_PNG" | base64 -d > 128x128.png
echo "$BASE64_PNG" | base64 -d > 128x128@2x.png
echo "$BASE64_PNG" | base64 -d > icon.png

echo "Placeholder icons created"
ls -lh *.png
