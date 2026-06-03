export interface Category {
  id: number;
  name: string;
  parent_id: number | null;
  sort_order: number;
  icon: string | null;
  folder_path: string | null;
  created_at: string;
  updated_at: string;
}
