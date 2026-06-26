/// MCP (Model Context Protocol) tool definitions for AI integration
/// These map to the same socket RPC methods, callable via CLI or MCP stdio.

/// MCP tool definitions — used by CLI `--mcp` mode
pub fn get_tools() -> serde_json::Value {
    serde_json::json!({
        "tools": [
            {
                "name": "scan",
                "description": "扫描内网查找显示器设备",
                "inputSchema": {"type": "object", "properties": {}}
            },
            {
                "name": "connect",
                "description": "连接到显示器",
                "inputSchema": {
                    "type": "object",
                    "properties": {"ip": {"type": "string"}},
                    "required": ["ip"]
                }
            },
            {
                "name": "get_status",
                "description": "获取连接状态",
                "inputSchema": {"type": "object", "properties": {}}
            },
            {
                "name": "set_picture_mode",
                "description": "切换画面模式 (standard/game/movie)",
                "inputSchema": {
                    "type": "object",
                    "properties": {"mode": {"type": "string", "enum": ["standard", "game", "movie"]}},
                    "required": ["mode"]
                }
            },
            {
                "name": "set_picture_setting",
                "description": "设置画面参数 (背光/对比度/饱和度等, 0-100)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "key": {"type": "string", "enum": ["backlight", "brightness", "contrast", "saturation", "hue", "sharpness"]},
                        "value": {"type": "integer", "minimum": 0, "maximum": 100}
                    },
                    "required": ["key", "value"]
                }
            },
            {
                "name": "set_local_dimming",
                "description": "设置精密控光等级",
                "inputSchema": {
                    "type": "object",
                    "properties": {"level": {"type": "string", "enum": ["off", "low", "medium", "high"]}},
                    "required": ["level"]
                }
            },
            {
                "name": "set_led",
                "description": "控制显示器 RGB LED 灯光",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "mode": {"type": "string", "enum": ["off", "lighting", "solid", "ambient", "cycle"]},
                        "brightness": {"type": "integer", "minimum": 1, "maximum": 15, "description": "亮度 1-15"},
                        "color_temp": {"type": "string", "enum": ["2700K", "4000K", "6500K"], "description": "照明模式色温"},
                        "color": {"type": "string", "enum": ["ice-blue", "gold", "azure", "grass", "sunset"], "description": "纯色模式颜色"}
                    },
                    "required": ["mode"]
                }
            },
            {
                "name": "set_input_source",
                "description": "切换输入源",
                "inputSchema": {
                    "type": "object",
                    "properties": {"source": {"type": "string", "enum": ["hdmi1", "hdmi2", "dp", "usbc"]}},
                    "required": ["source"]
                }
            },
            {
                "name": "send_remote_key",
                "description": "发送遥控器按键",
                "inputSchema": {
                    "type": "object",
                    "properties": {"key": {"type": "string", "enum": ["power", "home", "menu", "back", "up", "down", "left", "right", "ok", "volume_up", "volume_down", "mute"]}},
                    "required": ["key"]
                }
            }
        ]
    })
}
