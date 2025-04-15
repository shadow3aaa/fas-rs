declare module "@/i18n" {
  import { i18n } from "i18next";
  const instance: i18n;
  export default instance;
}

declare module "*.json" {
  const value: Record<string, string>;
  export default value;
}
