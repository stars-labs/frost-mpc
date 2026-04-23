#!/bin/sh

# One-off console-log cleanup script from an earlier refactor.
#
# Comments out decorative console.logs and keeps console.error
# plus security/audit-relevant messages. Creates a
# src.backup.<timestamp> directory before editing.
#
# If you run this again, audit + extend FILES_TO_CLEAN below to
# match the current tree — several 2024-vintage files in the
# original list (webrtcConnection.ts, messageRouter.ts,
# wasmInitializer.ts, patternRouter.ts, utils/messageHandler.ts,
# utils/sessionActions.ts, utils/uiState.ts, components/*.svelte)
# have since been consolidated or renamed. The \`file not found\`
# branch in the loop skips silently; this is a coverage warning,
# not a crash.

echo "🧹 Starting console log cleanup..."

# Create backup
echo "📦 Creating backup..."
cp -r src src.backup.$(date +%Y%m%d_%H%M%S)

# Files still in the tree that are likely to hold debug noise.
FILES_TO_CLEAN=(
    "src/entrypoints/offscreen/webrtc.ts"
    "src/entrypoints/background/messageHandlers.ts"
    "src/entrypoints/background/stateManager.ts"
    "src/entrypoints/content/provider.ts"
    "src/entrypoints/background/index.ts"
    "src/entrypoints/offscreen/index.ts"
    "src/entrypoints/background/webSocketManager.ts"
    "src/entrypoints/popup/App.svelte"
)

# Function to comment out debug logs
comment_debug_logs() {
    local file=$1
    echo "🔧 Processing: $file"
    
    # Create temp file
    local temp_file="${file}.tmp"
    
    # Process the file
    awk '
    # Skip already commented lines
    /^[[:space:]]*\/\// { print; next }
    
    # Keep error logs
    /console\.error/ { print; next }
    
    # Keep specific security/audit logs
    /Permission (granted|revoked)/ { print; next }
    /Signature (approved|rejected)/ { print; next }
    /Account created/ { print; next }
    /Network added/ { print; next }
    
    # Comment out debug logs with specific patterns
    /console\.log.*\[.*DEBUG.*\]/ { print "// " $0; next }
    /console\.log.*"🔍/ { print "// " $0; next }
    /console\.log.*"🟡/ { print "// " $0; next }
    /console\.log.*"📊/ { print "// " $0; next }
    /console\.log.*"📡/ { print "// " $0; next }
    /console\.log.*"🔧/ { print "// " $0; next }
    /console\.log.*"✅/ { print "// " $0; next }
    /console\.log.*"🔄/ { print "// " $0; next }
    /console\.log.*"📤/ { print "// " $0; next }
    /console\.log.*"📨/ { print "// " $0; next }
    /console\.log.*"🎯/ { print "// " $0; next }
    /console\.log.*"🖥️/ { print "// " $0; next }
    /console\.log.*"🔌/ { print "// " $0; next }
    /console\.log.*"🚀/ { print "// " $0; next }
    /console\.log.*"🎉/ { print "// " $0; next }
    /console\.log.*"🔗/ { print "// " $0; next }
    /console\.log.*"🧊/ { print "// " $0; next }
    /console\.log.*"💥/ { print "// " $0; next }
    
    # Comment out message routing logs
    /console\.log.*Processing.*message/ { print "// " $0; next }
    /console\.log.*Message.*received/ { print "// " $0; next }
    /console\.log.*Routing.*to/ { print "// " $0; next }
    /console\.log.*Forwarding.*to/ { print "// " $0; next }
    
    # Comment out state update logs
    /console\.log.*State.*update/ { print "// " $0; next }
    /console\.log.*Updating.*state/ { print "// " $0; next }
    /console\.log.*UI preferences/ { print "// " $0; next }
    
    # Comment out WebRTC connection logs
    /console\.log.*connection state:/ { print "// " $0; next }
    /console\.log.*Data channel/ { print "// " $0; next }
    /console\.log.*ICE candidate/ { print "// " $0; next }
    /console\.log.*Handling.*from/ { print "// " $0; next }
    
    # Comment out WASM debug logs
    /console\.log.*WASM.*modules/ { print "// " $0; next }
    /console\.log.*typeof.*Frost/ { print "// " $0; next }
    /console\.log.*FROST DKG INIT/ { print "// " $0; next }
    
    # Comment out decorative logs
    /console\.log.*"[│┌└─]/ { print "// " $0; next }
    
    # Default: keep the line as is
    { print }
    ' "$file" > "$temp_file"
    
    # Replace original file
    mv "$temp_file" "$file"
}

# Process each file
for file in "${FILES_TO_CLEAN[@]}"; do
    if [ -f "$file" ]; then
        comment_debug_logs "$file"
    else
        echo "⚠️  File not found: $file"
    fi
done

echo "✅ Console log cleanup complete!"
echo "📁 Backup created in: src.backup.*"
echo ""
echo "🔍 Remaining console statements:"
grep -r "console\." src --include="*.ts" --include="*.js" --include="*.svelte" | grep -v "^[[:space:]]*\/\/" | wc -l