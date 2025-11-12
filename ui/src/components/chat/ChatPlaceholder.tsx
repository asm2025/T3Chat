import { Button } from "@/components/ui/button";
import { Sparkles, Compass, Code, GraduationCap } from "lucide-react";

interface ChatPlaceholderProps {
    userName?: string;
    onPromptClick?: (prompt: string) => void;
}

export function ChatPlaceholder({ userName, onPromptClick }: ChatPlaceholderProps) {
    const greeting = userName ? `How can I help you, ${userName}?` : "How can I help you?";

    const actionButtons = [
        { label: "Create", icon: Sparkles },
        { label: "Explore", icon: Compass },
        { label: "Code", icon: Code },
        { label: "Learn", icon: GraduationCap },
    ];

    const prompts = ["How does AI work?", "Are black holes real?", 'How many Rs are in the word "strawberry"?', "What is the meaning of life?"];

    return (
        <div className="flex h-full items-center justify-center px-6">
            <div className="w-full max-w-2xl space-y-8 text-center">
                {/* Greeting */}
                <h2 className="text-3xl font-semibold tracking-tight text-foreground sm:text-4xl">{greeting}</h2>

                {/* Action Buttons */}
                <div className="flex flex-wrap justify-center gap-3">
                    {actionButtons.map(({ label, icon: Icon }) => (
                        <Button key={label} variant="outline" size="lg" className="gap-2 rounded-full">
                            <Icon className="h-4 w-4" />
                            {label}
                        </Button>
                    ))}
                </div>

                {/* Prompt Questions */}
                <div className="flex flex-col pt-4">
                    {prompts.map((prompt) => (
                        <Button
                            key={prompt}
                            variant="ghost"
                            className="h-auto justify-start rounded-none border-b border-border bg-background px-4 py-3 text-left text-sm font-normal text-muted-foreground hover:border-foreground/20 hover:bg-muted hover:text-foreground"
                            onClick={() => onPromptClick?.(prompt)}>
                            {prompt}
                        </Button>
                    ))}
                </div>
            </div>
        </div>
    );
}
