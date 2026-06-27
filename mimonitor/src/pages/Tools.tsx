import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { Terminal, Wrench } from "lucide-react";
import { Button } from "@/components/ui/button";

export default function Tools() {
  const [guardianStatus, setGuardianStatus] = useState<any>(null);
  const [deploying, setDeploying] = useState(false);

  const refreshGuardian = () => invoke<any>("get_guardian_status").then(setGuardianStatus).catch(() => {});

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
    <div>
      <h1 className="text-sm font-medium mb-4">工具</h1>

      <div className="border-b border-border/40 pb-3 mb-3">
        <div className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground mb-2">ADB</div>
        <Button variant="outline" onClick={() => invoke("open_adb_shell")}>
          <Terminal className="h-3.5 w-3.5 mr-1" />
          打开 ADB 终端
        </Button>
      </div>

      <div>
        <div className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground mb-2">ADB Guardian</div>
        <div className="space-y-2">
          <div>
            {guardianStatus?.ok ? (
              <span className="text-[12px] text-green-600 dark:text-green-400">正常运行</span>
            ) : guardianStatus ? (
              <span className="text-[12px] text-red-500">需要修复</span>
            ) : (
              <span className="text-[12px] text-muted-foreground">未检测</span>
            )}
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={refreshGuardian}>
              <Wrench className="h-3 w-3 mr-1" />
              检测状态
            </Button>
            <Button size="sm" onClick={handleDeploy} disabled={deploying}>
              {deploying ? "部署中..." : "部署/修复"}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
