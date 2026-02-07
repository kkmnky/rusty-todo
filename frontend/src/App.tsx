import { useState } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import AppPage from "./pages/AppPage";
import ChangePasswordPage from "./pages/ChangePasswordPage";
import LoginPage from "./pages/LoginPage";
import RegisterPage from "./pages/RegisterPage";
import RequireLogin from "./routes/RequireLogin";
import Layout from "./components/Layout";
import type { User } from "./types";
import { getCurrentUser, login } from "./api";

export default function App() {
  const [currentUser, setCurrentUser] = useState<User | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [loginError, setLoginError] = useState<string | null>(null);
  const [prefillEmail, setPrefillEmail] = useState("");

  const handleLogin = async (email: string, password: string) => {
    if (email.trim() === "" || password.trim() === "") {
      setLoginError("メールアドレスとパスワードを入力してください。");
      return false;
    }
    try {
      const result = await login(email.trim(), password);
      setAccessToken(result.access_token);
      const user = await getCurrentUser(result.access_token);
      setCurrentUser(user);
      setLoginError(null);
      return true;
    } catch (err) {
      setLoginError(
        err instanceof Error ? `ログインに失敗しました (${err.message})` : "ログインに失敗しました。"
      );
      setAccessToken(null);
      setCurrentUser(null);
      return false;
    }
  };

  const handleRegistered = (email: string) => {
    setPrefillEmail(email);
  };

  const handleLogout = () => {
    setCurrentUser(null);
    setAccessToken(null);
  };

  return (
    <BrowserRouter>
      <Layout>
        <Routes>
          <Route
            path="/"
            element={
              <LoginPage
                prefillEmail={prefillEmail}
                loginError={loginError}
                currentUser={currentUser}
                onLogin={handleLogin}
              />
            }
          />
          <Route
            path="/register"
            element={<RegisterPage onRegistered={handleRegistered} />}
          />
          <Route
            path="/app"
            element={
              <RequireLogin currentUser={currentUser}>
                <AppPage
                  currentUser={currentUser!}
                  accessToken={accessToken!}
                  onLogout={handleLogout}
                />
              </RequireLogin>
            }
          />
          <Route
            path="/password"
            element={
              <RequireLogin currentUser={currentUser}>
                <ChangePasswordPage accessToken={accessToken!} />
              </RequireLogin>
            }
          />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
}
