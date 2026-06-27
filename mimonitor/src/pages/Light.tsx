import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useRef, useCallback } from "react";
import { RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

export default function Light() {
  const [mode, setMode] = useState("0");
  const [brightness, setBrightness] = useState(10);
  const [colorTemp, setColorTemp] = useState("0");
  const [colorValue, setColorValue] = useState("0");
  const [loaded, setLoaded] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout>>();

  const refresh = useCallback(() => {
    invoke<Record<string, string>>("get_light_settings").then((s) => {
      setMode(s["atmosphere_light_switcher_pm2"] || "0");
      const b = parseInt(s["atmosphere_light_illumination"] || "9");
      if (!isNaN(b)) setBrightness(b + 1);
      setColorTemp(s["atmosphere_light_color_temp"] || "0");
      setColorValue(s["atmosphere_light_color_value"] || "0");
      setLoaded(true);
    }).catch((e) => {
      console.error("get_light_settings failed:", e);
      setLoaded(true);
    });
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const MODES: Record<string, string> = { "4": "off", "0": "lighting", "2": "solid", "1": "ambient", "3": "cycle" };

  const handleMode = (v: string) => {
    if (!MODES[v]) return;
    setMode(v);
    if (v === "0") invoke("set_led_lighting", { brightness: brightness - 1, colorTemp: parseInt(colorTemp) }).catch((e) => console.error("set_led_lighting failed:", e));
    else if (v === "2") invoke("set_led_solid", { brightness: brightness - 1, color: parseInt(colorValue) }).catch((e) => console.error("set_led_solid failed:", e));
    else invoke("set_led_mode", { mode: MODES[v] }).catch((e) => console.error("set_led_mode failed:", e));
  };

  const handleBrightness = (vals: number[]) => {
    const v = vals[0];
    setBrightness(v);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      if (mode === "0") invoke("set_led_lighting", { brightness: v - 1, colorTemp: parseInt(colorTemp) }).catch((e) => console.error("set_led_lighting failed:", e));
      else if (mode === "2") invoke("set_led_solid", { brightness: v - 1, color: parseInt(colorValue) }).catch((e) => console.error("set_led_solid failed:", e));
    }, 300);
  };

  const handleColorTemp = (v: string) => {
    if (!v) return;
    setColorTemp(v);
    invoke("set_led_lighting", { brightness: brightness - 1, colorTemp: parseInt(v) }).catch((e) => console.error("set_led_lighting failed:", e));
  };

  const handleColorValue = (v: string) => {
    if (!v) return;
    setColorValue(v);
    invoke("set_led_solid", { brightness: brightness - 1, color: parseInt(v) }).catch((e) => console.error("set_led_solid failed:", e));
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-sm font-medium mb-4">屏幕灯光</h1>
        <Button variant="outline" size="sm" onClick={refresh} disabled={!loaded}>
          <RefreshCw className="h-3 w-3 mr-1" />刷新
        </Button>
      </div>

      <div className={cn(!loaded && "opacity-50 pointer-events-none")}>
        <div className="border-b pb-3 mb-3">
          <div className="text-xs font-medium text-muted-foreground mb-2">炫彩灯模式</div>
          <ToggleGroup type="single" value={mode} onValueChange={handleMode} disabled={!loaded}>
            <ToggleGroupItem value="4">关闭</ToggleGroupItem>
            <ToggleGroupItem value="0">照明</ToggleGroupItem>
            <ToggleGroupItem value="2">纯色</ToggleGroupItem>
            <ToggleGroupItem value="1">屏幕同色</ToggleGroupItem>
            <ToggleGroupItem value="3">七彩梦境</ToggleGroupItem>
          </ToggleGroup>
        </div>

        {mode !== "4" && (
          <div className="border-b pb-3 mb-3">
            <div className="text-xs font-medium text-muted-foreground mb-2">亮度</div>
            <div className="flex items-center gap-2">
              <span className="w-12 text-xs text-muted-foreground">亮度</span>
              <Slider min={1} max={15} value={[brightness]} onValueChange={handleBrightness} className="flex-1" disabled={!loaded} />
              <span className="w-6 text-right text-xs tabular-nums">{brightness}</span>
            </div>
          </div>
        )}

        {mode === "0" && (
          <div className="border-b pb-3 mb-3">
            <div className="text-xs font-medium text-muted-foreground mb-2">照明色温</div>
            <ToggleGroup type="single" value={colorTemp} onValueChange={handleColorTemp} disabled={!loaded}>
              <ToggleGroupItem value="0">2700K (暖白)</ToggleGroupItem>
              <ToggleGroupItem value="1">4000K (自然)</ToggleGroupItem>
              <ToggleGroupItem value="2">6500K (冷白)</ToggleGroupItem>
            </ToggleGroup>
          </div>
        )}

        {mode === "2" && (
          <div>
            <div className="text-xs font-medium text-muted-foreground mb-2">纯色颜色</div>
            <ToggleGroup type="single" value={colorValue} onValueChange={handleColorValue} disabled={!loaded}>
              <ToggleGroupItem value="0">冰蓝</ToggleGroupItem>
              <ToggleGroupItem value="1">流金</ToggleGroupItem>
              <ToggleGroupItem value="2">天青</ToggleGroupItem>
              <ToggleGroupItem value="3">草地</ToggleGroupItem>
              <ToggleGroupItem value="4">日落</ToggleGroupItem>
            </ToggleGroup>
          </div>
        )}
      </div>
    </div>
  );
}
