import { apiRequest } from '../utils/api';

export interface Link {
    id: string;
    url: string;
    title: string;
    description: string;
    click_count: number;
    favicon_url?: string;
    uploader_name: string;
}

export interface CreateLinkData {
    url: string;
    title: string;
    description: string;
    favicon_url?: string;
    tags?: string[];
    categories?: string[];
}

export interface UpdateLinkData {
    title?: string;
    description?: string;
    url?: string;
    favicon_url?: string;
}

export const linksService = {
    getLinks: () =>
        apiRequest<Link[]>('/api/links', {
            method: 'GET',
        }),

    createLink: (data: CreateLinkData) =>
        apiRequest<Link>('/api/links', {
            method: 'POST',
            body: JSON.stringify(data),
        }),

    updateLink: (id: string, data: UpdateLinkData) =>
        apiRequest<Link>(`/api/links/${id}`, {
            method: 'PUT',
            body: JSON.stringify(data),
        }),

    deleteLink: (id: string) =>
        apiRequest(`/api/links/${id}`, {
            method: 'DELETE',
        }),

    incrementClickCount: (id: string) =>
        apiRequest(`/api/links/${id}/click`, {
            method: 'POST',
        }),

    getLinkTags: (id: string) =>
        apiRequest(`/api/links/${id}/tags`, {
            method: 'GET',
        }),

    addTag: (linkId: string, tagId: string) =>
        apiRequest(`/api/links/${linkId}/tags/${tagId}`, {
            method: 'POST',
        }),

    removeTag: (linkId: string, tagId: string) =>
        apiRequest(`/api/links/${linkId}/tags/${tagId}`, {
            method: 'DELETE',
        }),
}; 