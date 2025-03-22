import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import enTranslations from './locales/en/common.json';
import zhTranslations from './locales/zh/common.json';

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { common: enTranslations },
      zh: { common: zhTranslations }
    },
    fallbackLng: 'en',
    debug: process.env.NODE_ENV !== 'production',
    interpolation: {
      escapeValue: false,
    },
    ns: ['common'],
    defaultNS: 'common'
  });

export default i18n;
