const API = "/api/v1";

export async function api(path, options = {}) {
  const headers = new Headers(options.headers);
  const token = localStorage.getItem("claudia_token");
  if (token) headers.set("authorization", `Bearer ${token}`);
  if (options.body) headers.set("content-type", "application/json");
  const response = await fetch(`${API}${path}`, { ...options, headers });
  if (response.status === 204) return null;
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    const error = new Error(data.error || "Something went wrong. Please try again.");
    error.status = response.status;
    throw error;
  }
  return data;
}

export const money = (value) => new Intl.NumberFormat("id-ID", {
  style: "currency", currency: "IDR", maximumFractionDigits: 0,
}).format(value);

export const shortDate = (value) => new Intl.DateTimeFormat("id-ID", {
  day: "numeric", month: "short", year: "numeric",
}).format(new Date(value));
