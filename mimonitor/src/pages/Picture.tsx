import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useRef, useCallback } from "react";
import { RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Slider } from "@/components/ui/slider";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

interface SliderRowProps {
  label: string;
  settingKey: string;
  min: number;
  max: number;
  step?: number;
  disabled?: boolean;
  settings: Record<string, string>;
}

function SliderRow({ label, settingKey, min, max, step = 1, disabled, settings }: SliderRowProps) {
  const [value, setValue] = useState(min);
  const timerRef = useRef<ReturnType<typeof setTimeout>>();

  useEffect(() => {
    const v = parseInt(settings[settingKey] || String(min));
    if (!isNaN(v)) setValue(v);
  }, [settings, settingKey, min]);

  const handleChange = (vals: number[]) => {
    const v = vals[0];
    setValue(v);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => { invoke("set_picture_slider", { key: settingKey, value: v }); }, 300);
  };

  return (
    <div className="flex items-center gap-3">
      <span className="w-20 text-sm text-muted-foreground">{label}</span>
      <Slider min={min} max={max} step={step} value={[value]} onValueChange={handleChange} className="flex-1" disabled={disabled} />
      <span className="w-10 text-right text-sm tabular-nums">{value}</span>
    </div>
  );
}

interface ButtonGroupProps {
  label: string;
  options: { value: string; label: string }[];
  value: string;
  onChange: (val: string) => void;
  disabled?: boolean;
}

