import type { Message } from "@/types/chat";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

interface MessageBubbleProps {
    message: Message;
    streaming?: boolean;
}

export function MessageBubble({ message, streaming }: MessageBubbleProps) {
    const isUser = message.role === "user";

    if (!isUser) {
        // Assistant messages: render markdown
        return (
            <div className="flex w-full gap-3 justify-start">
                <Avatar className="h-8 w-8 border border-border bg-background shadow-sm">
                    <AvatarFallback className="text-xs font-medium">AI</AvatarFallback>
                </Avatar>
                <div className="flex max-w-[80%] flex-col items-start text-left">
                    <span className="mb-1.5 text-xs uppercase tracking-[0.16em] text-muted-foreground">Assistant</span>
                    <div className="w-full max-w-none">
                        <ReactMarkdown
                            remarkPlugins={[remarkGfm]}
                            components={{
                                // Style headings
                                h1: ({ node, ...props }) => <h1 className="text-xl font-bold mt-4 mb-2" {...props} />,
                                h2: ({ node, ...props }) => <h2 className="text-lg font-bold mt-4 mb-2" {...props} />,
                                h3: ({ node, ...props }) => <h3 className="text-base font-bold mt-3 mb-2" {...props} />,
                                h4: ({ node, ...props }) => <h4 className="text-sm font-bold mt-3 mb-2" {...props} />,
                                h5: ({ node, ...props }) => <h5 className="text-sm font-semibold mt-2 mb-1" {...props} />,
                                h6: ({ node, ...props }) => <h6 className="text-xs font-semibold mt-2 mb-1" {...props} />,
                                // Style paragraphs
                                p: ({ node, ...props }) => <p className="mb-3 leading-relaxed last:mb-0" {...props} />,
                                // Style lists
                                ul: ({ node, ...props }) => <ul className="list-disc list-outside mb-3 space-y-1 pl-6" {...props} />,
                                ol: ({ node, ...props }) => <ol className="list-decimal list-outside mb-3 space-y-1 pl-6" {...props} />,
                                li: ({ node, ...props }) => <li className="leading-relaxed" {...props} />,
                                // Style code blocks and inline code
                                code: ({ node, className, children, ...props }) => {
                                    const isInline = !className;
                                    return isInline ? (
                                        <code className="bg-muted px-1.5 py-0.5 rounded text-xs font-mono" {...props}>
                                            {children}
                                        </code>
                                    ) : (
                                        <code className={className} {...props}>
                                            {children}
                                        </code>
                                    );
                                },
                                pre: ({ node, children, ...props }) => (
                                    <pre className="bg-muted p-3 rounded-md overflow-x-auto mb-3" {...props}>
                                        {children}
                                    </pre>
                                ),
                                // Style links
                                a: ({ node, ...props }) => <a className="text-primary underline hover:text-primary/80" target="_blank" rel="noopener noreferrer" {...props} />,
                                // Style blockquotes
                                blockquote: ({ node, ...props }) => <blockquote className="border-l-4 border-border pl-4 italic my-3 text-muted-foreground" {...props} />,
                                // Style horizontal rules
                                hr: ({ node, ...props }) => <hr className="my-4 border-border" {...props} />,
                                // Style tables (from remark-gfm)
                                table: ({ node, ...props }) => <table className="border-collapse border border-border my-3 w-full" {...props} />,
                                thead: ({ node, ...props }) => <thead className="bg-muted" {...props} />,
                                tbody: ({ node, ...props }) => <tbody {...props} />,
                                tr: ({ node, ...props }) => <tr className="border-b border-border" {...props} />,
                                th: ({ node, ...props }) => <th className="border border-border px-3 py-2 text-left font-semibold" {...props} />,
                                td: ({ node, ...props }) => <td className="border border-border px-3 py-2" {...props} />,
                            }}>
                            {message.content}
                        </ReactMarkdown>
                        {streaming && <span className="ml-1 inline-block h-4 w-2 animate-pulse bg-current" />}
                        {(message.modelUsed || message.tokensUsed !== undefined) && (
                            <div className="mt-2 flex items-center gap-2 text-xs text-muted-foreground">
                                {message.modelUsed && <span>{message.modelUsed}</span>}
                                {message.tokensUsed !== undefined && <span>• {message.tokensUsed} tokens</span>}
                            </div>
                        )}
                    </div>
                </div>
            </div>
        );
    }

    // User messages: with bubble container
    return (
        <div className="flex w-full gap-3 justify-end">
            <div className="flex max-w-[80%] flex-col items-end text-right">
                <span className="mb-1.5 text-xs uppercase tracking-[0.16em] text-muted-foreground">You</span>
                <div className="w-full rounded-xl border border-foreground/20 bg-foreground text-background px-4 py-3 shadow-sm transition">
                    <p className="text-sm leading-relaxed whitespace-pre-wrap">
                        {message.content}
                        {streaming && <span className="ml-1 inline-block h-4 w-2 animate-pulse bg-current" />}
                    </p>
                    {(message.modelUsed || message.tokensUsed !== undefined) && (
                        <div className="mt-2 flex items-center gap-2 border-t border-current/10 pt-2 text-xs opacity-70">
                            {message.modelUsed && <span>{message.modelUsed}</span>}
                            {message.tokensUsed !== undefined && <span>• {message.tokensUsed} tokens</span>}
                        </div>
                    )}
                </div>
            </div>
            <Avatar className="h-8 w-8 border border-border bg-background shadow-sm">
                <AvatarFallback className="text-xs font-medium">You</AvatarFallback>
            </Avatar>
        </div>
    );
}
