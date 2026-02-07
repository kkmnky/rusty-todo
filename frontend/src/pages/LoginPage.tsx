import { useEffect, useState } from "react";
import {
  Alert,
  Button,
  Card,
  CardContent,
  Stack,
  TextField,
  Typography
} from "@mui/material";
import { Link, useNavigate } from "react-router-dom";
import type { User } from "../types";

type LoginPageProps = {
  prefillEmail: string;
  loginError: string | null;
  currentUser: User | null;
  onLogin: (email: string, password: string) => Promise<boolean>;
};

export default function LoginPage({
  prefillEmail,
  loginError,
  currentUser,
  onLogin
}: LoginPageProps) {
  const [email, setEmail] = useState(prefillEmail);
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    setEmail(prefillEmail);
  }, [prefillEmail]);

  const handleSubmit = async () => {
    setLoading(true);
    const success = await onLogin(email, password);
    setLoading(false);
    if (success) {
      setPassword("");
      navigate("/app");
    }
  };

  return (
    <Card>
      <CardContent>
        <Stack spacing={2}>
          <Typography variant="h6" fontWeight={600}>
            ログイン
          </Typography>
          {loginError && <Alert severity="error">{loginError}</Alert>}
          {currentUser && (
            <Alert severity="success">
              ログイン中: {currentUser.name} ({currentUser.email})
            </Alert>
          )}
          <TextField
            label="メールアドレス"
            value={email}
            onChange={(event) => setEmail(event.target.value)}
            fullWidth
          />
          <TextField
            label="パスワード"
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            fullWidth
          />
          <Stack direction="row" spacing={2}>
            <Button variant="contained" onClick={handleSubmit} disabled={loading}>
              {loading ? "送信中..." : "ログイン"}
            </Button>
            <Button component={Link} to="/register" variant="outlined">
              ユーザ登録はこちら
            </Button>
          </Stack>
        </Stack>
      </CardContent>
    </Card>
  );
}
