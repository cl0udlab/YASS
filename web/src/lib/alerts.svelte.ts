export type AlertType = "info" | "success" | "warning" | "error";

interface Alert {
  id: number;
  message: string;
  type: AlertType;
}

let nextId = 0;

export const alerts = $state<Alert[]>([]);

export function showAlert(message: string, type: AlertType = "info") {
  const id = nextId++;
  alerts.push({ id, message, type });
  setTimeout(() => {
    const index = alerts.findIndex((a) => a.id === id);
    if (index !== -1) alerts.splice(index, 1);
  }, 5000);
}