function ButtonGroup({ label, options, value, onChange, disabled }: ButtonGroupProps) {
  return (
    <div className="space-y-1.5">
      <span className="text-sm text-muted-foreground">{label}</span>
      <ToggleGroup type="single" value={value} onValueChange={(v) => v && onChange(v)} disabled={disabled}>
        {options.map((opt) => (
          <ToggleGroupItem key={opt.value} value={opt.value}>{opt.label}</ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}

export default function Picture() {
  const [loading, setLoading] = useState(true);
  const [pictureMode, setPictureMode] = useState("14");
  const [colorTemp, setColorTemp] = useState("1");
  const [localDimming, setLocalDimming] = useState("0");
  const [dynamicDef, setDynamicDef] = useState("0");
  const [responseTime, setResponseTime] = useState("1");
  const [colorSpace, setColorSpace] = useState("0");
  const [hdrToneMapping, setHdrToneMapping] = useState("0");
  const [settings, setSettings] = useState<Record<string, string>>({});

  // HDR picture modes that support tone mapping
  const HDR_MODES = new Set([11,12,13,15,16,17,18,19,22,23,29,30,31,32,33,39,40,41,42,43,44]);
  const showHdrToneMapping = HDR_MODES.has(parseInt(pictureMode));

  const refresh = useCallback(() => {
    setLoading(true);
    invoke<Record<string, string>>("get_picture_settings").then((s) => {
      setSettings(s);
      setPictureMode(s["picture_mode"] || "14");
      setColorTemp(s["picture_color_temperature"] || "1");
      setLocalDimming(s["picture_local_dimming"] || "0");
      setDynamicDef(s["picture_dynamic_definition"] || "0");
      setResponseTime(s["picture_response_time"] || "1");
      setColorSpace(s["tv_picture_advanced_video_color_space"] || "0");
      setHdrToneMapping(s["settings_display_hdr_color_tone"] || "0");
      setLoading(false);
    }).catch((e) => {
      console.error("get_picture_settings failed:", e);
      setLoading(false);
    });
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const showColorGains = colorTemp === "3";
  const handleChange = (setter: (v: string) => void, command: string) => (v: string) => {
    setter(v);
    invoke(command, { value: parseInt(v) }).then(() => setTimeout(refresh, 200));
  };

  const disabled = loading;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold tracking-tight">画面设置</h1>
        <Button variant="outline" size="sm" onClick={refresh} disabled={loading}>
          <RefreshCw className="h-3.5 w-3.5 mr-1.5" />刷新
        </Button>
      </div>

      <div className={cn("relative space-y-4", loading && "opacity-50 pointer-events-none")}>
        <Card>
          <CardHeader className="pb-3"><CardTitle className="text-sm font-medium">画面模式</CardTitle></CardHeader>
          <CardContent>
            <ToggleGroup type="single" value={pictureMode} onValueChange={(v) => {
              if (v) { setPictureMode(v); invoke("set_picture_mode", { mode: parseInt(v) }).then(() => setTimeout(refresh, 200)); }
            }} disabled={disabled}>
              <ToggleGroupItem value="14">标准</ToggleGroupItem>
              <ToggleGroupItem value="10">游戏</ToggleGroupItem>
              <ToggleGroupItem value="9">电影</ToggleGroupItem>
              <Button variant="outline" size="sm" className="ml-2" onClick={() => invoke("reset_picture_mode").then(refresh)} disabled={disabled}>恢复默认</Button>
            </ToggleGroup>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3"><CardTitle className="text-sm font-medium">画面参数</CardTitle></CardHeader>
          <CardContent className="space-y-4">
            <SliderRow label="背光" settingKey="picture_backlight" min={1} max={100} disabled={disabled} settings={settings} />
            <SliderRow label="黑色级别" settingKey="picture_brightness" min={0} max={100} disabled={disabled} settings={settings} />
            <SliderRow label="对比度" settingKey="picture_contrast" min={0} max={100} disabled={disabled} settings={settings} />
            <SliderRow label="饱和度" settingKey="picture_saturation" min={0} max={100} disabled={disabled} settings={settings} />
            <SliderRow label="色调" settingKey="picture_hue" min={0} max={100} disabled={disabled} settings={settings} />
            <SliderRow label="锐度" settingKey="picture_sharpness" min={0} max={100} disabled={disabled} settings={settings} />
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3"><CardTitle className="text-sm font-medium">高级设置</CardTitle></CardHeader>
          <CardContent className="space-y-4">
            <ButtonGroup label="色温" value={colorTemp} onChange={handleChange(setColorTemp, "set_color_temperature")} disabled={disabled} options={[
              { value: "0", label: "冷色" }, { value: "1", label: "标准" }, { value: "2", label: "暖色" }, { value: "8", label: "原色" }, { value: "3", label: "自定义" },
            ]} />
            <ButtonGroup label="精密控光" value={localDimming} onChange={handleChange(setLocalDimming, "set_local_dimming")} disabled={disabled} options={[
              { value: "0", label: "关" }, { value: "1", label: "低" }, { value: "2", label: "中" }, { value: "3", label: "高" },
            ]} />
            {showHdrToneMapping && (
              <ButtonGroup label="HDR 色调映射" value={hdrToneMapping} onChange={handleChange(setHdrToneMapping, "set_hdr_tone_mapping")} disabled={disabled} options={[
                { value: "0", label: "HGiG" }, { value: "1", label: "层次" }, { value: "2", label: "动态" }, { value: "3", label: "明亮" },
              ]} />
            )}
            <ButtonGroup label="动态清晰度" value={dynamicDef} onChange={handleChange(setDynamicDef, "set_dynamic_definition")} disabled={disabled} options={[
              { value: "0", label: "关" }, { value: "1", label: "低" }, { value: "2", label: "中" }, { value: "3", label: "高" },
            ]} />
            <ButtonGroup label="响应时间" value={responseTime} onChange={handleChange(setResponseTime, "set_response_time")} disabled={disabled} options={[
              { value: "1", label: "普通" }, { value: "2", label: "快速" }, { value: "3", label: "高速" },
            ]} />
            <ButtonGroup label="色域" value={colorSpace} onChange={handleChange(setColorSpace, "set_color_space")} disabled={disabled} options={[
              { value: "0", label: "自动" }, { value: "3", label: "sRGB" }, { value: "6", label: "DCI-P3" }, { value: "4", label: "AdobeRGB" }, { value: "5", label: "BT2020" }, { value: "7", label: "BT709" },
            ]} />
          </CardContent>
        </Card>

        {showColorGains && (
          <Card>
            <CardHeader className="pb-3"><CardTitle className="text-sm font-medium">RGB 增益</CardTitle></CardHeader>
            <CardContent className="space-y-4">
              <SliderRow label="红色" settingKey="picture_red_gain" min={524} max={1524} disabled={disabled} settings={settings} />
              <SliderRow label="绿色" settingKey="picture_green_gain" min={524} max={1524} disabled={disabled} settings={settings} />
              <SliderRow label="蓝色" settingKey="picture_blue_gain" min={524} max={1524} disabled={disabled} settings={settings} />
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}
