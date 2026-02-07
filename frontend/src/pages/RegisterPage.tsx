import { useState } from "react";
import {
  Alert,
  Button,
  Card,
  CardContent,
  Stack,
  TextField,
  Typography
} from "@mui/material";
import { useNavigate } from "react-router-dom";
import { registerUser } from "../api";

type RegisterPageProps = {
  onRegistered: (email: string) => void;
};

export default function RegisterPage({ onRegistered }: RegisterPageProps) {
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [registerError, setRegisterError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  const handleRegister = async () => {
    if (name.trim() === "" || email.trim() === "" || password.trim() === "") {
      setRegisterError("ユーザ名・メール・パスワードを入力してください。");
      return;
    }
    setLoading(true);
    try {
      await registerUser(name.trim(), email.trim(), password);
      setRegisterError(null);
      setPassword("");
      onRegistered(email.trim());
      navigate("/");
    } catch (err) {
      setRegisterError(
        err instanceof Error ? `登録に失敗しました (${err.message})` : "登録に失敗しました。"
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card>
      <CardContent>
        <Stack spacing={2}>
          <Typography variant="h6" fontWeight={600}>
            ユーザ登録
          </Typography>
          {registerError && <Alert severity="error">{registerError}</Alert>}
          <TextField
            label="ユーザ名"
            value={name}
            onChange={(event) => setName(event.target.value)}
            fullWidth
          />
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
            <Button variant="contained" onClick={handleRegister} disabled={loading}>
              {loading ? "送信中..." : "登録"}
            </Button>
            <Button variant="outlined" onClick={() => navigate("/")}>
              ログインへ戻る
            </Button>
          </Stack>
        </Stack>
      </CardContent>
    </Card>
  );
}
