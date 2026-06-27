import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useCallback } from "react";
import { RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

const SOURCES = [
  { id: 23, name: "HDMI 1" },
  { id: 24, name: "HDMI 2" },
  { id: 29, name: "DP" },
  { id: 30, name: "USBC" },
];

export default function Source() {
  const [activeSource, setActiveSource] = useState(0);
  const [sourceName, setSourceName] = useState("");
  const [loaded, setLoaded] = useState(false);

  const refresh = useCallback(() => {
    invoke<{ id: number; name: string }>("get_input_source").then((s) => {
      setActiveSource(s.id);
      setSourceName(s.name);
      setLoaded(true);
    });
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-sm font-medium mb-4">输入源</h1>
        <Button variant="outline" size="sm" onClick={refresh} disabled={!loaded}>
          <RefreshCw className="h-3 w-3 mr-1" />刷新
        </Button>
      </div>

      <div className="border-b pb-3 mb-3">
        <div className="text-xs font-medium text-muted-foreground mb-2">信号源切换</div>
        <ToggleGroup type="single" value={String(activeSource)} onValueChange={(v) => {
          if (v) {
            const id = parseInt(v);
            setActiveSource(id);
            setSourceName(SOURCES.find((s) => s.id === id)?.name || "");
            invoke("set_input_source", { sourceId: id });
          }
        }} disabled={!loaded}>
          {SOURCES.map((src) => (
            <ToggleGroupItem key={src.id} value={String(src.id)} className="flex-1">{src.name}</ToggleGroupItem>
          ))}
        </ToggleGroup>
      </div>

      {sourceName && (
        <div className="pt-4">
          <div className={cn("text-3xl font-light text-center tracking-wide", loaded ? "text-foreground/80" : "text-muted-foreground")}>{sourceName}</div>
        </div>
      )}
    </div>
  );
}
