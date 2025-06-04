"use client"

import { useState, useEffect, useCallback } from "react"
import { motion } from "framer-motion"
import {
  LinkIcon,
  Search,
  Plus,
  ExternalLink,
  Trash2,
  Eye,
  Tag,
  FolderOpen,
  Settings,
} from "lucide-react"
import { Link } from "react-router-dom"
import { linksService } from "../services/links.service"
import { Tag as TagType } from "../services/tags.service"
import { Category } from "../services/categories.service"
import { TagsManager } from "./TagsManager"
import { CategoriesManager } from "./CategoriesManager"
import { ProfileManager } from "./ProfileManager"

interface ApiLink {
  id: string;
  url: string;
  title: string;
  description: string;
  click_count: number;
  favicon_url?: string;
  created_at: string;
  tags?: TagType[];
  categories?: Category[];
}

export default function UserDashboard() {
  const [searchQuery, setSearchQuery] = useState("")
  const [links, setLinks] = useState<ApiLink[]>([])
  const [loading, setLoading] = useState<boolean>(true)
  const [error, setError] = useState<string | null>(null)
  const [filteredLinks, setFilteredLinks] = useState<ApiLink[]>([])
  const [activeTab, setActiveTab] = useState<'links' | 'tags' | 'categories' | 'profile'>('links')

  // Fetch user's links
  const fetchLinks = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await linksService.getLinks()
      setLinks(data)
    } catch (e: any) {
      console.error("Error fetching links:", e)
      setError(e instanceof Error ? e.message : 'An error occurred')
    } finally {
      setLoading(false)
    }
  }, [])

  // Fetch links on component mount
  useEffect(() => {
    fetchLinks()
  }, [fetchLinks])

  // Filter links based on search query
  useEffect(() => {
    let results = links
    if (searchQuery) {
      const lowerCaseQuery = searchQuery.toLowerCase()
      results = results.filter(
        (link) =>
          link.title.toLowerCase().includes(lowerCaseQuery) ||
          (link.description && link.description.toLowerCase().includes(lowerCaseQuery)) ||
          link.url.toLowerCase().includes(lowerCaseQuery) ||
          link.tags?.some(tag => tag.name.toLowerCase().includes(lowerCaseQuery)) ||
          link.categories?.some(category => category.name.toLowerCase().includes(lowerCaseQuery))
      )
    }
    setFilteredLinks(results)
  }, [searchQuery, links])

  // Delete link handler
  const handleDelete = async (id: string) => {
    if (!window.confirm("Are you sure you want to delete this link?")) {
      return
    }
    try {
      await linksService.deleteLink(id)
      setLinks(prevLinks => prevLinks.filter(link => link.id !== id))
      console.log(`Link ${id} deleted successfully.`)
    } catch (e: any) {
      console.error("Error deleting link:", e)
      setError(`Failed to delete link: ${e.message || 'Unknown error'}`)
    }
  }

  const renderContent = () => {
    switch (activeTab) {
      case 'tags':
        return <TagsManager />;
      case 'categories':
        return <CategoriesManager />;
      case 'profile':
        return <ProfileManager />;
      default:
        return (
          <div className="grid gap-4 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
            {filteredLinks.map((link) => (
              <motion.div
                key={link.id}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -20 }}
                className="bg-white dark:bg-gray-800 rounded-xl p-4 shadow-lg hover:shadow-xl transition-shadow border border-gray-200 dark:border-gray-700"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1 min-w-0">
                    <h3 className="font-semibold text-lg truncate">{link.title}</h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
                      {link.description}
                    </p>
                  </div>
                  {link.favicon_url && (
                    <img
                      src={link.favicon_url}
                      alt="Favicon"
                      className="w-6 h-6 rounded-full ml-2"
                    />
                  )}
                </div>

                {/* Tags and Categories */}
                <div className="mt-3 space-y-2">
                  {link.tags && link.tags.length > 0 && (
                    <div className="flex flex-wrap gap-2">
                      {link.tags.map(tag => (
                        <span
                          key={tag.id}
                          className="px-2 py-1 text-xs rounded-full bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300"
                        >
                          {tag.name}
                        </span>
                      ))}
                    </div>
                  )}
                  {link.categories && link.categories.length > 0 && (
                    <div className="flex flex-wrap gap-2">
                      {link.categories.map(category => (
                        <span
                          key={category.id}
                          className="px-2 py-1 text-xs rounded-full bg-pink-100 text-pink-700 dark:bg-pink-900/30 dark:text-pink-300"
                        >
                          {category.name}
                        </span>
                      ))}
                    </div>
                  )}
                </div>

                <div className="mt-4 flex items-center justify-between text-sm">
                  <div className="flex items-center text-gray-500 dark:text-gray-400">
                    <Eye size={16} className="mr-1" />
                    <span>{link.click_count} views</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <a
                      href={link.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="p-2 text-gray-500 hover:text-purple-500 dark:text-gray-400 dark:hover:text-purple-400 transition-colors"
                    >
                      <ExternalLink size={16} />
                    </a>
                    <button
                      onClick={() => handleDelete(link.id)}
                      className="p-2 text-gray-500 hover:text-red-500 dark:text-gray-400 dark:hover:text-red-400 transition-colors"
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        );
    }
  };

  return (
    <div className="flex flex-col h-full bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 rounded-xl overflow-hidden border border-purple-100 dark:border-purple-900/30 shadow-xl">
      {/* Header */}
      <div className="p-6 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <button
              onClick={() => setActiveTab('links')}
              className={`flex items-center px-4 py-2 rounded-lg transition-colors ${
                activeTab === 'links'
                  ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300'
                  : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
              }`}
            >
              <LinkIcon size={20} className="mr-2" />
              Links
            </button>
            <button
              onClick={() => setActiveTab('tags')}
              className={`flex items-center px-4 py-2 rounded-lg transition-colors ${
                activeTab === 'tags'
                  ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300'
                  : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
              }`}
            >
              <Tag size={20} className="mr-2" />
              Tags
            </button>
            <button
              onClick={() => setActiveTab('categories')}
              className={`flex items-center px-4 py-2 rounded-lg transition-colors ${
                activeTab === 'categories'
                  ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300'
                  : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
              }`}
            >
              <FolderOpen size={20} className="mr-2" />
              Categories
            </button>
            <button
              onClick={() => setActiveTab('profile')}
              className={`flex items-center px-4 py-2 rounded-lg transition-colors ${
                activeTab === 'profile'
                  ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300'
                  : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
              }`}
            >
              <Settings size={20} className="mr-2" />
              Profile
            </button>
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

        {/* Search bar - only show for links tab */}
        {activeTab === 'links' && (
          <div className="mt-4 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" size={20} />
            <input
              type="text"
              placeholder="Search links..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 dark:focus:ring-purple-400 focus:border-transparent"
            />
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6">
        {loading && activeTab === 'links' ? (
          <div className="flex items-center justify-center h-full">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500"></div>
          </div>
        ) : error ? (
          <div className="text-center text-red-500">{error}</div>
        ) : activeTab === 'links' && filteredLinks.length === 0 ? (
          <div className="text-center text-gray-500 dark:text-gray-400">
            {searchQuery ? "No links found matching your search." : "No links added yet."}
          </div>
        ) : (
          renderContent()
        )}
      </div>
    </div>
  )
} 