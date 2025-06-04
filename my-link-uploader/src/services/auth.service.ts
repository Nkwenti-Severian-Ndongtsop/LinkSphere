import { apiRequest } from '../utils/api';

interface LoginCredentials {
    email: string;
    password: string;
}

interface RegisterData {
    username: string;
    email: string;
    password: string;
}

interface AuthResponse {
    user: {
        id: string;
        username: string;
        email: string;
        role: string;
    };
    token: string;
}

export const authService = {
    login: (credentials: LoginCredentials) =>
        apiRequest<AuthResponse>('/api/auth/login', {
            method: 'POST',
            body: JSON.stringify(credentials),
        }),

    register: (data: RegisterData) =>
        apiRequest<AuthResponse>('/api/auth/register', {
            method: 'POST',
            body: JSON.stringify(data),
        }),

    logout: () =>
        apiRequest('/api/auth/logout', {
            method: 'POST',
        }),

    verifyEmail: (token: string) =>
        apiRequest('/api/auth/verify-email', {
            method: 'POST',
            body: JSON.stringify({ token }),
        }),
}; 