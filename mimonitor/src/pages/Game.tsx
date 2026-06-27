import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useCallback } from "react";
import { RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

interface ToggleRowProps {
  label: string;
  options: { value: string; label: string }[];
  value: string;
  onChange: (val: string) => void;
  disabled?: boolean;
}

function ToggleRow({ label, options, value, onChange, disabled }: ToggleRowProps) {
  return (
    <div className="space-y-1">
      <span className="text-xs text-muted-foreground">{label}</span>
      <ToggleGroup type="single" value={value} onValueChange={(v) => v && onChange(v)} disabled={disabled}>
        {options.map((opt) => (
          <ToggleGroupItem key={opt.value} value={opt.value}>{opt.label}</ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}

export default function Game() {
  const [crosshair, setCrosshair] = useState("0");
  const [dynamicFt, setDynamicFt] = useState("0");
  const [scope, setScope] = useState("0");
  const [scopeNight, setScopeNight] = useState("0");
  const [fpsCounter, setFpsCounter] = useState("0");
  const [stopwatch, setStopwatch] = useState("0");
  const [timer, setTimer] = useState("0");
  const [loaded, setLoaded] = useState(false);

  const refresh = useCallback(() => {
    invoke<Record<string, string>>("get_game_settings").then((s) => {
      setCrosshair(s["front_sight_index"] || "0");
      setDynamicFt(s["mt_game_dynamic_ft"] || "0");
      setScope(s["mt_game_scope"] || "0");
      setScopeNight(s["mt_game_scope_night"] || "0");
      setFpsCounter(s["monitor_menu_fps_counter"] || "0");
      setStopwatch(s["monitor_menu_stopwatch"] || "0");
      setTimer(s["monitor_menu_timer"] || "0");
      setLoaded(true);
    });
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const handleChange = (setter: (v: string) => void, command: string) => (v: string) => {
    setter(v);
    invoke(command, { value: parseInt(v) }).then(() => setTimeout(refresh, 200));
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-sm font-medium mb-4">游戏模式</h1>
        <Button variant="outline" size="sm" onClick={refresh} disabled={!loaded}>
          <RefreshCw className="h-3 w-3 mr-1" />刷新
        </Button>
      </div>

      <div className={cn(!loaded && "opacity-50 pointer-events-none")}>
        <div className="text-xs font-medium text-muted-foreground mb-2">游戏功能</div>
        <div className="space-y-2">
          <ToggleRow label="准星" value={crosshair} onChange={handleChange(setCrosshair, "set_crosshair")} disabled={!loaded} options={[
            { value: "0", label: "关" }, { value: "1", label: "1" }, { value: "2", label: "2" }, { value: "3", label: "3" }, { value: "4", label: "4" }, { value: "5", label: "5" },
          ]} />
          <ToggleRow label="动态准星" value={dynamicFt} onChange={handleChange(setDynamicFt, "set_dynamic_crosshair")} disabled={!loaded} options={[
            { value: "0", label: "关" }, { value: "1", label: "开" },
          ]} />
          <ToggleRow label="狙击镜" value={scope} onChange={handleChange(setScope, "set_sniper_scope")} disabled={!loaded} options={[
            { value: "0", label: "关" }, { value: "1", label: "1.1x" }, { value: "3", label: "1.3x" }, { value: "5", label: "1.5x" }, { value: "7", label: "1.7x" }, { value: "10", label: "2.0x" },
          ]} />
          <ToggleRow label="夜视" value={scopeNight} onChange={handleChange(setScopeNight, "set_scope_night_vision")} disabled={!loaded} options={[
            { value: "0", label: "关" }, { value: "1", label: "开" },
          ]} />
          <ToggleRow label="FPS 计数器" value={fpsCounter} onChange={handleChange(setFpsCounter, "set_fps_counter")} disabled={!loaded} options={[
            { value: "0", label: "关" }, { value: "1", label: "刷新率" }, { value: "2", label: "柱状图" },
          ]} />
          <ToggleRow label="秒表" value={stopwatch} onChange={handleChange(setStopwatch, "set_stopwatch")} disabled={!loaded} options={[
            { value: "0", label: "关" }, { value: "1", label: "开" },
          ]} />
          <ToggleRow label="定时器" value={timer} onChange={handleChange(setTimer, "set_timer")} disabled={!loaded} options={[
            { value: "0", label: "关" }, { value: "60", label: "1分钟" }, { value: "300", label: "5分钟" }, { value: "1800", label: "30分钟" }, { value: "3600", label: "60分钟" },
          ]} />
        </div>
      </div>
    </div>
  );
}
