import { apiRequest } from '../utils/api';

export interface Tag {
    id: string;
    name: string;
    user_id: string;
}

export interface CreateTagData {
    name: string;
}

export const tagsService = {
    getTags: () =>
        apiRequest<Tag[]>('/api/tags', {
            method: 'GET',
        }),

    createTag: (data: CreateTagData) =>
        apiRequest<Tag>('/api/tags', {
            method: 'POST',
            body: JSON.stringify(data),
        }),

    deleteTag: (id: string) =>
        apiRequest(`/api/tags/${id}`, {
            method: 'DELETE',
        }),
}; 