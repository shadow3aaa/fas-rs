export interface App {
  package_name: string;
}

export const fetchApps = async (): Promise<App[]> => {
  try {
    const response = await fetch("http://127.0.0.1:8080/api/apps", {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json();

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