import React, { useState, useEffect } from 'react';
import { Category, categoriesService } from '../services/categories.service';

export const CategoriesManager: React.FC = () => {
    const [categories, setCategories] = useState<Category[]>([]);
    const [newCategory, setNewCategory] = useState({ name: '', description: '' });
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        loadCategories();
    }, []);

    const loadCategories = async () => {
        try {
            setLoading(true);
            const fetchedCategories = await categoriesService.getCategories();
            setCategories(fetchedCategories);
            setError('');
        } catch (err: any) {
            setError(err.message || 'Failed to load categories');
        } finally {
            setLoading(false);
        }
    };

    const handleCreateCategory = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!newCategory.name.trim()) return;

        try {
            setLoading(true);
            const createdCategory = await categoriesService.createCategory({
                name: newCategory.name.trim(),
                description: newCategory.description.trim() || undefined,
            });
            setCategories([...categories, createdCategory]);
            setNewCategory({ name: '', description: '' });
            setError('');
        } catch (err: any) {
            setError(err.message || 'Failed to create category');
        } finally {
            setLoading(false);
        }
    };

    const handleDeleteCategory = async (categoryId: string) => {
        try {
            setLoading(true);
            await categoriesService.deleteCategory(categoryId);
            setCategories(categories.filter(category => category.id !== categoryId));
            setError('');
        } catch (err: any) {
            setError(err.message || 'Failed to delete category');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="p-4">
            <h2 className="text-2xl font-bold mb-4">Manage Categories</h2>
            
            {error && (
                <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                    {error}
                </div>
            )}

            <form onSubmit={handleCreateCategory} className="mb-6 space-y-4">
                <div>
                    <input
                        type="text"
                        value={newCategory.name}
                        onChange={(e) => setNewCategory({ ...newCategory, name: e.target.value })}
                        placeholder="Category name"
                        className="w-full p-2 border rounded"
                        disabled={loading}
                    />
                </div>
                <div>
                    <textarea
                        value={newCategory.description}
                        onChange={(e) => setNewCategory({ ...newCategory, description: e.target.value })}
                        placeholder="Category description (optional)"
                        className="w-full p-2 border rounded"
                        disabled={loading}
                        rows={3}
                    />
                </div>
                <button
                    type="submit"
                    disabled={loading || !newCategory.name.trim()}
                    className="w-full bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 disabled:bg-blue-300"
                >
                    Add Category
                </button>
            </form>

            {loading ? (
                <div className="text-center">Loading...</div>
            ) : (
                <div className="grid gap-4">
                    {categories.map(category => (
                        <div
                            key={category.id}
                            className="p-4 bg-gray-100 rounded"
                        >
                            <div className="flex justify-between items-start">
                                <div>
                                    <h3 className="font-semibold">{category.name}</h3>
                                    {category.description && (
                                        <p className="text-gray-600 mt-1">{category.description}</p>
                                    )}
                                </div>
                                <button
                                    onClick={() => handleDeleteCategory(category.id)}
                                    className="text-red-500 hover:text-red-700"
                                    disabled={loading}
                                >
                                    Delete
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}; 