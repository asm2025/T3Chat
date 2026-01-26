import type { ComponentType } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Loader2, CheckCircle2, XCircle, Search, Code, FileText } from "lucide-react";
import type { ToolExecutionState } from "@/types/agent-events";

interface ToolExecutionTimelineProps {
    executions: ToolExecutionState[];
    thinkingMessage?: string | null;
}

const TOOL_ICONS: Record<string, ComponentType<{ className?: string }>> = {
    web_search: Search,
    code_interpreter: Code,
    file_read: FileText,
};

export function ToolExecutionTimeline({ executions, thinkingMessage }: ToolExecutionTimelineProps) {
    if (!thinkingMessage && executions.length === 0) {
        return null;
    }

    return (
        <div className="space-y-2">
            {thinkingMessage && (
                <Card className="bg-muted/50">
                    <CardContent className="p-3">
                        <div className="flex items-center gap-2">
                            <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                            <span className="text-sm text-muted-foreground">{thinkingMessage}</span>
                        </div>
                    </CardContent>
                </Card>
            )}

            {executions.map((execution) => {
                const Icon = TOOL_ICONS[execution.tool_type] || Code;
                const isRunning = execution.status === "running";
                const isFailed = execution.status === "failed";

                return (
                    <Card key={execution.tool_call_id} className="bg-muted/30">
                        <CardContent className="p-3">
                            <div className="flex items-start gap-3">
                                <div className="mt-0.5">
                                    {isRunning ? (
                                        <Loader2 className="h-4 w-4 animate-spin text-blue-500" />
                                    ) : isFailed ? (
                                        <XCircle className="h-4 w-4 text-red-500" />
                                    ) : (
                                        <CheckCircle2 className="h-4 w-4 text-green-500" />
                                    )}
                                </div>

                                <div className="flex-1 space-y-2">
                                    <div className="flex items-center gap-2">
                                        <Icon className="h-4 w-4 text-muted-foreground" />
                                        <span className="font-medium text-sm">{execution.tool_name}</span>
                                        <Badge variant={isRunning ? "secondary" : isFailed ? "destructive" : "default"}>{execution.status}</Badge>
                                    </div>

                                    {execution.arguments && (
                                        <details className="text-xs text-muted-foreground">
                                            <summary className="cursor-pointer">Arguments</summary>
                                            <pre className="mt-1 p-2 bg-background rounded text-xs overflow-auto">
                                                {JSON.stringify(execution.arguments, null, 2)}
                                            </pre>
                                        </details>
                                    )}

                                    {execution.output !== undefined && execution.output !== null && (
                                        <details className="text-xs text-muted-foreground">
                                            <summary className="cursor-pointer">Output</summary>
                                            <pre className="mt-1 p-2 bg-background rounded text-xs overflow-auto max-h-48">
                                                {typeof execution.output === "string" ? execution.output : JSON.stringify(execution.output, null, 2)}
                                            </pre>
                                        </details>
                                    )}

                                    {execution.error && (
                                        <div className="text-xs text-red-500 bg-red-50 dark:bg-red-950 p-2 rounded">
                                            {execution.error}
                                        </div>
                                    )}
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                );
            })}
        </div>
    );
}
