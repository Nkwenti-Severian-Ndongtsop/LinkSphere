import React, { useState, useEffect } from 'react';
import { UserProfile, profileService } from '../services/profile.service';

export const ProfileManager: React.FC = () => {
    const [profile, setProfile] = useState<UserProfile | null>(null);
    const [editMode, setEditMode] = useState(false);
    const [formData, setFormData] = useState({
        username: '',
        email: '',
    });
    const [passwordData, setPasswordData] = useState({
        current_password: '',
        new_password: '',
        confirm_password: '',
    });
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        loadProfile();
    }, []);

    const loadProfile = async () => {
        try {
            setLoading(true);
            const userProfile = await profileService.getProfile();
            setProfile(userProfile);
            setFormData({
                username: userProfile.username,
                email: userProfile.email,
            });
            setError('');
        } catch (err: any) {
            setError(err.message || 'Failed to load profile');
        } finally {
            setLoading(false);
        }
    };

    const handleUpdateProfile = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            setLoading(true);
            const updatedProfile = await profileService.updateProfile({
                username: formData.username !== profile?.username ? formData.username : undefined,
                email: formData.email !== profile?.email ? formData.email : undefined,
            });
            setProfile(updatedProfile);
            setEditMode(false);
            setSuccess('Profile updated successfully');
            setError('');
        } catch (err: any) {
            setError(err.message || 'Failed to update profile');
        } finally {
            setLoading(false);
        }
    };

    const handleChangePassword = async (e: React.FormEvent) => {
        e.preventDefault();
        if (passwordData.new_password !== passwordData.confirm_password) {
            setError('New passwords do not match');
            return;
        }

        try {
            setLoading(true);
            await profileService.changePassword({
                current_password: passwordData.current_password,
                new_password: passwordData.new_password,
            });
            setPasswordData({
                current_password: '',
                new_password: '',
                confirm_password: '',
            });
            setSuccess('Password changed successfully');
            setError('');
        } catch (err: any) {
            setError(err.message || 'Failed to change password');
        } finally {
            setLoading(false);
        }
    };

    if (loading && !profile) {
        return <div className="text-center p-4">Loading...</div>;
    }

    return (
        <div className="p-4 max-w-2xl mx-auto">
            <h2 className="text-2xl font-bold mb-4">Profile Settings</h2>
            
            {error && (
                <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                    {error}
                </div>
            )}

            {success && (
                <div className="bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded mb-4">
                    {success}
                </div>
            )}

            <div className="bg-white shadow rounded-lg p-6 mb-6">
                <h3 className="text-lg font-semibold mb-4">Profile Information</h3>
                <form onSubmit={handleUpdateProfile} className="space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-gray-700">Username</label>
                        <input
                            type="text"
                            value={formData.username}
                            onChange={(e) => setFormData({ ...formData, username: e.target.value })}
                            disabled={!editMode || loading}
                            className="mt-1 block w-full p-2 border rounded"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-gray-700">Email</label>
                        <input
                            type="email"
                            value={formData.email}
                            onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                            disabled={!editMode || loading}
                            className="mt-1 block w-full p-2 border rounded"
                        />
                    </div>
                    {editMode ? (
                        <div className="flex gap-2">
                            <button
                                type="submit"
                                disabled={loading}
                                className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 disabled:bg-blue-300"
                            >
                                Save Changes
                            </button>
                            <button
                                type="button"
                                onClick={() => {
                                    setEditMode(false);
                                    setFormData({
                                        username: profile?.username || '',
                                        email: profile?.email || '',
                                    });
                                }}
                                className="bg-gray-500 text-white px-4 py-2 rounded hover:bg-gray-600"
                            >
                                Cancel
                            </button>
                        </div>
                    ) : (
                        <button
                            type="button"
                            onClick={() => setEditMode(true)}
                            className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
                        >
                            Edit Profile
                        </button>
                    )}
                </form>
            </div>

            <div className="bg-white shadow rounded-lg p-6">
                <h3 className="text-lg font-semibold mb-4">Change Password</h3>
                <form onSubmit={handleChangePassword} className="space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-gray-700">Current Password</label>
                        <input
                            type="password"
                            value={passwordData.current_password}
                            onChange={(e) => setPasswordData({ ...passwordData, current_password: e.target.value })}
                            disabled={loading}
                            className="mt-1 block w-full p-2 border rounded"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-gray-700">New Password</label>
                        <input
                            type="password"
                            value={passwordData.new_password}
                            onChange={(e) => setPasswordData({ ...passwordData, new_password: e.target.value })}
                            disabled={loading}
                            className="mt-1 block w-full p-2 border rounded"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-gray-700">Confirm New Password</label>
                        <input
                            type="password"
                            value={passwordData.confirm_password}
                            onChange={(e) => setPasswordData({ ...passwordData, confirm_password: e.target.value })}
                            disabled={loading}
                            className="mt-1 block w-full p-2 border rounded"
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={loading || !passwordData.current_password || !passwordData.new_password || !passwordData.confirm_password}
                        className="w-full bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 disabled:bg-blue-300"
                    >
                        Change Password
                    </button>
                </form>
            </div>
        </div>
    );
}; 