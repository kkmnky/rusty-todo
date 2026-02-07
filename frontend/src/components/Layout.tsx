import type { ReactNode } from "react";
import { Box, Container, Stack, Typography } from "@mui/material";

type LayoutProps = {
  children: ReactNode;
};

export default function Layout({ children }: LayoutProps) {
  return (
    <Box
      sx={{
        minHeight: "100vh",
        py: { xs: 4, md: 6 },
        background:
          "linear-gradient(180deg, rgba(247,247,242,1) 0%, rgba(236,242,245,1) 100%)"
      }}
    >
      <Container maxWidth="md">
        <Stack spacing={3}>
          <Box>
            <Typography variant="h4" fontWeight={700}>
              Rusty Todo
            </Typography>
            <Typography variant="body1" color="text.secondary">
              ログイン後に自分のTodoとユーザ管理を確認できます。
            </Typography>
          </Box>
          {children}
        </Stack>
      </Container>
    </Box>
  );
}
