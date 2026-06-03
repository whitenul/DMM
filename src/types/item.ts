export interface Item {
  id: number;
  category_id: number;
  name: string;
  pinyin_name: string | null;
  item_type: "App" | "File" | "Folder" | "Web";
  path: string;
  icon_path: string | null;
  arguments: string | null;
  working_dir: string | null;
  sort_order: number;
  is_pinned: boolean;
  created_at: string;
  updated_at: string;
}
