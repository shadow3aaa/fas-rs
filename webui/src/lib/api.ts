import axios from "axios";

const apiClient = axios.create({
  baseURL: 'http://localhost:8000'
});

export interface App {
  name: string;
  package_name: string;
}

export const fetchApps = async (): Promise<App[]> => {
  const response = await apiClient.get("/api/apps");
  return response.data;
};
