import axios from "axios";

const apiClient = axios.create({
  baseURL: "http://localhost:8080",
  headers: {
    "Content-Type": "application/json",
  },
});

export interface App {
  package_name: string;
}

export const fetchApps = async (): Promise<App[]> => {
  try {
    const response = await apiClient.get<App[]>("/api/apps");

    console.log("API Response:", response.data);

    return Array.isArray(response.data)
      ? response.data.map((item) => {
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
