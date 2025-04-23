import { exec } from "@/lib/kernelsu";

export interface App {
  package_name: string;
}

export const parseNumberedList = (text: string): App[] => {
  const lineRegex = /^package:(\S+)/gm;
  const matches = [...text.matchAll(lineRegex)];
  return matches.map((m) => ({ package_name: m[1] }));
};

export const fetchApps = async (): Promise<App[]> => {
  try {
    const { errno, stdout } = await exec("pm list packages -3");
    if (errno) {
      throw new Error(`Failed to fetch apps: ${stdout}`);
    }
    const data = parseNumberedList(stdout);

    return Array.isArray(data)
      ? data.map((item) => {
          if (item && typeof item.package_name === "object") {
            const firstKey = Object.keys(item.package_name)[0];
            return { package_name: firstKey || "unknown" };
          }
          return item;
        })
      : [];
  } catch (error) {
    console.error("Error fetching apps:", error);
    return [];
  }
};
