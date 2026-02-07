import { useEffect, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  Divider,
  List,
  ListItem,
  ListItemText,
  Stack,
  Tab,
  Tabs,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  Typography
} from "@mui/material";
import { Link, useNavigate } from "react-router-dom";
import { SAMPLE_TODOS } from "../data/sample";
import { deleteUser, listUsers } from "../api";
import type { User } from "../types";

type AppPageProps = {
  currentUser: User;
  accessToken: string;
  onLogout: () => void;
};

export default function AppPage({ currentUser, accessToken, onLogout }: AppPageProps) {
  const [tab, setTab] = useState(0);
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

  const fetchUsers = async () => {
    setUsersLoading(true);
    setUsersError(null);
    try {
      const data = await listUsers(accessToken);
      setUsers(data);
      setHasLoadedUsers(true);
    } catch (err) {
      setUsersError(
        err instanceof Error ? `取得に失敗しました (${err.message})` : "取得に失敗しました。"
      );
    } finally {
      setUsersLoading(false);
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
              <Alert severity="warning">
                Todo APIは未実装のため、仮データを表示しています。
              </Alert>
              <List>
                {SAMPLE_TODOS.map((todo) => (
                  <ListItem key={todo.id} divider>
                    <ListItemText
                      primary={todo.title}
                      secondary={`Status: ${todo.status} | Priority: ${todo.priority} | Due: ${todo.due}`}
                    />
                  </ListItem>
                ))}
              </List>
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
