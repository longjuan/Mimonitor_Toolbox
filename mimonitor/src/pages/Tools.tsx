import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
import { Terminal, Wrench, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

export default function Tools() {
  const [config, setConfig] = useState<any>({});
  const [guardianStatus, setGuardianStatus] = useState<any>(null);
  const [deploying, setDeploying] = useState(false);

  useEffect(() => { invoke<any>("get_config").then(setConfig); }, []);

  const refreshGuardian = () => invoke<any>("get_guardian_status").then(setGuardianStatus).catch(() => {});

  const updateConfig = (updates: any) => {
    invoke("update_config", { updates }).then(() => invoke<any>("get_config").then(setConfig));
  };

  const handleDeploy = async () => {
    setDeploying(true);
    try {
      await invoke("deploy_guardian");
      await refreshGuardian();
    } catch (e) {
      console.error("deploy_guardian failed:", e);
    } finally {
      setDeploying(false);
    }
  };

  return (
    <div className="space-y-4">
      <h1 className="text-xl font-semibold tracking-tight">工具设置</h1>

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">软件设置</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-1.5">
            <span className="text-sm text-muted-foreground">关闭行为</span>
            <ToggleGroup type="single" value={config.close_behavior} onValueChange={(v) => v && updateConfig({ close_behavior: v })}>
              <ToggleGroupItem value="tray">最小化到托盘</ToggleGroupItem>
              <ToggleGroupItem value="exit">直接退出</ToggleGroupItem>
            </ToggleGroup>
          </div>
          <div className="space-y-1.5">
            <span className="text-sm text-muted-foreground">主题</span>
            <ToggleGroup type="single" value={config.theme} onValueChange={(v) => v && updateConfig({ theme: v })}>
              <ToggleGroupItem value="auto">跟随系统</ToggleGroupItem>
              <ToggleGroupItem value="dark">深色</ToggleGroupItem>
              <ToggleGroupItem value="light">浅色</ToggleGroupItem>
            </ToggleGroup>
          </div>
          <div className="flex items-center gap-2">
            <Switch checked={config.autostart} onCheckedChange={(v) => updateConfig({ autostart: v })} />
            <span className="text-sm">开机自动启动并最小化到托盘</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">ADB Shell</CardTitle>
        </CardHeader>
        <CardContent>
          <Button variant="outline" onClick={() => invoke("open_adb_shell")}>
            <Terminal className="h-4 w-4 mr-1.5" />
            打开 ADB 终端
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">ADB Guardian</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div>
            {guardianStatus?.ok ? (
              <span className="text-sm text-green-600 dark:text-green-400">正常运行</span>
            ) : guardianStatus ? (
              <span className="text-sm text-red-500">需要修复</span>
            ) : (
              <span className="text-sm text-muted-foreground">未检测</span>
            )}
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={refreshGuardian}>
              <Wrench className="h-3.5 w-3.5 mr-1.5" />
              检测状态
            </Button>
            <Button size="sm" onClick={handleDeploy} disabled={deploying}>
              {deploying ? "部署中..." : "部署/修复"}
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className="pt-6">
          <a href="https://github.com/YiHooong/Mimonitor_Toolbox" target="_blank" rel="noopener noreferrer"
            className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
            <ExternalLink className="h-3.5 w-3.5" />
            GitHub 仓库
          </a>
        </CardContent>
      </Card>
    </div>
  );
}
