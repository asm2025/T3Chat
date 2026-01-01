import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { MessageSquare, Zap, Shield, Users } from "lucide-react";

export function Home() {
    return (
        <div className="min-h-screen bg-linear-to-b from-background to-muted/20">
            <div className="container mx-auto px-4 py-16">
                {/* Hero Section */}
                <div className="text-center space-y-6 mb-16">
                    <h1 className="text-5xl font-bold tracking-tight">Welcome to T3Chat</h1>
                    <p className="text-xl text-muted-foreground max-w-2xl mx-auto">A modern, open-source chat platform powered by AI. Connect, collaborate, and create with intelligent chats.</p>
                    <div className="flex gap-4 justify-center">
                        <Button asChild size="lg">
                            <Link to="/login">Get Started</Link>
                        </Button>
                        <Button asChild variant="outline" size="lg">
                            <Link to="/about">Learn More</Link>
                        </Button>
                    </div>
                </div>

                {/* Features Section */}
                <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6 mb-16">
                    <Card>
                        <CardHeader>
                            <MessageSquare className="h-8 w-8 text-primary mb-2" />
                            <CardTitle>AI-Powered Chat</CardTitle>
                            <CardDescription>Engage in intelligent chats with state-of-the-art AI models</CardDescription>
                        </CardHeader>
                    </Card>

                    <Card>
                        <CardHeader>
                            <Zap className="h-8 w-8 text-primary mb-2" />
                            <CardTitle>Lightning Fast</CardTitle>
                            <CardDescription>Built for speed and performance with modern web technologies</CardDescription>
                        </CardHeader>
                    </Card>

                    <Card>
                        <CardHeader>
                            <Shield className="h-8 w-8 text-primary mb-2" />
                            <CardTitle>Secure & Private</CardTitle>
                            <CardDescription>Enterprise-grade security with OIDC authentication</CardDescription>
                        </CardHeader>
                    </Card>

                    <Card>
                        <CardHeader>
                            <Users className="h-8 w-8 text-primary mb-2" />
                            <CardTitle>Open Source</CardTitle>
                            <CardDescription>Built by the community, for the community</CardDescription>
                        </CardHeader>
                    </Card>
                </div>

                {/* CTA Section */}
                <div className="text-center space-y-4">
                    <h2 className="text-3xl font-bold">Ready to get started?</h2>
                    <p className="text-muted-foreground">Join thousands of users already using T3Chat</p>
                    <Button asChild size="lg">
                        <Link to="/login">Sign in</Link>
                    </Button>
                </div>
            </div>
        </div>
    );
}
