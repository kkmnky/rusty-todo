import { createTheme } from "@mui/material/styles";

export const theme = createTheme({
  palette: {
    mode: "light",
    primary: { main: "#0f766e" },
    secondary: { main: "#f59e0b" },
    background: { default: "#f7f7f2", paper: "#ffffff" }
  },
  typography: {
    fontFamily: "\"Noto Sans JP\", \"Hiragino Sans\", \"Segoe UI\", sans-serif"
  },
  shape: {
    borderRadius: 10
  }
});
