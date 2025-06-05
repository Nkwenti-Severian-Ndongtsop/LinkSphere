"use client"

import { useState, useEffect, useCallback, useMemo } from "react"
import { motion, AnimatePresence } from "framer-motion"
import {
  Menu,
  LinkIcon,
  ExternalLink,
  Search,
  Trash2,
  X,
  Plus,
  UserIcon,
  Users,
} from "lucide-react"
import { Link } from "react-router-dom"

interface ApiLink {
  id: string;
  user_id: string;
  url: string;
  title: string;
  description: string;
  click_count: number;
  favicon_url?: string;
  created_at: string;
  updated_at: string;
  uploader_name: string;
}

interface ApiUser {
  id: string;
  username: string;
  email: string;
  user_role: string;
  is_email_verified: boolean;
  created_at: string;
}

export default function AdminDashboard() {
  const [isSidebarOpen, setIsSidebarOpen] = useState(true)
  const [searchQuery, setSearchQuery] = useState("")
  const [links, setLinks] = useState<ApiLink[]>([])
  const [users, setUsers] = useState<ApiUser[]>([])
  const [loading, setLoading] = useState<boolean>(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'links' | 'users'>('links')

  const [filteredLinks, setFilteredLinks] = useState<ApiLink[]>([])
  const [filteredUsers, setFilteredUsers] = useState<ApiUser[]>([])
  const [selectedLink, setSelectedLink] = useState<ApiLink | null>(null)
  const [isDetailModalOpen, setIsDetailModalOpen] = useState(false)

  // Fetch data from backend
  const fetchData = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const [linksResponse, usersResponse] = await Promise.all([
        fetch('/api/admin/all-links', {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`
          }
        }),
        fetch('/api/admin/user-stats', {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`
          }
        })
      ]);

      if (!linksResponse.ok) {
        throw new Error(`Failed to fetch links: ${linksResponse.status} ${linksResponse.statusText}`);
      }
      if (!usersResponse.ok) {
        throw new Error(`Failed to fetch users: ${usersResponse.status} ${usersResponse.statusText}`);
      }

      const [linksData, usersData] = await Promise.all([
        linksResponse.json(),
        usersResponse.json()
      ]);

      setLinks(linksData)
      setUsers(usersData)
    } catch (e: any) {
      console.error("Error fetching data:", e)
      if (e.message.includes('Failed to fetch')) {
        setError('Unable to connect to server. Please check your internet connection.')
      } else if (e.status === 401) {
        setError('Your session has expired. Please log in again.')
      } else if (e.status === 403) {
        setError('You do not have permission to access this resource.')
      } else {
        setError(e.message || 'An unexpected error occurred. Please try again.')
      }
    } finally {
      setLoading(false)
    }
  }, [])

  // Fetch data on component mount
  useEffect(() => {
    fetchData()
  }, [fetchData])

  // Filter data based on search query
  useEffect(() => {
    if (activeTab === 'links') {
      let results = links
      if (searchQuery) {
        const lowerCaseQuery = searchQuery.toLowerCase()
        results = results.filter(
          (link) =>
            link.title.toLowerCase().includes(lowerCaseQuery) ||
            link.description.toLowerCase().includes(lowerCaseQuery) ||
            link.url.toLowerCase().includes(lowerCaseQuery) ||
            link.uploader_name.toLowerCase().includes(lowerCaseQuery)
        )
      }
      setFilteredLinks(results)
    } else {
      let results = users
      if (searchQuery) {
        const lowerCaseQuery = searchQuery.toLowerCase()
        results = results.filter(
          (user) =>
            user.username.toLowerCase().includes(lowerCaseQuery) ||
            user.email.toLowerCase().includes(lowerCaseQuery)
        )
      }
      setFilteredUsers(results)
    }
  }, [searchQuery, links, users, activeTab])

  // Delete link handler
  const handleDeleteLink = async (id: string) => {
    if (!window.confirm("Are you sure you want to delete this link?")) {
      return
    }
    try {
      const response = await fetch(`/api/admin/links/${id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        }
      })

      if (response.ok || response.status === 204) {
        setLinks(prevLinks => prevLinks.filter(link => link.id !== id))
        if (selectedLink?.id === id) {
          closeDetailModal()
        }
      } else {
        const errorText = await response.text()
        throw new Error(`Failed to delete link ${id}. Status: ${response.status}. ${errorText}`)
      }
    } catch (e: any) {
      console.error("Error deleting link:", e)
      setError(`Failed to delete link: ${e.message || 'Unknown error'}`)
    }
  }

  // Delete user handler
  const handleDeleteUser = async (id: string) => {
    if (!window.confirm("Are you sure you want to delete this user? All their links will also be deleted.")) {
      return
    }
    try {
      const response = await fetch(`/api/admin/users/${id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        }
      })

      if (response.ok || response.status === 204) {
        setUsers(prevUsers => prevUsers.filter(user => user.id !== id))
        setLinks(prevLinks => prevLinks.filter(link => link.user_id !== id))
      } else {
        const errorText = await response.text()
        throw new Error(`Failed to delete user ${id}. Status: ${response.status}. ${errorText}`)
      }
    } catch (e: any) {
      console.error("Error deleting user:", e)
      setError(`Failed to delete user: ${e.message || 'Unknown error'}`)
    }
  }

  // Modal handlers
  const openDetailModal = (link: ApiLink) => {
    setSelectedLink(link)
    setIsDetailModalOpen(true)
  }
  const closeDetailModal = () => {
    setIsDetailModalOpen(false)
    setSelectedLink(null)
  }

  // Calculate statistics
  const stats = useMemo(() => ({
    totalLinks: links.length,
    totalUsers: users.length,
    totalClicks: links.reduce((sum, link) => sum + link.click_count, 0),
    verifiedUsers: users.filter(user => user.is_email_verified).length
  }), [links, users])

  return (
    <div className="flex h-full bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 rounded-xl overflow-hidden border border-purple-100 dark:border-purple-900/30 shadow-xl">
      {/* Sidebar */}
      <motion.div
        className={`${isSidebarOpen ? "w-64" : "w-20"} bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col transition-all duration-300 ease-in-out z-20`}
        initial={false}
        animate={{ width: isSidebarOpen ? 256 : 80 }}
      >
        {/* Logo */}
        <div className="p-5 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between h-16">
          <div className="flex items-center space-x-2 overflow-hidden">
            <div className="w-10 h-10 bg-gradient-to-br from-purple-600 to-pink-500 rounded-xl flex items-center justify-center shadow-lg shadow-purple-500/20">
              <LinkIcon size={20} className="text-white" />
            </div>
            {isSidebarOpen && (
              <motion.div
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -10 }}
                transition={{ duration: 0.2, delay: 0.1 }}
              >
                <h1 className="font-bold text-xl tracking-tight bg-gradient-to-r from-purple-600 to-pink-500 bg-clip-text text-transparent whitespace-nowrap">
                  LinkSphere
                </h1>
                <p className="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">Admin Dashboard</p>
              </motion.div>
            )}
          </div>
          <button
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 transition-colors flex-shrink-0"
          >
            <Menu size={20} />
          </button>
        </div>

        {/* Navigation */}
        <div className="flex-1 overflow-y-auto">
          <nav className="p-4 space-y-2">
            <button
              onClick={() => setActiveTab('links')}
              className={`w-full flex items-center space-x-3 px-4 py-3 rounded-lg transition-colors ${
                activeTab === 'links'
                  ? 'bg-purple-50 dark:bg-purple-900/20 text-purple-600 dark:text-purple-400'
                  : 'hover:bg-gray-100 dark:hover:bg-gray-700/50'
              }`}
            >
              <LinkIcon size={20} />
              {isSidebarOpen && <span>Links</span>}
            </button>
            <button
              onClick={() => setActiveTab('users')}
              className={`w-full flex items-center space-x-3 px-4 py-3 rounded-lg transition-colors ${
                activeTab === 'users'
                  ? 'bg-purple-50 dark:bg-purple-900/20 text-purple-600 dark:text-purple-400'
                  : 'hover:bg-gray-100 dark:hover:bg-gray-700/50'
              }`}
            >
              <Users size={20} />
              {isSidebarOpen && <span>Users</span>}
            </button>
          </nav>
        </div>

        {/* Stats */}
        {isSidebarOpen && (
          <div className="p-4 border-t border-gray-200 dark:border-gray-700">
            <div className="space-y-4">
              <div className="bg-gray-50 dark:bg-gray-700/50 p-4 rounded-lg">
                <h4 className="text-sm font-medium text-gray-600 dark:text-gray-300">Statistics</h4>
                <div className="mt-2 space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">Total Links</span>
                    <span className="font-medium">{stats.totalLinks}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">Total Users</span>
                    <span className="font-medium">{stats.totalUsers}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">Total Clicks</span>
                    <span className="font-medium">{stats.totalClicks}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">Verified Users</span>
                    <span className="font-medium">{stats.verifiedUsers}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </motion.div>

      {/* Main Content */}
      <div className="flex-1 overflow-hidden">
        {/* Header */}
        <div className="h-16 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between px-6">
          <h2 className="text-xl font-semibold">
            {activeTab === 'links' ? 'All Links' : 'All Users'}
          </h2>
          <div className="flex items-center space-x-4">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" size={20} />
              <input
                type="text"
                placeholder={`Search ${activeTab}...`}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10 pr-4 py-2 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 dark:focus:ring-purple-400 focus:border-transparent"
              />
            </div>
            {activeTab === 'links' && (
              <Link
                to="/upload"
                className="inline-flex items-center px-4 py-2 bg-gradient-to-r from-purple-600 to-pink-500 text-white rounded-lg shadow-lg shadow-purple-500/20 hover:shadow-purple-500/40 transition-shadow"
              >
                <Plus size={20} className="mr-2" />
                Add New Link
              </Link>
            )}
          </div>
        </div>

        {/* Content */}
        <div className="p-6 overflow-auto" style={{ height: 'calc(100vh - 4rem)' }}>
          {loading ? (
            <div className="flex items-center justify-center h-full">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500"></div>
            </div>
          ) : error ? (
            <div className="text-center text-red-500">{error}</div>
          ) : activeTab === 'links' ? (
            // Links Table
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
              <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                <thead className="bg-gray-50 dark:bg-gray-700">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Title</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">URL</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Uploader</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Clicks</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Created At</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Actions</th>
                  </tr>
                </thead>
                <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                  {filteredLinks.map((link) => (
                    <tr key={link.id} className="hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="flex items-center">
                          {link.favicon_url && (
                            <img src={link.favicon_url} alt="" className="h-5 w-5 rounded-full mr-2" />
                          )}
                          <span className="font-medium">{link.title}</span>
                        </div>
                      </td>
                      <td className="px-6 py-4">
                        <a
                          href={link.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-purple-600 dark:text-purple-400 hover:underline flex items-center"
                        >
                          <span className="truncate max-w-xs">{link.url}</span>
                          <ExternalLink size={14} className="ml-1 flex-shrink-0" />
                        </a>
                      </td>
                      <td className="px-6 py-4">{link.uploader_name}</td>
                      <td className="px-6 py-4">{link.click_count}</td>
                      <td className="px-6 py-4">
                        {new Date(link.created_at).toLocaleDateString()}
                      </td>
                      <td className="px-6 py-4 text-right">
                        <button
                          onClick={() => handleDeleteLink(link.id)}
                          className="text-red-600 hover:text-red-900 dark:text-red-400 dark:hover:text-red-300"
                        >
                          <Trash2 size={18} />
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            // Users Table
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
              <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                <thead className="bg-gray-50 dark:bg-gray-700">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Username</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Email</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Role</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Status</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Joined</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Actions</th>
                  </tr>
                </thead>
                <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                  {filteredUsers.map((user) => (
                    <tr key={user.id} className="hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                      <td className="px-6 py-4">
                        <div className="flex items-center">
                          <div className="h-8 w-8 rounded-full bg-gradient-to-br from-purple-600 to-pink-500 flex items-center justify-center">
                            <UserIcon size={16} className="text-white" />
                          </div>
                          <span className="ml-2 font-medium">{user.username}</span>
                        </div>
                      </td>
                      <td className="px-6 py-4">{user.email}</td>
                      <td className="px-6 py-4">
                        <span className={`px-2 py-1 text-xs rounded-full ${
                          user.user_role === 'admin'
                            ? 'bg-purple-100 dark:bg-purple-900/20 text-purple-700 dark:text-purple-400'
                            : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
                        }`}>
                          {user.user_role}
                        </span>
                      </td>
                      <td className="px-6 py-4">
                        <span className={`px-2 py-1 text-xs rounded-full ${
                          user.is_email_verified
                            ? 'bg-green-100 dark:bg-green-900/20 text-green-700 dark:text-green-400'
                            : 'bg-yellow-100 dark:bg-yellow-900/20 text-yellow-700 dark:text-yellow-400'
                        }`}>
                          {user.is_email_verified ? 'Verified' : 'Pending'}
                        </span>
                      </td>
                      <td className="px-6 py-4">
                        {new Date(user.created_at).toLocaleDateString()}
                      </td>
                      <td className="px-6 py-4 text-right">
                        {user.user_role !== 'admin' && (
                          <button
                            onClick={() => handleDeleteUser(user.id)}
                            className="text-red-600 hover:text-red-900 dark:text-red-400 dark:hover:text-red-300"
                          >
                            <Trash2 size={18} />
                          </button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>

      {/* Link Detail Modal */}
      <AnimatePresence>
        {isDetailModalOpen && selectedLink && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50"
          >
            <motion.div
              initial={{ scale: 0.9, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.9, opacity: 0 }}
              className="bg-white dark:bg-gray-800 rounded-xl p-6 max-w-lg w-full"
            >
              <div className="flex justify-between items-start mb-4">
                <h3 className="text-xl font-semibold">{selectedLink.title}</h3>
                <button
                  onClick={closeDetailModal}
                  className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                >
                  <X size={20} />
                </button>
              </div>
              <div className="space-y-4">
                <p className="text-gray-600 dark:text-gray-300">{selectedLink.description}</p>
                <div className="flex items-center justify-between text-sm text-gray-500 dark:text-gray-400">
                  <span>Uploaded by {selectedLink.uploader_name}</span>
                  <span>{selectedLink.click_count} clicks</span>
                </div>
                <div className="flex justify-end space-x-2">
                  <a
                    href={selectedLink.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
                  >
                    Visit Link
                  </a>
                  <button
                    onClick={() => {
                      handleDeleteLink(selectedLink.id)
                      closeDetailModal()
                    }}
                    className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                  >
                    Delete
                  </button>
                </div>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}
