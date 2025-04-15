"use client";
import { I18nextProvider } from "react-i18next";
import type React from "react";

import "./globals.css";
import { useState, useEffect } from "react";
import i18n from "../i18n";
import Navbar from "@/components/Navbar";

export default function ClientLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const [_, setCurrentLang] = useState(() => {
    if (typeof window !== "undefined") {
      return i18n.language;
    }
    return "en";
  });

  useEffect(() => {
    const handleLanguageChange = () => {
      setCurrentLang(i18n.language);
      document.documentElement.lang = i18n.language;
    };

    i18n.on("languageChanged", handleLanguageChange);
    return () => i18n.off("languageChanged", handleLanguageChange);
  }, []);

  return (
    <I18nextProvider i18n={i18n}>
      <div className="flex min-h-screen flex-col bg-background">
        <Navbar />
        <main className="flex-1">{children}</main>
      </div>
    </I18nextProvider>
  );
}
