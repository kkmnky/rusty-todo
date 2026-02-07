import { Navigate } from "react-router-dom";
import type { User } from "../types";

type RequireLoginProps = {
  currentUser: User | null;
  children: JSX.Element;
};

export default function RequireLogin({
  currentUser,
  children
}: RequireLoginProps) {
  if (!currentUser) {
    return <Navigate to="/" replace />;
  }
  return children;
}
