import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Home, Monitor, Gamepad2, Radio, Lightbulb, Smartphone, Keyboard, Wrench, Settings,
  Minus, Square, X,
} from "lucide-react";
import { cn } from "@/lib/utils";
import HomeP from "@/pages/Home";
import Picture from "@/pages/Picture";
import Game from "@/pages/Game";
import Source from "@/pages/Source";
import Light from "@/pages/Light";
import Remote from "@/pages/Remote";
import Hotkey from "@/pages/Hotkey";
import Tools from "@/pages/Tools";
import SettingsP from "@/pages/Settings";

// Items that require ADB connection
const NEEDS_CONNECTION = new Set(["picture", "game", "source", "light", "remote", "hotkey", "tools"]);

const NAV_ITEMS = [
  { id: "home", label: "首页", icon: Home },
  { id: "picture", label: "画面", icon: Monitor },
  { id: "game", label: "游戏", icon: Gamepad2 },
  { id: "source", label: "信号源", icon: Radio },
  { id: "light", label: "灯光", icon: Lightbulb },
  { id: "remote", label: "遥控器", icon: Smartphone },
  { id: "hotkey", label: "热键", icon: Keyboard },
  { id: "tools", label: "工具", icon: Wrench },
  { id: "settings", label: "设置", icon: Settings },
];

const isWindows = navigator.platform.toLowerCase().includes("win");

export default function App() {
  const [currentPage, setCurrentPage] = useState("home");
  const [connected, setConnected] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const logRef = useRef<HTMLDivElement>(null);

  // Apply theme on mount
  useEffect(() => {
    invoke<{ theme?: string }>("get_config").then((config) => {
      applyTheme(config.theme || "auto");
    });
  }, []);

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

  const applyTheme = (theme: string) => {
    const root = document.documentElement;
    root.classList.remove("dark");
    if (theme === "dark") {
      root.classList.add("dark");
    } else if (theme === "auto") {
      if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
        root.classList.add("dark");
      }
    }
  };

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
      case "settings": return <SettingsP onThemeChange={applyTheme} />;
      default: return <HomeP onConnectChange={setConnected} logs={logs} addLog={addLog} logRef={logRef} />;
    }
  };

  return (
    <div className="flex h-screen bg-background text-foreground">
      {/* Sidebar */}
      <nav className="w-60 shrink-0 border-r border-border/40 flex flex-col">
        {/* Title bar area: traffic lights (macOS) or window controls (Windows) */}
        <div className="h-8 shrink-0 flex items-center justify-end px-2"
          onMouseDown={(e) => { if (!(e.target as HTMLElement).closest("button")) getCurrentWindow().startDragging(); }}>
          {isWindows && (
            <div className="flex items-center -mr-1">
              <button className="w-11 h-8 flex items-center justify-center text-muted-foreground hover:bg-accent/50 hover:text-foreground transition-colors"
                onClick={() => getCurrentWindow().minimize()}>
                <Minus className="h-4 w-4" />
              </button>
              <button className="w-11 h-8 flex items-center justify-center text-muted-foreground hover:bg-accent/50 hover:text-foreground transition-colors"
                onClick={() => getCurrentWindow().toggleMaximize()}>
                <Square className="h-3.5 w-3.5" />
              </button>
              <button className="w-11 h-8 flex items-center justify-center text-muted-foreground hover:bg-destructive hover:text-destructive-foreground transition-colors"
                onClick={() => getCurrentWindow().close()}>
                <X className="h-4 w-4" />
              </button>
            </div>
          )}
        </div>

        {/* Navigation */}
        <div className="flex-1 px-2 py-1.5 space-y-0.5">
          {NAV_ITEMS.map((item) => {
            const Icon = item.icon;
            const active = currentPage === item.id;
            const disabled = !connected && NEEDS_CONNECTION.has(item.id);
            return (
              <button
                key={item.id}
                className={cn(
                  "w-full flex items-center gap-2.5 px-2.5 h-[30px] rounded-md text-[13px] transition-colors duration-150",
                  active
                    ? "bg-accent text-accent-foreground font-medium"
                    : "text-muted-foreground hover:text-foreground hover:bg-accent/50",
                  disabled && "opacity-40 cursor-not-allowed"
                )}
                onClick={() => { if (!disabled) setCurrentPage(item.id); }}
                disabled={disabled}
              >
                <Icon className="h-4 w-4 shrink-0" />
                {item.label}
              </button>
            );
          })}
        </div>

        {/* Footer */}
        <div className="px-4 py-2.5 border-t border-border/40">
          <div className="text-[11px] text-muted-foreground/60">
            {connected ? "已连接" : "未连接"}
          </div>
        </div>
      </nav>

      {/* Main content */}
      <main className="flex-1 overflow-y-auto">
        <div className="h-8 shrink-0 cursor-default" onMouseDown={() => getCurrentWindow().startDragging()} />
        <div className="max-w-2xl px-6 pb-5">{renderPage()}</div>
      </main>
    </div>
  );
}
