import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./en";
import zhHans from "./zh-Hans";

void i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    "zh-Hans": { translation: zhHans },
  },
  lng: "en",
  fallbackLng: "en",
  interpolation: { escapeValue: false },
  returnNull: false,
});

export default i18n;
