import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useRef, useCallback } from "react";
import { RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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
    });
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const MODES: Record<string, string> = { "4": "off", "0": "lighting", "2": "solid", "1": "ambient", "3": "cycle" };

  const handleMode = (v: string) => {
    if (!MODES[v]) return;
    setMode(v);
    // Send full command to ensure device uses current UI values
    if (v === "0") invoke("set_led_lighting", { brightness: brightness - 1, colorTemp: parseInt(colorTemp) });
    else if (v === "2") invoke("set_led_solid", { brightness: brightness - 1, color: parseInt(colorValue) });
    else invoke("set_led_mode", { mode: MODES[v] });
  };

  const handleBrightness = (vals: number[]) => {
    const v = vals[0];
    setBrightness(v);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      if (mode === "0") invoke("set_led_lighting", { brightness: v - 1, colorTemp: parseInt(colorTemp) });
      else if (mode === "2") invoke("set_led_solid", { brightness: v - 1, color: parseInt(colorValue) });
    }, 300);
  };

  const handleColorTemp = (v: string) => {
    if (!v) return;
    setColorTemp(v);
    invoke("set_led_lighting", { brightness: brightness - 1, colorTemp: parseInt(v) });
  };

  const handleColorValue = (v: string) => {
    if (!v) return;
    setColorValue(v);
    invoke("set_led_solid", { brightness: brightness - 1, color: parseInt(v) });
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold tracking-tight">屏幕灯光</h1>
        <Button variant="outline" size="sm" onClick={refresh} disabled={!loaded}>
          <RefreshCw className="h-3.5 w-3.5 mr-1.5" />刷新
        </Button>
      </div>

      <div className={cn("space-y-4", !loaded && "opacity-50 pointer-events-none")}>
        <Card>
          <CardHeader className="pb-3"><CardTitle className="text-sm font-medium">炫彩灯模式</CardTitle></CardHeader>
          <CardContent>
            <ToggleGroup type="single" value={mode} onValueChange={handleMode} disabled={!loaded}>
              <ToggleGroupItem value="4">关闭</ToggleGroupItem>
              <ToggleGroupItem value="0">照明</ToggleGroupItem>
              <ToggleGroupItem value="2">纯色</ToggleGroupItem>
              <ToggleGroupItem value="1">屏幕同色</ToggleGroupItem>
              <ToggleGroupItem value="3">七彩梦境</ToggleGroupItem>
            </ToggleGroup>
          </CardContent>
        </Card>

        {mode !== "4" && (
          <Card>
            <CardHeader className="pb-3"><CardTitle className="text-sm font-medium">亮度</CardTitle></CardHeader>
            <CardContent>
              <div className="flex items-center gap-3">
                <span className="w-16 text-sm text-muted-foreground">亮度</span>
                <Slider min={1} max={15} value={[brightness]} onValueChange={handleBrightness} className="flex-1" disabled={!loaded} />
                <span className="w-8 text-right text-sm tabular-nums">{brightness}</span>
              </div>
            </CardContent>
          </Card>
        )}

        {mode === "0" && (
          <Card>
            <CardHeader className="pb-3"><CardTitle className="text-sm font-medium">照明色温</CardTitle></CardHeader>
            <CardContent>
              <ToggleGroup type="single" value={colorTemp} onValueChange={handleColorTemp} disabled={!loaded}>
                <ToggleGroupItem value="0">2700K (暖白)</ToggleGroupItem>
                <ToggleGroupItem value="1">4000K (自然)</ToggleGroupItem>
                <ToggleGroupItem value="2">6500K (冷白)</ToggleGroupItem>
              </ToggleGroup>
            </CardContent>
          </Card>
        )}

        {mode === "2" && (
          <Card>
            <CardHeader className="pb-3"><CardTitle className="text-sm font-medium">纯色颜色</CardTitle></CardHeader>
            <CardContent>
              <ToggleGroup type="single" value={colorValue} onValueChange={handleColorValue} disabled={!loaded}>
                <ToggleGroupItem value="0">冰蓝</ToggleGroupItem>
                <ToggleGroupItem value="1">流金</ToggleGroupItem>
                <ToggleGroupItem value="2">天青</ToggleGroupItem>
                <ToggleGroupItem value="3">草地</ToggleGroupItem>
                <ToggleGroupItem value="4">日落</ToggleGroupItem>
              </ToggleGroup>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}
