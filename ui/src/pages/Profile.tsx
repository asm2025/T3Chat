import { useAuth } from "@/stores/appStore";
import { t3ChatClient } from "@/lib/t3-chat-client";
import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar";
import { Skeleton } from "@/components/ui/skeleton";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Edit2, X } from "lucide-react";
import { MasterLayout } from "@/components/MasterLayout";
import { toast } from "@/lib/toast";
import { getErrorMessage } from "@/lib/utils";

interface UserProfile {
    id: string;
    email: string | null;
    display_name: string | null;
    image_url: string | null;
    created_at: string;
    updated_at: string;
}

export function Profile() {
    const { user, forceRefresh } = useAuth();
    const [userProfile, setUserProfile] = useState<UserProfile | null>(null);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [displayName, setDisplayName] = useState("");
    const [isEditing, setIsEditing] = useState(false);
    const [originalDisplayName, setOriginalDisplayName] = useState("");

    useEffect(() => {
        async function fetchUserInfo() {
            if (user) {
                try {
                    setLoading(true);
                    const data = await t3ChatClient.getCurrentUser();
                    setUserProfile(data);
                    setDisplayName(data.display_name || "");
                    setOriginalDisplayName(data.display_name || "");
                } catch (error) {
                    const errorMessage = getErrorMessage(error);
                    toast.error("Failed to fetch user profile", {
                        description: errorMessage,
                    });
                    console.error("Server error:", error);
                } finally {
                    setLoading(false);
                }
            }
        }
        fetchUserInfo();
    }, [user]);

    const handleEdit = () => {
        if (userProfile) {
            setOriginalDisplayName(userProfile.display_name || "");
            setDisplayName(userProfile.display_name || "");
            setIsEditing(true);
        }
    };

    const handleCancel = () => {
        setDisplayName(originalDisplayName);
        setIsEditing(false);
    };

    const handleSave = async () => {
        if (!user || !userProfile) return;

        try {
            setSaving(true);
            await t3ChatClient.updateUser({ display_name: displayName || null });

            // Refresh the profile
            const updatedData = await t3ChatClient.getCurrentUser();
            setUserProfile(updatedData);
            setOriginalDisplayName(updatedData.display_name || "");

            // Refresh the user data in the navbar
            forceRefresh();

            setIsEditing(false);
            toast.success("Profile updated successfully");
        } catch (error) {
            const errorMessage = getErrorMessage(error);
            toast.error("Failed to update profile", {
                description: errorMessage,
            });
        } finally {
            setSaving(false);
        }
    };

    const formatDate = (dateString: string) => {
        try {
            return new Date(dateString).toLocaleDateString("en-US", {
                year: "numeric",
                month: "long",
                day: "numeric",
            });
        } catch {
            return dateString;
        }
    };

    const getInitials = (name: string | null, email: string | null) => {
        if (name) {
            return name
                .split(" ")
                .map((n) => n[0])
                .join("")
                .toUpperCase()
                .slice(0, 2);
        }
        if (email) {
            return email[0].toUpperCase();
        }
        return "U";
    };

    if (loading) {
        return (
            <MasterLayout contentClassName="px-0">
                <div className="container mx-auto px-6 max-w-2xl">
                    <Card>
                        <CardHeader>
                            <div className="flex items-center gap-4">
                                <Skeleton className="h-20 w-20 rounded-full" />
                                <div className="space-y-2 flex-1">
                                    <Skeleton className="h-6 w-48" />
                                    <Skeleton className="h-4 w-64" />
                                </div>
                            </div>
                        </CardHeader>
                        <CardContent>
                            <div className="space-y-4">
                                <Skeleton className="h-4 w-full" />
                                <Skeleton className="h-4 w-3/4" />
                                <Skeleton className="h-4 w-1/2" />
                            </div>
                        </CardContent>
                    </Card>
                </div>
            </MasterLayout>
        );
    }

    // Error is now handled via toast, but we still show a fallback UI if profile failed to load
    if (!loading && !userProfile) {
        return (
            <MasterLayout contentClassName="px-0">
                <div className="container mx-auto px-6 max-w-2xl">
                    <Card>
                        <CardContent className="pt-6">
                            <p className="text-muted-foreground">Unable to load profile. Please try again.</p>
                        </CardContent>
                    </Card>
                </div>
            </MasterLayout>
        );
    }

    if (!userProfile) {
        return (
            <MasterLayout contentClassName="px-0">
                <div className="container mx-auto px-6 max-w-2xl">
                    <Card>
                        <CardContent className="pt-6">
                            <p className="text-muted-foreground">No user profile available</p>
                        </CardContent>
                    </Card>
                </div>
            </MasterLayout>
        );
    }

    return (
        <MasterLayout contentClassName="px-0">
            <div className="container mx-auto px-6 max-w-2xl">
                <div className="space-y-6">
                    <div>
                        <h1 className="text-3xl font-bold">Profile</h1>
                        <p className="text-muted-foreground mt-2">View and manage your account information</p>
                    </div>

                    <Card>
                        <CardHeader>
                            <div className="flex items-center gap-4">
                                <Avatar className="h-20 w-20">
                                    {userProfile.image_url && <AvatarImage src={userProfile.image_url} alt={userProfile.display_name || "User"} />}
                                    <AvatarFallback className="text-2xl">{getInitials(userProfile.display_name, userProfile.email)}</AvatarFallback>
                                </Avatar>
                                <div>
                                    <CardTitle className="text-2xl">{userProfile.display_name || "User"}</CardTitle>
                                    <CardDescription className="text-base">{userProfile.email || "No email address"}</CardDescription>
                                </div>
                            </div>
                        </CardHeader>
                        <CardContent className="space-y-6">
                            {!isEditing ? (
                                <>
                                    <div className="space-y-4">
                                        <div className="space-y-2">
                                            <Label htmlFor="displayName">Display Name</Label>
                                            <div className="text-sm py-2 px-3 rounded-md border bg-muted/50">{userProfile.display_name || "Not set"}</div>
                                        </div>
                                    </div>

                                    <div className="border-t pt-4 space-y-2">
                                        <div className="flex justify-between items-center py-2">
                                            <span className="text-sm font-medium text-muted-foreground">User ID</span>
                                            <span className="text-sm font-mono">{userProfile.id}</span>
                                        </div>
                                        <div className="flex justify-between items-center py-2">
                                            <span className="text-sm font-medium text-muted-foreground">Email</span>
                                            <span className="text-sm">{userProfile.email || "Not set"}</span>
                                        </div>
                                        <div className="flex justify-between items-center py-2">
                                            <span className="text-sm font-medium text-muted-foreground">Member Since</span>
                                            <span className="text-sm">{formatDate(userProfile.created_at)}</span>
                                        </div>
                                        <div className="flex justify-between items-center py-2">
                                            <span className="text-sm font-medium text-muted-foreground">Last Updated</span>
                                            <span className="text-sm">{formatDate(userProfile.updated_at)}</span>
                                        </div>
                                    </div>

                                    <div className="flex justify-end pt-4">
                                        <Button onClick={handleEdit} variant="outline" className="flex items-center gap-2">
                                            <Edit2 className="h-4 w-4" />
                                            Edit
                                        </Button>
                                    </div>
                                </>
                            ) : (
                                <>
                                    <div className="space-y-4">
                                        <div className="space-y-2">
                                            <Label htmlFor="displayName">Display Name</Label>
                                            <Input id="displayName" value={displayName} onChange={(e) => setDisplayName(e.target.value)} placeholder="Enter your display name" />
                                        </div>
                                    </div>

                                    <div className="border-t pt-4 space-y-2">
                                        <div className="flex justify-between items-center py-2">
                                            <span className="text-sm font-medium text-muted-foreground">User ID</span>
                                            <span className="text-sm font-mono">{userProfile.id}</span>
                                        </div>
                                        <div className="flex justify-between items-center py-2">
                                            <span className="text-sm font-medium text-muted-foreground">Email</span>
                                            <span className="text-sm">{userProfile.email || "Not set"}</span>
                                        </div>
                                        <div className="flex justify-between items-center py-2">
                                            <span className="text-sm font-medium text-muted-foreground">Member Since</span>
                                            <span className="text-sm">{formatDate(userProfile.created_at)}</span>
                                        </div>
                                        <div className="flex justify-between items-center py-2">
                                            <span className="text-sm font-medium text-muted-foreground">Last Updated</span>
                                            <span className="text-sm">{formatDate(userProfile.updated_at)}</span>
                                        </div>
                                    </div>

                                    <div className="flex justify-end gap-2 pt-4">
                                        <Button onClick={handleCancel} variant="outline" disabled={saving} className="flex items-center gap-2">
                                            <X className="h-4 w-4" />
                                            Cancel
                                        </Button>
                                        <Button onClick={handleSave} disabled={saving} className="flex items-center gap-2">
                                            {saving ? "Saving..." : "Save Changes"}
                                        </Button>
                                    </div>
                                </>
                            )}
                        </CardContent>
                    </Card>
                </div>
            </div>
        </MasterLayout>
    );
}
