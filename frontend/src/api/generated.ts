export type Locale = "en" | "es" | "fr" | "de";
export type Channel = "email" | "sms" | "push";
export type Frequency = "once" | "daily" | "weekly" | "hourly" | "monthly";
export type Platform = "ios" | "android" | "web";

export interface HealthResponse {
  status: "ok";
  version: string;
}

export interface ReadyResponse {
  status: "ok";
  version: string;
  database: "connected";
}

export interface ErrorResponse {
  error: string;
  status_code?: number;
}

export interface ReminderPreferences {
  vault_id: string;
  channels: Channel[];
  hours_before_expiry: number;
  frequency: Frequency;
}

export interface SetPreferencesRequest {
  channels: Channel[];
  hours_before_expiry: number;
  frequency: Frequency;
}

export interface RegisterTokenRequest {
  owner: string;
  token: string;
  platform: Platform;
}

export interface DeviceToken {
  owner: string;
  token: string;
  platform: Platform;
  registered_at: string;
}

export interface CsrfTokenResponse {
  csrf_token: string;
}
