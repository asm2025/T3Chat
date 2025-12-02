import { ReactNode } from 'react';
import { AdminNav } from '@/components/admin/admin-nav';

export function AdminLayout({ children }: { children: ReactNode }) {
  return (
    <div className="flex min-h-screen bg-background">
      <AdminNav />
      <main className="flex-1 p-8">
        {children}
      </main>
    </div>
  );
}

