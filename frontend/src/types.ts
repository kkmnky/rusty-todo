export type User = {
  id: string;
  name: string;
  email: string;
};

export type Todo = {
  id: string;
  assignee_user_id: string;
  title: string;
  completed: boolean;
  due_at: string | null;
};
