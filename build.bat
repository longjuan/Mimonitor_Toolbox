@echo off
pip install pyinstaller pyqt6 PyQt6-Fluent-Widgets
pyinstaller --onefile --windowed --name "MonitorToolbox" --icon=assets\app\icon.ico --hidden-import qfluentwidgets --add-binary "assets\runtime\adb.exe;assets\runtime" --add-binary "assets\runtime\AdbWinApi.dll;assets\runtime" --add-binary "assets\runtime\AdbWinUsbApi.dll;assets\runtime" --add-binary "assets\runtime\MtkDirectTool.jar;assets\runtime" --add-binary "assets\runtime\ColorfulLedTool.jar;assets\runtime" --add-binary "assets\adb_guardian\adbguardian-signed.apk;assets\adb_guardian" monitor_controller.py
echo Done! Check dist\MonitorToolbox.exe
pause
