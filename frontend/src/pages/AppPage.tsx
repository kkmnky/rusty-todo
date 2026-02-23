import { useEffect, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  Checkbox,
  Divider,
  Stack,
  Tab,
  Tabs,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  TextField,
  Typography
} from "@mui/material";
import { Link, useNavigate } from "react-router-dom";
import {
  deleteTodo,
  deleteUser,
  listMyTodos,
  listUsers,
  registerTodo,
  updateTodo,
  updateTodoCompleted
} from "../api";
import type { Todo, User } from "../types";

type AppPageProps = {
  currentUser: User;
  accessToken: string;
  onLogout: () => void;
};

function toDateTimeLocalValue(value: string | null): string {
  if (!value) {
    return "";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  const local = new Date(date.getTime() - date.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 16);
}

function toIsoString(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    throw new Error("日時形式が不正です。");
  }
  return date.toISOString();
}

function isSameDateTime(left: string | null, right: string | null): boolean {
  if (left === null && right === null) {
    return true;
  }
  if (left === null || right === null) {
    return false;
  }
  const leftTime = new Date(left).getTime();
  const rightTime = new Date(right).getTime();
  if (Number.isNaN(leftTime) || Number.isNaN(rightTime)) {
    return left === right;
  }
  return leftTime === rightTime;
}

