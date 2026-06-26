import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function Hotkey() {
  return (
    <div className="space-y-4">
      <h1 className="text-xl font-semibold tracking-tight">热键设置</h1>
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">全局快捷键</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">热键功能将在后续版本中实现。</p>
        </CardContent>
      </Card>
    </div>
  );
}
