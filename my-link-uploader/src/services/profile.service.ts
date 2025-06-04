import { apiRequest } from '../utils/api';

export interface UserProfile {
    id: string;
    username: string;
    email: string;
    is_email_verified: boolean;
    created_at: string;
}

export interface UpdateProfileData {
    username?: string;
    email?: string;
}

export interface ChangePasswordData {
    current_password: string;
    new_password: string;
}

export const profileService = {
    getProfile: () =>
        apiRequest<UserProfile>('/api/users/me', {
            method: 'GET',
        }),

    updateProfile: (data: UpdateProfileData) =>
        apiRequest<UserProfile>('/api/users/me', {
            method: 'PUT',
            body: JSON.stringify(data),
        }),

    changePassword: (data: ChangePasswordData) =>
        apiRequest('/api/users/me/password', {
            method: 'PUT',
            body: JSON.stringify(data),
        }),
}; 