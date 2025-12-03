import { LoginForm } from "@/components/login-form";
import { MasterLayout } from "@/components/master-layout";

export function Login() {
    return (
        <MasterLayout contentClassName="px-0">
            <div className="relative isolate flex min-h-[calc(100vh-8rem)] items-center justify-center overflow-hidden px-4 py-12 sm:py-16">
                <div className="w-full max-w-2xl">
                    <LoginForm />
                </div>
            </div>
        </MasterLayout>
    );
}
