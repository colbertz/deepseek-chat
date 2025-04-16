export interface LoginRequest {
  email: string;
  password: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

export interface User {
  id?: number;
  email: string;
  role: string;
}

export interface RefreshRequest {
  refresh_token: string;
}
