import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Home, Monitor, Gamepad2, Radio, Lightbulb, Smartphone, Keyboard, Settings,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import HomeP from "@/pages/Home";
import Picture from "@/pages/Picture";
import Game from "@/pages/Game";
import Source from "@/pages/Source";
import Light from "@/pages/Light";
import Remote from "@/pages/Remote";
import Hotkey from "@/pages/Hotkey";
import Tools from "@/pages/Tools";

const NAV_ITEMS = [
  { id: "home", label: "首页", icon: Home },
  { id: "picture", label: "画面", icon: Monitor },
  { id: "game", label: "游戏", icon: Gamepad2 },
  { id: "source", label: "信号源", icon: Radio },
  { id: "light", label: "灯光", icon: Lightbulb },
  { id: "remote", label: "遥控器", icon: Smartphone },
  { id: "hotkey", label: "热键", icon: Keyboard },
  { id: "tools", label: "工具", icon: Settings },
];

export default function App() {
  const [currentPage, setCurrentPage] = useState("home");
  const [connected, setConnected] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const logRef = useRef<HTMLDivElement>(null);

  const addLog = useCallback((msg: string) => {
    const time = new Date().toLocaleTimeString("zh-CN", { hour12: false });
    setLogs((prev) => [...prev, `[${time}] ${msg}`]);
  }, []);

  useEffect(() => {
    if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight;
  }, [logs]);

  useEffect(() => {
    const checkStatus = async () => {
      try {
        const status = await invoke<{ connected: boolean }>("get_connection_status");
        setConnected(status.connected);
      } catch {}
    };
    checkStatus();
    const interval = setInterval(checkStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const renderPage = () => {
    switch (currentPage) {
      case "home": return <HomeP onConnectChange={setConnected} logs={logs} addLog={addLog} logRef={logRef} />;
      case "picture": return <Picture />;
      case "game": return <Game />;
      case "source": return <Source />;
      case "light": return <Light />;
      case "remote": return <Remote />;
      case "hotkey": return <Hotkey />;
      case "tools": return <Tools />;
      default: return <HomeP onConnectChange={setConnected} logs={logs} addLog={addLog} logRef={logRef} />;
    }
  };

  return (
    <div className="flex h-screen bg-background text-foreground">
      <nav className="w-56 shrink-0 border-r bg-card/50 backdrop-blur-sm flex flex-col">
        <div className="flex items-center gap-2.5 px-5 py-4 border-b">
          <div className="w-7 h-7 rounded-lg bg-primary flex items-center justify-center">
            <span className="text-xs font-bold text-primary-foreground">M</span>
          </div>
          <span className="font-semibold text-sm tracking-tight">Mimonitor</span>
          <div className={cn("ml-auto w-2 h-2 rounded-full", connected ? "bg-green-500" : "bg-red-400")} />
        </div>
        <div className="flex-1 px-3 py-2 space-y-0.5">
          {NAV_ITEMS.map((item) => {
            const Icon = item.icon;
            const disabled = !connected && item.id !== "home";
            return (
              <Button key={item.id} variant={currentPage === item.id ? "secondary" : "ghost"}
                className={cn("w-full justify-start gap-2.5 h-9 font-normal text-[13px]", currentPage === item.id && "bg-accent font-medium")}
                onClick={() => { if (!disabled) setCurrentPage(item.id); }} disabled={disabled}>
                <Icon className="h-4 w-4" />{item.label}
              </Button>
            );
          })}
        </div>
      </nav>
      <main className="flex-1 overflow-y-auto">
        <div className="max-w-3xl mx-auto px-6 py-6">{renderPage()}</div>
      </main>
    </div>
  );
}
