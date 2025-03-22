'use client';
import { Toaster } from "@/components/ui/sonner";
import { useConfig } from "@/hooks/useConfig";

export default function ClientLayout({ children }: { children: React.ReactNode }) {
  const { language, toggleLanguage } = useConfig();
  
  return (
    <>
      {children}
      <Toaster />
      {/* 语言切换悬浮按钮 */}
      <div className="fixed bottom-6 right-6 z-50">
        <button
          onClick={toggleLanguage}
          className="p-3 rounded-full bg-primary text-primary-foreground shadow-lg hover:bg-primary/90 transition-colors duration-200 hover:scale-105"
          suppressHydrationWarning
          title={language === 'en' ? '切换中文' : 'Switch to English'}
        >
          <span suppressHydrationWarning>{language === 'en' ? '中' : 'EN'}</span>
        </button>
      </div>
    </>
  );
}
