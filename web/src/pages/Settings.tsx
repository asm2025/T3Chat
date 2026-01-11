import { useEffect, useRef } from "react";
import { MasterLayout } from "@/components/master-layout";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { EndpointSettings } from "@/components/Endpoints/EndpointSettings";
import { useLibreChatCurrentChat } from "@/stores/appStore";
import { createDefaultEndpointOptions } from "@/constants/endpoint-options";
import type { EndpointOption } from "@/types/librechat";

export function Settings() {
    const { endpointOptions, setEndpointOptions } = useLibreChatCurrentChat();
    const defaultOptionsRef = useRef(createDefaultEndpointOptions());
    const activeOptions = endpointOptions ?? defaultOptionsRef.current;

    useEffect(() => {
        if (!endpointOptions) {
            setEndpointOptions(createDefaultEndpointOptions());
        }
    }, [endpointOptions, setEndpointOptions]);

    const handleOptionsChange = (options: EndpointOption) => {
        setEndpointOptions(options);
    };

    return (
        <MasterLayout contentClassName="px-0">
            <div className="container mx-auto px-6 max-w-4xl space-y-6">
                <div>
                    <h1 className="text-3xl font-bold">Model Settings</h1>
                    <p className="text-muted-foreground">Configure the default behavior for future AI chats.</p>
                </div>

                <Card>
                    <CardHeader>
                        <CardTitle>Endpoint Configuration</CardTitle>
                        <CardDescription>Update your system prompt, sampling parameters, and advanced flags.</CardDescription>
                    </CardHeader>
                    <CardContent className="p-0">
                        <EndpointSettings options={activeOptions} onChange={handleOptionsChange} />
                    </CardContent>
                </Card>
            </div>
        </MasterLayout>
    );
}
