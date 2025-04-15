"use client";
import { useTranslation } from "react-i18next";
import { Button } from "./ui/button";
import { useState, useEffect } from "react";

export default function LanguageSwitcher() {
  const { i18n } = useTranslation();
  const [currentLang, setCurrentLang] = useState(i18n.language);
  useEffect(() => {
    setCurrentLang(i18n.language);
  }, [i18n.language]);

  const changeLanguage = (lng: string) => {
    i18n.changeLanguage(lng);
    setCurrentLang(lng);
  };

  return (
    <div className="fixed bottom-4 right-4 flex gap-2 bg-background/95 p-2 rounded-lg shadow-lg backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <Button
        variant={currentLang === "en" ? "default" : "outline"}
        size="sm"
        onClick={() => changeLanguage("en")}
      >
        English
      </Button>
      <Button
        variant={currentLang === "zh" ? "default" : "outline"}
        size="sm"
        onClick={() => changeLanguage("zh")}
      >
        中文
      </Button>
    </div>
  );
}
