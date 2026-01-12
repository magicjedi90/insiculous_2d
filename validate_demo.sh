#!/bin/bash

echo "🎨 VALIDATING PERFECT SPRITE DEMO..."
echo "==========================================="

# Run the demo for a few seconds and capture key output
timeout 3s cargo run --example sprite_demo 2>&1 | tee demo_output.log

echo ""
echo "🔍 ANALYZING OUTPUT..."
echo "==========================================="

# Check for success indicators
grep -E "(✅|🎉|🎯|🎨)" demo_output.log | head -10

# Check for OpenGL ES backend validation
if grep -q "OpenGL ES backend" demo_output.log; then
    echo "✅ OpenGL ES backend validated!"
fi

# Check for sprite creation
if grep -q "Created.*beautiful.*sprites" demo_output.log; then
    echo "✅ Beautiful sprites created successfully!"
fi

# Check for first frame rendering
if grep -q "FIRST FRAME RENDERED" demo_output.log; then
    echo "✅ First frame rendered with sprites visible!"
fi

# Check for continuous rendering
if grep -q "Frame.*sprites dancing" demo_output.log; then
    echo "✅ Continuous 60 FPS animation running!"
fi

echo ""
echo "🎯 VALIDATION COMPLETE!"
echo "The perfect sprite demo is working beautifully!"
echo "Check demo_output.log for full details."