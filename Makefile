.PHONY: all jar clean dev build test

ANDROID_API    := 34
ANDROID_JAR    := tools/monitor-tool/android.jar
JAR_SRC        := tools/monitor-tool/MonitorTool.java
JAR_OUT        := mimonitor/resources/MonitorTool.jar

# Local SDK cache (not committed)
SDK_DIR        := tools/monitor-tool/.sdk
CMDLINE_TOOLS  := $(SDK_DIR)/cmdline-tools/latest/bin/sdkmanager
D8             := $(SDK_DIR)/build-tools/34.0.0/d8

# Proxy (set via env or make PROXY=...)
PROXY          ?= $(or $(https_proxy),$(http_proxy))
CURL           := curl --retry 3 --retry-delay 5 $(if $(PROXY),-x $(PROXY),)

# Platform detection for cmdline-tools download
UNAME_S        := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
  CMDLINE_URL  := https://dl.google.com/android/repository/commandlinetools-mac-11076708_latest.zip
else
  CMDLINE_URL  := https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip
endif

all: jar

# ── Android SDK platform (android.jar) + build-tools (d8) ───

$(CMDLINE_TOOLS):
	@echo "==> Downloading Android cmdline-tools..."
	@mkdir -p $(SDK_DIR)
	$(CURL) -fsSL -o /tmp/cmdline-tools.zip "$(CMDLINE_URL)" || (echo "ERROR: Failed to download cmdline-tools" && exit 1)
	@rm -rf $(SDK_DIR)/cmdline-tools-tmp
	unzip -qo /tmp/cmdline-tools.zip -d $(SDK_DIR)/cmdline-tools-tmp || (echo "ERROR: Failed to unzip cmdline-tools" && exit 1)
	@rm -rf $(SDK_DIR)/cmdline-tools
	@mkdir -p $(SDK_DIR)/cmdline-tools
	@mv $(SDK_DIR)/cmdline-tools-tmp/cmdline-tools $(SDK_DIR)/cmdline-tools/latest
	@rm -rf $(SDK_DIR)/cmdline-tools-tmp
	@rm -f /tmp/cmdline-tools.zip
	@echo "==> cmdline-tools installed"

$(ANDROID_JAR): $(CMDLINE_TOOLS)
	@echo "==> Installing Android platform $(ANDROID_API)..."
	yes | $(CMDLINE_TOOLS) --sdk_root=$(SDK_DIR) "platforms;android-$(ANDROID_API)" 2>&1 || \
		yes | $(CMDLINE_TOOLS) --sdk_root=$(SDK_DIR) --channel=0 "platforms;android-$(ANDROID_API)" 2>&1
	@cp $(SDK_DIR)/platforms/android-$(ANDROID_API)/android.jar $@ || (echo "ERROR: Failed to copy android.jar" && exit 1)
	@echo "==> android.jar ready"

$(D8): $(CMDLINE_TOOLS)
	@echo "==> Installing build-tools..."
	yes | $(CMDLINE_TOOLS) --sdk_root=$(SDK_DIR) "build-tools;34.0.0" 2>&1 || (echo "ERROR: Failed to install build-tools" && exit 1)
	@echo "==> build-tools installed"

# ── Compile MonitorTool.java → MonitorTool.jar (DEX format) ──

STUBS_DIR      := tools/monitor-tool/stubs

jar: $(ANDROID_JAR) $(D8) $(JAR_SRC)
	@echo "==> Compiling HIDL stubs..."
	@mkdir -p build/stubs
	javac -source 8 -target 8 \
		-classpath $(ANDROID_JAR) \
		-d build/stubs \
		$(shell find $(STUBS_DIR) -name '*.java')
	@echo "==> Compiling MonitorTool.java..."
	@mkdir -p build/monitor-tool
	javac -source 8 -target 8 \
		-classpath "$(ANDROID_JAR):build/stubs" \
		-d build/monitor-tool \
		$(JAR_SRC)
	@echo "==> Converting to DEX format..."
	$(D8) --output build/monitor-tool build/monitor-tool/MonitorTool.class
	@echo "==> Packaging MonitorTool.jar..."
	@rm -f build/monitor-tool/MonitorTool.class
	jar cf $(JAR_OUT) -C build/monitor-tool .
	@rm -rf build/monitor-tool build/stubs
	@echo "==> $(JAR_OUT) ready ($$(wc -c < $(JAR_OUT) | tr -d ' ') bytes)"

# ── App shortcuts ─────────────────────────────────────────────

dev: jar
	cd mimonitor && npm install && cargo tauri dev

build: jar
	cd mimonitor && npm install && cargo tauri build

test:
	cd mimonitor/src-tauri && cargo test

# ── Clean ─────────────────────────────────────────────────────

clean:
	rm -rf build/
	rm -f $(JAR_OUT)

clean-sdk:
	rm -rf $(SDK_DIR)
