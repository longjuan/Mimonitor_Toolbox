import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useRef, RefObject } from "react";
import { Wifi, WifiOff, Search, PlugZap, Unplug } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface HomeProps {
  onConnectChange: (connected: boolean) => void;
  logs: string[];
  addLog: (msg: string) => void;
  logRef: RefObject<HTMLDivElement | null>;
}

export default function Home({ onConnectChange, logs, addLog, logRef }: HomeProps) {
  const [ip, setIp] = useState("");
  const [status, setStatus] = useState<"disconnected" | "scanning" | "connected">("disconnected");
  const [model, setModel] = useState("");
  const [devices, setDevices] = useState<{ ip: string; model: string }[]>([]);
  const scannedRef = useRef(false);

  useEffect(() => {
    invoke<{ saved_ip: string }>("get_config").then((config) => {
      if (config.saved_ip) setIp(config.saved_ip);
    });
    invoke<{ connected: boolean; ip: string; model: string }>("get_connection_status").then((s) => {
      if (s.connected) {
        setStatus("connected");
        setModel(s.model);
        setIp(s.ip);
        onConnectChange(true);
      } else if (!scannedRef.current) {
        scannedRef.current = true;
        handleScan();
      }
    });
  }, []);

  const handleConnect = async () => {
    if (!ip) return;
    setStatus("scanning");
    addLog(`正在连接 ${ip}...`);
    try {
      const ok = await invoke<boolean>("connect", { ip });
      if (ok) {
        setStatus("connected");
        onConnectChange(true);
        const s = await invoke<{ connected: boolean; model: string }>("get_connection_status");
        setModel(s.model);
        addLog(`已连接: ${s.model} (${ip})`);
      }
    } catch (e: any) {
      setStatus("disconnected");
      onConnectChange(false);
      addLog(`连接失败: ${e}`);
    }
  };

  const handleDisconnect = async () => {
    await invoke("disconnect");
    setStatus("disconnected");
    setModel("");
    onConnectChange(false);
    addLog("已断开连接");
  };

  const handleScan = async () => {
    setStatus("scanning");
    addLog("正在扫描内网...");
    try {
      const result = await invoke<{ devices: { ip: string; model: string }[]; subnets: string[] }>("scan_network");
      setDevices(result.devices);
      for (const subnet of result.subnets) {
        addLog(`扫描网段: ${subnet}.1-254`);
      }
      if (result.devices.length > 0) {
        addLog(`发现 ${result.devices.length} 个设备`);
        for (const d of result.devices) {
          addLog(`  → ${d.model} (${d.ip})`);
        }
        const mitv = result.devices.find((d) => d.model.toLowerCase().includes("mitv"));
        if (mitv) {
          setIp(mitv.ip);
          addLog(`自动选择: ${mitv.model} (${mitv.ip})`);
        }
        // If only one device found, connect directly
        if (result.devices.length === 1) {
          const d = result.devices[0];
          setIp(d.ip);
          addLog(`仅发现一台设备，自动连接: ${d.model} (${d.ip})`);
          setStatus("scanning");
          try {
            const ok = await invoke<boolean>("connect", { ip: d.ip });
            if (ok) {
              setStatus("connected");
              setModel(d.model);
              onConnectChange(true);
              addLog(`已连接: ${d.model} (${d.ip})`);
              return;
            }
          } catch (e: any) {
            addLog(`自动连接失败: ${e}`);
          }
        }
      } else {
        addLog("未发现设备");
      }
    } catch (e: any) {
      addLog(`扫描失败: ${e}`);
    }
    setStatus("disconnected");
  };

  return (
    <div className="space-y-4">
      <h1 className="text-xl font-semibold tracking-tight">红米 G Pro 27U Toolbox</h1>

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">连接</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex gap-2">
            <Input placeholder="显示器 IP 地址" value={ip} onChange={(e) => setIp(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleConnect()} className="w-60" />
            {status === "connected" ? (
              <Button variant="outline" onClick={handleDisconnect}><Unplug className="h-4 w-4 mr-1.5" />断开</Button>
            ) : (
              <Button onClick={handleConnect} disabled={status === "scanning"}>
                <PlugZap className="h-4 w-4 mr-1.5" />{status === "scanning" ? "连接中..." : "连接"}
              </Button>
            )}
            <Button variant="outline" onClick={handleScan} disabled={status === "scanning"}>
              <Search className="h-4 w-4 mr-1.5" />扫描
            </Button>
          </div>
          <div className="flex items-center gap-2 text-sm">
            {status === "connected" ? (
              <><Wifi className="h-3.5 w-3.5 text-green-500" /><span className="text-green-600 dark:text-green-400">已连接 {model}</span></>
            ) : status === "scanning" ? (
              <><Search className="h-3.5 w-3.5 text-amber-500 animate-pulse" /><span className="text-amber-600 dark:text-amber-400">扫描中...</span></>
            ) : (
              <><WifiOff className="h-3.5 w-3.5 text-muted-foreground" /><span className="text-muted-foreground">未连接</span></>
            )}
          </div>
          {devices.length > 0 && (
            <select className="w-60 h-9 rounded-md border bg-transparent px-3 text-sm" value={ip} onChange={(e) => setIp(e.target.value)}>
              {devices.map((d) => (<option key={d.ip} value={d.ip}>{d.model} ({d.ip})</option>))}
            </select>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">日志</CardTitle>
        </CardHeader>
        <CardContent>
          <div ref={logRef as React.RefObject<HTMLDivElement>} className="h-52 overflow-y-auto rounded-md bg-muted/50 p-3 font-mono text-xs text-muted-foreground whitespace-pre-wrap">
            {logs.length === 0 ? "等待操作..." : logs.join("\n")}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
