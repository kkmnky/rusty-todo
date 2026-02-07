import type { Todo } from "../types";

export const SAMPLE_TODOS: Todo[] = [
  {
    id: "todo-1",
    title: "API仕様の棚卸し",
    status: "In Progress",
    priority: "High",
    due: "2026-02-10"
  },
  {
    id: "todo-2",
    title: "画面フローのラフ整理",
    status: "Todo",
    priority: "Medium",
    due: "2026-02-13"
  },
  {
    id: "todo-3",
    title: "認証方式の決定",
    status: "Blocked",
    priority: "High",
    due: "2026-02-15"
  }
];
