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
import { changePassword } from "../api";

type ChangePasswordPageProps = {
  accessToken: string;
};

export default function ChangePasswordPage({ accessToken }: ChangePasswordPageProps) {
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  const handleSubmit = async () => {
    if (currentPassword.trim() === "" || newPassword.trim() === "") {
      setError("現在のパスワードと新しいパスワードを入力してください。");
      setSuccess(false);
      return;
    }
    if (newPassword !== confirmPassword) {
      setError("新しいパスワードが一致しません。");
      setSuccess(false);
      return;
    }
    setLoading(true);
    try {
      await changePassword(accessToken, currentPassword, newPassword);
      setError(null);
      setSuccess(true);
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
    } catch (err) {
      setError(
        err instanceof Error
          ? `パスワード変更に失敗しました (${err.message})`
          : "パスワード変更に失敗しました。"
      );
      setSuccess(false);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card>
      <CardContent>
        <Stack spacing={2}>
          <Typography variant="h6" fontWeight={600}>
            パスワード変更
          </Typography>
          {error && <Alert severity="error">{error}</Alert>}
          {success && <Alert severity="success">変更しました。</Alert>}
          <TextField
            label="現在のパスワード"
            type="password"
            value={currentPassword}
            onChange={(event) => setCurrentPassword(event.target.value)}
            fullWidth
          />
          <TextField
            label="新しいパスワード"
            type="password"
            value={newPassword}
            onChange={(event) => setNewPassword(event.target.value)}
            fullWidth
          />
          <TextField
            label="新しいパスワード（確認）"
            type="password"
            value={confirmPassword}
            onChange={(event) => setConfirmPassword(event.target.value)}
            fullWidth
          />
          <Stack direction="row" spacing={2}>
            <Button variant="contained" onClick={handleSubmit} disabled={loading}>
              {loading ? "送信中..." : "変更する"}
            </Button>
            <Button variant="outlined" onClick={() => navigate("/app")}>
              戻る
            </Button>
          </Stack>
        </Stack>
      </CardContent>
    </Card>
  );
}
