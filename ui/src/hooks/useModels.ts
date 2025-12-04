import { useEffect, useState } from "react";
import { t3ChatClient } from "@/lib/t3-chat-client";
import type { AIModel } from "@/types/model";

export function useModels() {
    const [models, setModels] = useState<AIModel[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<Error | null>(null);

    useEffect(() => {
        let cancelled = false;

        const loadModels = async () => {
            try {
                setLoading(true);
                setError(null);
                const data = await t3ChatClient.listModels();
                if (!cancelled) {
                    setModels(data);
                }
            } catch (err) {
                if (!cancelled) {
                    setError(err as Error);
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        };

        loadModels();

        return () => {
            cancelled = true;
        };
    }, []);

    return { models, loading, error };
}

