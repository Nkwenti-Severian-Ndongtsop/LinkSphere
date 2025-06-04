import React, { useState, useEffect } from 'react';
import { Tag, tagsService } from '../services/tags.service';

export const TagsManager: React.FC = () => {
    const [tags, setTags] = useState<Tag[]>([]);
    const [newTagName, setNewTagName] = useState('');
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        loadTags();
    }, []);

    const loadTags = async () => {
        try {
            setLoading(true);
            const fetchedTags = await tagsService.getTags();
            setTags(fetchedTags);
            setError('');
        } catch (err: any) {
            setError(err.message || 'Failed to load tags');
        } finally {
            setLoading(false);
        }
    };

    const handleCreateTag = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!newTagName.trim()) return;

        try {
            setLoading(true);
            const newTag = await tagsService.createTag({ name: newTagName.trim() });
            setTags([...tags, newTag]);
            setNewTagName('');
            setError('');
        } catch (err: any) {
            setError(err.message || 'Failed to create tag');
        } finally {
            setLoading(false);
        }
    };

    const handleDeleteTag = async (tagId: string) => {
        try {
            setLoading(true);
            await tagsService.deleteTag(tagId);
            setTags(tags.filter(tag => tag.id !== tagId));
            setError('');
        } catch (err: any) {
            setError(err.message || 'Failed to delete tag');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="p-4">
            <h2 className="text-2xl font-bold mb-4">Manage Tags</h2>
            
            {error && (
                <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                    {error}
                </div>
            )}

            <form onSubmit={handleCreateTag} className="mb-6">
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={newTagName}
                        onChange={(e) => setNewTagName(e.target.value)}
                        placeholder="Enter tag name"
                        className="flex-1 p-2 border rounded"
                        disabled={loading}
                    />
                    <button
                        type="submit"
                        disabled={loading || !newTagName.trim()}
                        className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 disabled:bg-blue-300"
                    >
                        Add Tag
                    </button>
                </div>
            </form>

            {loading ? (
                <div className="text-center">Loading...</div>
            ) : (
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                    {tags.map(tag => (
                        <div
                            key={tag.id}
                            className="flex items-center justify-between p-3 bg-gray-100 rounded"
                        >
                            <span>{tag.name}</span>
                            <button
                                onClick={() => handleDeleteTag(tag.id)}
                                className="text-red-500 hover:text-red-700"
                                disabled={loading}
                            >
                                Delete
                            </button>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}; 