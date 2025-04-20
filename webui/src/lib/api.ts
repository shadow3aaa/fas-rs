import axios from "axios";

export interface App {
  name: string;
  package_name: string;
}

export const fetchApps = async (): Promise<App[]> => {
  const response = await axios.get("/api/apps");
  return response.data;
};
