import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
import { ExternalLink } from "lucide-react";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { openUrl } from "@tauri-apps/plugin-opener";

export default function Settings({ onThemeChange }: { onThemeChange?: (theme: string) => void }) {
  const [config, setConfig] = useState<any>({});

  useEffect(() => { invoke<any>("get_config").then(setConfig); }, []);

  const updateConfig = (updates: any) => {
    invoke("update_config", { updates }).then(() => {
      invoke<any>("get_config").then((c) => {
        setConfig(c);
        if (updates.theme && onThemeChange) onThemeChange(updates.theme);
      });
    });
  };

  return (
    <div>
      <h1 className="text-sm font-medium mb-4">设置</h1>

      <div className="border-b border-border/40 pb-3 mb-3">
        <div className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground mb-2">通用</div>
        <div className="space-y-2.5">
          <div className="space-y-1">
            <span className="text-[12px] text-muted-foreground">关闭行为</span>
            <ToggleGroup type="single" value={config.close_behavior} onValueChange={(v) => v && updateConfig({ close_behavior: v })}>
              <ToggleGroupItem value="tray">最小化到托盘</ToggleGroupItem>
              <ToggleGroupItem value="exit">直接退出</ToggleGroupItem>
            </ToggleGroup>
          </div>
          <div className="space-y-1">
            <span className="text-[12px] text-muted-foreground">主题</span>
            <ToggleGroup type="single" value={config.theme} onValueChange={(v) => v && updateConfig({ theme: v })}>
              <ToggleGroupItem value="auto">跟随系统</ToggleGroupItem>
              <ToggleGroupItem value="dark">深色</ToggleGroupItem>
              <ToggleGroupItem value="light">浅色</ToggleGroupItem>
            </ToggleGroup>
          </div>
          <div className="flex items-center gap-2">
            <Switch checked={config.autostart} onCheckedChange={(v) => updateConfig({ autostart: v })} />
            <span className="text-[12px]">开机自动启动并最小化到托盘</span>
          </div>
        </div>
      </div>

      <div>
        <button onClick={() => openUrl("https://github.com/longjuan/Mimonitor_Toolbox")}
          className="inline-flex items-center gap-1.5 text-[12px] text-muted-foreground hover:text-foreground transition-colors">
          <ExternalLink className="h-3 w-3" />
          GitHub 仓库
        </button>
      </div>
    </div>
  );
}
