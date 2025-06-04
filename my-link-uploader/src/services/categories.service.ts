import { apiRequest } from '../utils/api';

export interface Category {
    id: string;
    name: string;
    description?: string;
    user_id: string;
}

export interface CreateCategoryData {
    name: string;
    description?: string;
}

export const categoriesService = {
    getCategories: () =>
        apiRequest<Category[]>('/api/categories', {
            method: 'GET',
        }),

    createCategory: (data: CreateCategoryData) =>
        apiRequest<Category>('/api/categories', {
            method: 'POST',
            body: JSON.stringify(data),
        }),

    deleteCategory: (id: string) =>
        apiRequest(`/api/categories/${id}`, {
            method: 'DELETE',
        }),
}; 