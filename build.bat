@echo off
python -m pip install -r requirements-build.txt
python -m PyInstaller --clean --noconfirm MonitorToolbox.spec
echo Done! Check dist\MonitorToolbox.exe
pause
