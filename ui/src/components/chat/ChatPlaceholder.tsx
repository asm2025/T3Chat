import { Sparkles, Newspaper, Code, GraduationCap } from "lucide-react";

interface ChatPlaceholderProps {
    userName?: string;
    onPromptClick?: (prompt: string) => void;
}

export function ChatPlaceholder({ userName, onPromptClick }: ChatPlaceholderProps) {
    const greeting = userName ? `How can I help you, ${userName}?` : "How can I help you?";

    const actionButtons = [
        { label: "Create", icon: Sparkles },
        { label: "Explore", icon: Newspaper },
        { label: "Code", icon: Code },
        { label: "Learn", icon: GraduationCap },
    ];

    const prompts = ["How does AI work?", "Are black holes real?", 'How many Rs are in the word "strawberry"?', "What is the meaning of life?"];

    return (
        <div className="flex h-[calc(100vh-20rem)] items-start justify-center">
            <div className="animate-in fade-in-50 zoom-in-95 w-full space-y-6 px-2 pt-[calc(max(15vh,2.5rem))] duration-300 sm:px-8">
                <h2 className="text-3xl font-semibold">{greeting}</h2>

                <div className="flex flex-row flex-wrap gap-2.5 text-sm max-sm:justify-evenly">
                    {actionButtons.map(({ label, icon: Icon }) => {
                        return (
                            <button
                                key={label}
                                data-selected="false"
                                className="focus-visible:ring-ring justify-center text-sm whitespace-nowrap transition-colors focus-visible:ring-1 focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 bg-primary text-primary-foreground disabled:hover:bg-primary shadow-sm hover:bg-pink-600/90 h-9 outline-secondary/70 data-[selected=false]:bg-secondary/30 data-[selected=false]:text-secondary-foreground/90 data-[selected=false]:hover:bg-secondary flex items-center gap-1 rounded-xl px-5 py-2 font-semibold outline-1 backdrop-blur-xl data-[selected=false]:outline-solid max-sm:size-16 max-sm:flex-col sm:gap-2 sm:rounded-full">
                                <Icon className="max-sm:block" />
                                <div>{label}</div>
                            </button>
                        );
                    })}
                </div>

                <div className="text-foreground flex flex-col">
                    {prompts.map((prompt) => {
                        return (
                            <div key={prompt} className="border-secondary/40 flex items-start gap-2 border-t py-1 first:border-none">
                                <button className="text-secondary-foreground hover:bg-secondary/50 w-full rounded-md py-2 text-left sm:px-3" onClick={() => onPromptClick?.(prompt)}>
                                    <span>{prompt}</span>
                                </button>
                            </div>
                        );
                    })}
                </div>
            </div>
        </div>
    );
}
