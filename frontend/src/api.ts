import type { Todo, User } from "./types";

type LoginResponse = {
  access_token: string;
  expires_in: number;
  user_id: string;
};

type UsersResponse = {
  items: User[];
};

type TodosResponse = {
  items: Todo[];
};

const jsonHeaders = {
  "Content-Type": "application/json",
  Accept: "application/json"
};

function authHeaders(token: string, withJson = false): HeadersInit {
  return {
    ...(withJson ? jsonHeaders : { Accept: "application/json" }),
    Authorization: `Bearer ${token}`
  };
}

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
    headers: authHeaders(token)
  });
  return handleJson<User>(res);
}

export async function listUsers(token: string): Promise<User[]> {
  const res = await fetch("/api/v1/users", {
    method: "GET",
    headers: authHeaders(token)
  });
  const data = await handleJson<UsersResponse>(res);
  return data.items;
}

export async function deleteUser(token: string, userId: string): Promise<void> {
  const res = await fetch(`/api/v1/users/${userId}`, {
    method: "DELETE",
    headers: authHeaders(token)
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
    headers: authHeaders(token, true),
    body: JSON.stringify({
      currentPassword,
      newPassword
    })
  });
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`);
  }
}

type RegisterTodoInput = {
  title: string;
  assigneeUserId: string;
  dueAt?: string | null;
};

export async function registerTodo(
  token: string,
  input: RegisterTodoInput
): Promise<Todo> {
  const res = await fetch("/api/v1/todos", {
    method: "POST",
    headers: authHeaders(token, true),
    body: JSON.stringify(input)
  });
  return handleJson<Todo>(res);
}

export async function listMyTodos(token: string): Promise<Todo[]> {
  const res = await fetch("/api/v1/todos/me", {
    method: "GET",
    headers: authHeaders(token)
  });
  const data = await handleJson<TodosResponse>(res);
  return data.items;
}

export async function updateTodoCompleted(
  token: string,
  todoId: string,
  completed: boolean
): Promise<Todo> {
  const res = await fetch(`/api/v1/todos/${todoId}/completed`, {
    method: "PATCH",
    headers: authHeaders(token, true),
    body: JSON.stringify({ completed })
  });
  return handleJson<Todo>(res);
}

type UpdateTodoInput = {
  title?: string;
  assigneeUserId?: string;
  dueAt?: string | null;
};

export async function updateTodo(
  token: string,
  todoId: string,
  input: UpdateTodoInput
): Promise<Todo> {
  const res = await fetch(`/api/v1/todos/${todoId}`, {
    method: "PATCH",
    headers: authHeaders(token, true),
    body: JSON.stringify(input)
  });
  return handleJson<Todo>(res);
}

export async function deleteTodo(token: string, todoId: string): Promise<void> {
  const res = await fetch(`/api/v1/todos/${todoId}`, {
    method: "DELETE",
    headers: authHeaders(token)
  });
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`);
  }
}
