import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Github, Book, Code, Database } from "lucide-react";

export function About() {
    return (
        <div className="min-h-screen bg-background">
            <div className="container mx-auto px-4 py-16 max-w-4xl">
                <div className="space-y-12">
                    {/* Header */}
                    <div className="text-center space-y-4">
                        <h1 className="text-4xl font-bold">About T3Chat</h1>
                        <p className="text-xl text-muted-foreground">A modern, open-source chat platform built with cutting-edge technology</p>
                    </div>

                    {/* Project Information */}
                    <Card>
                        <CardHeader>
                            <CardTitle>Project Information</CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <p className="text-muted-foreground">
                                T3Chat is an open-source chat application that provides a modern interface for AI-powered chats. Built with performance and security in mind, T3Chat offers enterprise-grade features while remaining accessible to everyone.
                            </p>
                            <p className="text-muted-foreground">
                                The platform supports multiple AI providers and models, allowing users to choose the best AI for their needs. With features like chat history, custom presets, and role-based access control, T3Chat is suitable for both
                                individual users and organizations.
                            </p>
                        </CardContent>
                    </Card>

                    {/* Technology Stack */}
                    <Card>
                        <CardHeader>
                            <CardTitle>Technology Stack</CardTitle>
                            <CardDescription>Built with modern, reliable technologies</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="grid md:grid-cols-2 gap-6">
                                <div className="space-y-2">
                                    <h3 className="font-semibold flex items-center gap-2">
                                        <Code className="h-5 w-5" />
                                        Frontend
                                    </h3>
                                    <ul className="list-disc list-inside text-muted-foreground space-y-1">
                                        <li>React 19 with TypeScript</li>
                                        <li>Vite for fast development</li>
                                        <li>Tailwind CSS for styling</li>
                                        <li>ShadCN UI components</li>
                                        <li>React Router for navigation</li>
                                    </ul>
                                </div>

                                <div className="space-y-2">
                                    <h3 className="font-semibold flex items-center gap-2">
                                        <Database className="h-5 w-5" />
                                        Backend
                                    </h3>
                                    <ul className="list-disc list-inside text-muted-foreground space-y-1">
                                        <li>Rust with Axum framework</li>
                                        <li>PostgreSQL database</li>
                                        <li>OIDC authentication</li>
                                        <li>JWT token management</li>
                                        <li>RESTful API design</li>
                                    </ul>
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    {/* Links */}
                    <Card>
                        <CardHeader>
                            <CardTitle>Resources</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <div className="flex flex-wrap gap-4">
                                <Button asChild variant="outline">
                                    <a href="https://github.com" target="_blank" rel="noopener noreferrer">
                                        <Github className="mr-2 h-4 w-4" />
                                        GitHub Repository
                                    </a>
                                </Button>
                                <Button asChild variant="outline">
                                    <Link to="/health">
                                        <Book className="mr-2 h-4 w-4" />
                                        API Health
                                    </Link>
                                </Button>
                            </div>
                        </CardContent>
                    </Card>

                    {/* Footer */}
                    <div className="text-center text-muted-foreground">
                        <p>© 2025 T3Chat. Open source software.</p>
                    </div>
                </div>
            </div>
        </div>
    );
}
