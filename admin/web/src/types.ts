export type Overview = {
  total_bots: number;
  online_bots: number;
  messages_today: number;
  forward_failures_today: number;
  last_heartbeat_at: string | null;
};

export type BotListItem = {
  bot_id: string;
  status: string;
  is_online: boolean;
  last_heartbeat_at: string | null;
  last_heartbeat_display: string;
  messages_today: number;
  forward_failures_today: number;
  updated_at: string;
};

export type BotListResponse = {
  bots: BotListItem[];
};

export type BotSession = {
  session_id: string;
  user_id: string;
  status: string;
  created_at: string;
  updated_at: string;
};

export type BotDetail = {
  bot_id: string;
  status: string;
  is_online: boolean;
  can_start: boolean;
  has_runtime: boolean;
  has_qr_url: boolean;
  heartbeat_display: string;
  created_at: string;
  updated_at: string;
  register_link: string;
  register_qr_image_url: string | null;
  sessions: BotSession[];
};

export type ForwardPolicy = {
  bot_id: string;
  forwarding_enabled: boolean;
  allowed_targets: string[];
};

export type CreateBotResponse = {
  bot_id: string;
  detail_api: string;
  register_link: string;
};

export type SessionHistoryRow = {
  received_at: string;
  from_user_id: string;
  to_user_id: string;
  content_type: string;
  text_content: string;
  direction: string;
};

export type SessionHistory = {
  session_id: string;
  bot_id: string;
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
  rows: SessionHistoryRow[];
};

export type SystemLogPayload = {
  source: string;
  requested_lines: number;
  returned_lines: number;
  lines: string[];
};
