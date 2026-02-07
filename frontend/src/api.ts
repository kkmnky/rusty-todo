import type { User } from "./types";

type LoginResponse = {
  access_token: string;
  expires_in: number;
  user_id: string;
};

type UsersResponse = {
  items: User[];
};

const jsonHeaders = {
  "Content-Type": "application/json",
  Accept: "application/json"
};

async function handleJson<T>(res: Response): Promise<T> {
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

export async function login(email: string, password: string): Promise<LoginResponse> {
  const res = await fetch("/api/v1/auth/login", {
    method: "POST",
    headers: jsonHeaders,
    body: JSON.stringify({ email, password })
  });
  return handleJson<LoginResponse>(res);
}

export async function registerUser(
  name: string,
  email: string,
  password: string
): Promise<User> {
  const res = await fetch("/api/v1/users", {
    method: "POST",
    headers: jsonHeaders,
    body: JSON.stringify({ name, email, password })
  });
  return handleJson<User>(res);
}

export async function getCurrentUser(token: string): Promise<User> {
  const res = await fetch("/api/v1/users/me", {
    method: "GET",
    headers: {
      Accept: "application/json",
      Authorization: `Bearer ${token}`
    }
  });
  return handleJson<User>(res);
}

export async function listUsers(token: string): Promise<User[]> {
  const res = await fetch("/api/v1/users", {
    method: "GET",
    headers: {
      Accept: "application/json",
      Authorization: `Bearer ${token}`
    }
  });
  const data = await handleJson<UsersResponse>(res);
  return data.items;
}

export async function deleteUser(token: string, userId: string): Promise<void> {
  const res = await fetch(`/api/v1/users/${userId}`, {
    method: "DELETE",
    headers: {
      Accept: "application/json",
      Authorization: `Bearer ${token}`
    }
  });
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`);
  }
}

export async function changePassword(
  token: string,
  currentPassword: string,
  newPassword: string
): Promise<void> {
  const res = await fetch("/api/v1/users/me/password", {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
      Authorization: `Bearer ${token}`
    },
    body: JSON.stringify({
      currentPassword,
      newPassword
    })
  });
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`);
  }
}
