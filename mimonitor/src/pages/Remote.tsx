import { invoke } from "@tauri-apps/api/core";
import {
  Power, Home, Menu, ArrowLeft, ChevronUp, ChevronDown, ChevronLeft, ChevronRight,
  Volume2, VolumeX, Volume1,
} from "lucide-react";
import { Button } from "@/components/ui/button";

export default function Remote() {
  const sendKey = (key: string) => invoke("send_remote_key", { key });

  return (
    <div>
      <h1 className="text-sm font-medium mb-4">虚拟遥控器</h1>

      <div className="max-w-xs mx-auto pt-2">
        <div className="text-center mb-4">
          <span className="text-[10px] font-medium tracking-[0.2em] text-muted-foreground uppercase">G Pro Control</span>
        </div>
        <div className="flex flex-col items-center gap-5 pb-6">
          <Button variant="destructive" size="icon" className="rounded-full h-12 w-12" onClick={() => sendKey("power")}>
            <Power className="h-5 w-5" />
          </Button>

          <div className="flex gap-4">
            <Button variant="outline" size="icon" className="rounded-full" onClick={() => sendKey("home")}>
              <Home className="h-4 w-4" />
            </Button>
            <Button variant="outline" size="icon" className="rounded-full" onClick={() => sendKey("menu")}>
              <Menu className="h-4 w-4" />
            </Button>
            <Button variant="outline" size="icon" className="rounded-full" onClick={() => sendKey("back")}>
              <ArrowLeft className="h-4 w-4" />
            </Button>
          </div>

          <div className="relative w-40 h-40">
            <Button variant="outline" size="icon" className="absolute top-0 left-1/2 -translate-x-1/2 rounded-full" onClick={() => sendKey("up")}>
              <ChevronUp className="h-5 w-5" />
            </Button>
            <Button variant="outline" size="icon" className="absolute bottom-0 left-1/2 -translate-x-1/2 rounded-full" onClick={() => sendKey("down")}>
              <ChevronDown className="h-5 w-5" />
            </Button>
            <Button variant="outline" size="icon" className="absolute left-0 top-1/2 -translate-y-1/2 rounded-full" onClick={() => sendKey("left")}>
              <ChevronLeft className="h-5 w-5" />
            </Button>
            <Button variant="outline" size="icon" className="absolute right-0 top-1/2 -translate-y-1/2 rounded-full" onClick={() => sendKey("right")}>
              <ChevronRight className="h-5 w-5" />
            </Button>
            <Button variant="default" size="icon" className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full" onClick={() => sendKey("ok")}>
              OK
            </Button>
          </div>

          <div className="flex gap-4">
            <Button variant="outline" size="icon" className="rounded-full" onClick={() => sendKey("volume_down")}>
              <Volume1 className="h-4 w-4" />
            </Button>
            <Button variant="outline" size="icon" className="rounded-full" onClick={() => sendKey("mute")}>
              <VolumeX className="h-4 w-4" />
            </Button>
            <Button variant="outline" size="icon" className="rounded-full" onClick={() => sendKey("volume_up")}>
              <Volume2 className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