function formatDueAt(value: string | null): string {
  if (!value) {
    return "なし";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString("ja-JP");
}

export default function AppPage({ currentUser, accessToken, onLogout }: AppPageProps) {
  const [tab, setTab] = useState(0);

  const [todos, setTodos] = useState<Todo[]>([]);
  const [todosLoading, setTodosLoading] = useState(false);
  const [todosError, setTodosError] = useState<string | null>(null);
  const [hasLoadedTodos, setHasLoadedTodos] = useState(false);
  const [newTodoTitle, setNewTodoTitle] = useState("");
  const [newTodoDueAt, setNewTodoDueAt] = useState("");
  const [addingTodo, setAddingTodo] = useState(false);
  const [savingTodoId, setSavingTodoId] = useState<string | null>(null);
  const [deletingTodoId, setDeletingTodoId] = useState<string | null>(null);
  const [editingTodoId, setEditingTodoId] = useState<string | null>(null);
  const [editTitle, setEditTitle] = useState("");
  const [editDueAt, setEditDueAt] = useState("");

  const [users, setUsers] = useState<User[]>([]);
  const [usersLoading, setUsersLoading] = useState(false);
  const [usersError, setUsersError] = useState<string | null>(null);
  const [hasLoadedUsers, setHasLoadedUsers] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const navigate = useNavigate();

  const handleLogout = () => {
    onLogout();
    navigate("/");
  };

  const fetchTodos = async () => {
    setTodosLoading(true);
    setTodosError(null);
    try {
      const data = await listMyTodos(accessToken);
      setTodos(data);
    } catch (err) {
      setTodosError(
        err instanceof Error ? `Todo取得に失敗しました (${err.message})` : "Todo取得に失敗しました。"
      );
    } finally {
      setTodosLoading(false);
      setHasLoadedTodos(true);
    }
  };

  const resetEditing = () => {
    setEditingTodoId(null);
    setEditTitle("");
    setEditDueAt("");
  };

  const handleCreateTodo = async () => {
    if (newTodoTitle.trim() === "") {
      setTodosError("Todoタイトルを入力してください。");
      return;
    }

    setAddingTodo(true);
    setTodosError(null);
    try {
      const dueAt = newTodoDueAt.trim() === "" ? undefined : toIsoString(newTodoDueAt);
      const created = await registerTodo(accessToken, {
        title: newTodoTitle.trim(),
        assigneeUserId: currentUser.id,
        ...(dueAt !== undefined ? { dueAt } : {})
      });
      setTodos((prev) => [created, ...prev]);
      setNewTodoTitle("");
      setNewTodoDueAt("");
    } catch (err) {
      setTodosError(
        err instanceof Error ? `Todo追加に失敗しました (${err.message})` : "Todo追加に失敗しました。"
      );
    } finally {
      setAddingTodo(false);
    }
  };

  const handleToggleCompleted = async (todo: Todo) => {
    setSavingTodoId(todo.id);
    setTodosError(null);
    try {
      const updated = await updateTodoCompleted(accessToken, todo.id, !todo.completed);
      setTodos((prev) => prev.map((item) => (item.id === updated.id ? updated : item)));
    } catch (err) {
      setTodosError(
        err instanceof Error
          ? `完了状態の更新に失敗しました (${err.message})`
          : "完了状態の更新に失敗しました。"
      );
    } finally {
      setSavingTodoId(null);
    }
  };

  const handleStartEdit = (todo: Todo) => {
    setEditingTodoId(todo.id);
    setEditTitle(todo.title);
    setEditDueAt(toDateTimeLocalValue(todo.due_at));
    setTodosError(null);
  };

  const handleSaveEdit = async (todoId: string) => {
    const target = todos.find((todo) => todo.id === todoId);
    if (!target) {
      setTodosError("更新対象のTodoが見つかりません。");
      return;
    }
    if (editTitle.trim() === "") {
      setTodosError("Todoタイトルを入力してください。");
      return;
    }

    let parsedDueAt: string | null;
    try {
      parsedDueAt = editDueAt.trim() === "" ? null : toIsoString(editDueAt);
    } catch (err) {
      setTodosError(
        err instanceof Error ? `期限の形式が不正です (${err.message})` : "期限の形式が不正です。"
      );
      return;
    }

    const payload: {
      title?: string;
      dueAt?: string | null;
    } = {};
    const trimmedTitle = editTitle.trim();

    if (trimmedTitle !== target.title) {
      payload.title = trimmedTitle;
    }
    if (!isSameDateTime(parsedDueAt, target.due_at)) {
      payload.dueAt = parsedDueAt;
    }

    if (Object.keys(payload).length === 0) {
      setTodosError("更新項目がありません。");
      return;
    }

    setSavingTodoId(todoId);
    setTodosError(null);
    try {
      const updated = await updateTodo(accessToken, todoId, payload);
      setTodos((prev) => prev.map((item) => (item.id === updated.id ? updated : item)));
      resetEditing();
    } catch (err) {
      setTodosError(
        err instanceof Error ? `Todo更新に失敗しました (${err.message})` : "Todo更新に失敗しました。"
      );
    } finally {
      setSavingTodoId(null);
    }
  };

  const handleDeleteTodo = async (todoId: string) => {
    const ok = window.confirm("このTodoを削除しますか？");
    if (!ok) {
      return;
    }
    setDeletingTodoId(todoId);
    setTodosError(null);
    try {
      await deleteTodo(accessToken, todoId);
      setTodos((prev) => prev.filter((todo) => todo.id !== todoId));
      if (editingTodoId === todoId) {
        resetEditing();
      }
    } catch (err) {
      setTodosError(
        err instanceof Error ? `Todo削除に失敗しました (${err.message})` : "Todo削除に失敗しました。"
      );
    } finally {
      setDeletingTodoId(null);
    }
  };

  const fetchUsers = async () => {
    setUsersLoading(true);
    setUsersError(null);
    try {
      const data = await listUsers(accessToken);
      setUsers(data);
    } catch (err) {
      setUsersError(
        err instanceof Error ? `取得に失敗しました (${err.message})` : "取得に失敗しました。"
      );
    } finally {
      setUsersLoading(false);
      setHasLoadedUsers(true);
    }
  };

  const handleDelete = async (userId: string) => {
    const ok = window.confirm("このユーザを削除しますか？");
    if (!ok) {
      return;
    }
    setDeletingId(userId);
    setUsersError(null);
    try {
      await deleteUser(accessToken, userId);
      setUsers((prev) => prev.filter((user) => user.id !== userId));
    } catch (err) {
      setUsersError(
        err instanceof Error ? `削除に失敗しました (${err.message})` : "削除に失敗しました。"
      );
    } finally {
      setDeletingId(null);
    }
  };

  useEffect(() => {
    if (tab === 0 && !hasLoadedTodos && !todosLoading) {
      fetchTodos();
    }
  }, [tab, hasLoadedTodos, todosLoading]);

  useEffect(() => {
    if (tab === 2 && !hasLoadedUsers && !usersLoading) {
      fetchUsers();
    }
  }, [tab, hasLoadedUsers, usersLoading]);

  return (
    <Card>
      <CardContent>
        <Stack spacing={2}>
          <Stack
            direction={{ xs: "column", sm: "row" }}
            spacing={2}
            alignItems={{ xs: "flex-start", sm: "center" }}
            justifyContent="space-between"
          >
            <Box>
              <Typography variant="h6" fontWeight={600}>
                こんにちは、{currentUser.name}さん
              </Typography>
              <Typography variant="body2" color="text.secondary">
                ログイン中: {currentUser.email}
              </Typography>
            </Box>
            <Button variant="outlined" onClick={handleLogout}>
              ログアウト
            </Button>
          </Stack>

          <Tabs
            value={tab}
            onChange={(_, value) => setTab(value)}
            variant="fullWidth"
          >
            <Tab label="My Todos" />
            <Tab label="Profile" />
            <Tab label="Users" />
          </Tabs>

          <Divider />

          {tab === 0 && (
            <Stack spacing={2}>
              <Typography variant="h6" fontWeight={600}>
                My Todos
              </Typography>
              <Card variant="outlined">
                <CardContent>
                  <Stack spacing={2}>
                    <TextField
                      label="タイトル"
                      value={newTodoTitle}
                      onChange={(event) => setNewTodoTitle(event.target.value)}
                      fullWidth
                    />
                    <TextField
                      label="期限（任意）"
                      type="datetime-local"
                      value={newTodoDueAt}
                      onChange={(event) => setNewTodoDueAt(event.target.value)}
                      InputLabelProps={{ shrink: true }}
                      fullWidth
                    />
                    <Stack direction="row" spacing={2}>
                      <Button
                        variant="contained"
                        onClick={handleCreateTodo}
                        disabled={addingTodo}
                      >
                        {addingTodo ? "追加中..." : "Todoを追加"}
                      </Button>
                      <Button
                        variant="outlined"
                        onClick={fetchTodos}
                        disabled={todosLoading || addingTodo}
                      >
                        再取得
                      </Button>
                    </Stack>
                  </Stack>
                </CardContent>
              </Card>

              {todosError && <Alert severity="error">{todosError}</Alert>}
              {todosLoading && <Alert severity="info">読み込み中...</Alert>}
              {!todosLoading && hasLoadedTodos && todos.length === 0 && !todosError && (
                <Alert severity="info">Todoがまだありません。</Alert>
              )}

              {todos.length > 0 && (
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>完了</TableCell>
                      <TableCell>タイトル</TableCell>
                      <TableCell>期限</TableCell>
                      <TableCell>操作</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {todos.map((todo) => {
                      const isEditing = editingTodoId === todo.id;
                      const isSaving = savingTodoId === todo.id;
                      const isDeleting = deletingTodoId === todo.id;
                      return (
                        <TableRow key={todo.id}>
                          <TableCell>
                            <Checkbox
                              checked={todo.completed}
                              onChange={() => handleToggleCompleted(todo)}
                              disabled={isSaving || isDeleting}
                            />
                          </TableCell>
                          <TableCell>
                            {isEditing ? (
                              <TextField
                                value={editTitle}
                                onChange={(event) => setEditTitle(event.target.value)}
                                size="small"
                                fullWidth
                              />
                            ) : (
                              <Typography
                                sx={{
                                  textDecoration: todo.completed ? "line-through" : "none"
                                }}
                              >
                                {todo.title}
                              </Typography>
                            )}
                          </TableCell>
                          <TableCell>
                            {isEditing ? (
                              <TextField
                                type="datetime-local"
                                value={editDueAt}
                                onChange={(event) => setEditDueAt(event.target.value)}
                                size="small"
                                InputLabelProps={{ shrink: true }}
                                fullWidth
                              />
                            ) : (
                              formatDueAt(todo.due_at)
                            )}
                          </TableCell>
                          <TableCell>
                            <Stack direction="row" spacing={1}>
                              {isEditing ? (
                                <>
                                  <Button
                                    variant="contained"
                                    size="small"
                                    onClick={() => handleSaveEdit(todo.id)}
                                    disabled={isSaving}
                                  >
                                    {isSaving ? "保存中..." : "保存"}
                                  </Button>
                                  <Button
                                    variant="outlined"
                                    size="small"
                                    onClick={resetEditing}
                                    disabled={isSaving}
                                  >
                                    キャンセル
                                  </Button>
                                </>
                              ) : (
                                <>
                                  <Button
                                    variant="outlined"
                                    size="small"
                                    onClick={() => handleStartEdit(todo)}
                                    disabled={
                                      isSaving || isDeleting || editingTodoId !== null
                                    }
                                  >
                                    編集
                                  </Button>
                                  <Button
                                    color="error"
                                    variant="outlined"
                                    size="small"
                                    onClick={() => handleDeleteTodo(todo.id)}
                                    disabled={isDeleting || isSaving}
                                  >
                                    {isDeleting ? "削除中..." : "削除"}
                                  </Button>
                                </>
                              )}
                            </Stack>
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              )}
            </Stack>
          )}

          {tab === 1 && (
            <Stack spacing={2}>
              <Typography variant="h6" fontWeight={600}>
                プロファイル
              </Typography>
              <Divider />
              <Stack spacing={1}>
                <Typography>ID: {currentUser.id}</Typography>
                <Typography>名前: {currentUser.name}</Typography>
                <Typography>メール: {currentUser.email}</Typography>
              </Stack>
              <Button component={Link} to="/password" variant="outlined">
                パスワードを変更する
              </Button>
            </Stack>
          )}

          {tab === 2 && (
            <Stack spacing={2}>
              <Typography variant="h6" fontWeight={600}>
                ユーザ一覧
              </Typography>
              {usersError && <Alert severity="error">{usersError}</Alert>}
              {usersLoading && <Alert severity="info">読み込み中...</Alert>}
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>名前</TableCell>
                    <TableCell>メール</TableCell>
                    <TableCell>操作</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {users.map((user) => (
                    <TableRow key={user.id}>
                      <TableCell>{user.name}</TableCell>
                      <TableCell>{user.email}</TableCell>
                      <TableCell>
                        <Button
                          color="error"
                          variant="outlined"
                          size="small"
                          onClick={() => handleDelete(user.id)}
                          disabled={deletingId === user.id}
                        >
                          {deletingId === user.id ? "削除中..." : "削除"}
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
              {!usersLoading && users.length === 0 && !usersError && (
                <Alert severity="info">ユーザがまだいません。</Alert>
              )}
            </Stack>
          )}
        </Stack>
      </CardContent>
    </Card>
  );
}
