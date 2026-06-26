# MonitorTool.jar

显示器硬件控制工具，提供 MTK JNI 寄存器读写和 LED 灯光控制。

## 构建

需要 Android SDK 的 `android.jar`：

```bash
export ANDROID_SDK=~/Library/Android/sdk/platforms/android-34/android.jar
cd tools/merged
./build.sh
```

输出: `assets/runtime/MonitorTool.jar`

## 命令

```bash
# MTK JNI 操作
MonitorTool get <key>                    # 读取配置值
MonitorTool set <key> <value> [isUpdate] # 写入配置值
MonitorTool getMinMax <key>              # 查询范围 [min, max]
MonitorTool setColorGains <r> <g> <b>    # 设置 RGB 增益
MonitorTool dump                         # 批量读取所有画面参数

# LED 控制
MonitorTool led off                      # 关闭
MonitorTool led ambient                  # 屏幕同色
MonitorTool led cycle                    # 七彩循环
MonitorTool led lighting [brightness] [colorTemp]  # 照明 (0-14, 0=2700K/1=4000K/2=6500K)
MonitorTool led solid [brightness] [color]         # 纯色 (0=冰蓝/1=流金/2=天青/3=草地/4=日落)
MonitorTool led raw <mode> <color>       # 原始模式
```
