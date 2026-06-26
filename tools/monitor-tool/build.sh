#!/bin/bash
# Build MonitorTool.jar
# 需要 Android SDK 的 android.jar 在 classpath 中

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$SCRIPT_DIR/MonitorTool.java"
OUT_DIR="$SCRIPT_DIR/build"
JAR_OUT="$SCRIPT_DIR/../../assets/runtime/MonitorTool.jar"

# 查找 android.jar
if [ -z "$ANDROID_SDK" ]; then
    # 尝试常见路径
    for p in \
        "$HOME/Library/Android/sdk/platforms/android-34/android.jar" \
        "$HOME/Android/Sdk/platforms/android-34/android.jar" \
        "/usr/local/lib/android/sdk/platforms/android-34/android.jar"; do
        if [ -f "$p" ]; then
            ANDROID_SDK="$p"
            break
        fi
    done
fi

if [ -z "$ANDROID_SDK" ] || [ ! -f "$ANDROID_SDK" ]; then
    echo "Error: android.jar not found."
    echo "Set ANDROID_SDK to the path of android.jar"
    echo "  e.g. export ANDROID_SDK=~/Library/Android/sdk/platforms/android-34/android.jar"
    exit 1
fi

echo "Using android.jar: $ANDROID_SDK"

# 编译
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
javac -source 8 -target 8 -bootclasspath "$ANDROID_SDK" -d "$OUT_DIR" "$SRC"

# 打包
cd "$OUT_DIR"
jar cf MonitorTool.jar MonitorTool.class
cp MonitorTool.jar "$JAR_OUT"

echo "Built: $JAR_OUT"
ls -la "$JAR_OUT"

# 清理
rm -rf "$OUT_DIR"
