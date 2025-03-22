"use client";
import { I18nextProvider } from 'react-i18next';
import './globals.css';
import LanguageSwitcher from '@/components/LanguageSwitcher';
import { useState, useEffect } from 'react';
import i18n from '../i18n';

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const [_, setCurrentLang] = useState(() => {
    if (typeof window !== 'undefined') {
      return i18n.language;
    }
    return 'en';
  });
  
  useEffect(() => {
    const handleLanguageChange = () => {
      setCurrentLang(i18n.language);
      document.documentElement.lang = i18n.language;
    };
    
    i18n.on('languageChanged', handleLanguageChange);
    return () => i18n.off('languageChanged', handleLanguageChange);
  }, []);

  return (
    <html lang="en">
      <body>
        <I18nextProvider i18n={i18n}>
          {children}
          <LanguageSwitcher />
        </I18nextProvider>
      </body>
    </html>
  );
}
