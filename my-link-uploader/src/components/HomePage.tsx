"use client"

import React, { useState, useEffect, useRef } from "react"
import { Link } from "react-router-dom"
import { motion } from "framer-motion"
import { Search, ArrowRight, LinkIcon, ExternalLink, Plus } from "lucide-react"

// Define the Link type based on backend struct
interface Link {
    id: string;
    url: string;
    title: string;
    description: string;
    click_count: number;
    favicon_url?: string;
    uploader_name: string;
    created_at: string;
}

// Define the ApiLink type from backend
interface ApiLink {
    id: number;
    user_id: number; // Might not be used directly in UI
    url: string;
    title: string;
    description: string;
    click_count: number;
    favicon_url?: string; // Optional
    created_at: string; // Date will be string from JSON
}

const HomePage: React.FC = () => {
    const [links, setLinks] = useState<Link[]>([])
    const [filteredLinks, setFilteredLinks] = useState<Link[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)
    const [searchQuery, setSearchQuery] = useState("")
    const isAuthenticated = !!localStorage.getItem("token")

    const fetchLinks = async () => {
        setLoading(true)
        setError(null)
        try {
            const response = await fetch('/api/links/public')
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`)
            }
            const data = await response.json()
            setLinks(data)
            setFilteredLinks(data)
        } catch (e: any) {
            console.error("Error fetching links:", e)
            setError("Failed to load links.")
        } finally {
            setLoading(false)
        }
    }

    useEffect(() => {
        fetchLinks()
    }, [])

    useEffect(() => {
        if (searchQuery) {
            const lowerCaseQuery = searchQuery.toLowerCase()
            const filtered = links.filter(
                (link) =>
                    link.title.toLowerCase().includes(lowerCaseQuery) ||
                    link.description.toLowerCase().includes(lowerCaseQuery) ||
                    link.url.toLowerCase().includes(lowerCaseQuery) ||
                    link.uploader_name.toLowerCase().includes(lowerCaseQuery)
            )
            setFilteredLinks(filtered)
        } else {
            setFilteredLinks(links)
        }
    }, [searchQuery, links])

    const handleIncrementClick = async (linkId: string) => {
        try {
            await fetch(`/api/links/${linkId}/increment-click`, {
                method: 'POST',
            })
        } catch (error) {
            console.error('Failed to increment click count:', error)
        }
    }

    const containerVariants = {
        hidden: { opacity: 0 },
        visible: {
            opacity: 1,
            transition: {
                staggerChildren: 0.1,
            },
        },
    }

    const itemVariants = {
        hidden: { y: 20, opacity: 0 },
        visible: {
            y: 0,
            opacity: 1,
            transition: { type: "spring", stiffness: 300, damping: 24 },
        },
    }

    return (
        <div className="min-h-screen bg-gradient-to-br from-purple-50 via-white to-pink-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900">
            {/* Hero Section */}
            <div className="py-16 px-4 sm:px-6 lg:px-8 text-center">
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.5 }}
                >
                    <div className="w-20 h-20 bg-gradient-to-br from-purple-600 to-pink-500 rounded-2xl mx-auto flex items-center justify-center shadow-lg shadow-purple-500/20 mb-6">
                        <LinkIcon size={32} className="text-white" />
                    </div>
                    <h1 className="text-4xl font-bold bg-gradient-to-r from-purple-600 to-pink-500 bg-clip-text text-transparent mb-4">
                        Welcome to LinkSphere
                    </h1>
                    <p className="text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto mb-8">
                        Discover and share valuable resources with the community
                    </p>
                    {isAuthenticated ? (
                        <Link
                            to="/upload"
                            className="inline-flex items-center px-6 py-3 bg-gradient-to-r from-purple-600 to-pink-500 text-white rounded-lg shadow-lg shadow-purple-500/20 hover:shadow-purple-500/40 transition-shadow"
                        >
                            <Plus size={20} className="mr-2" />
                            Share a Link
                        </Link>
                    ) : (
                        <div className="space-x-4">
                            <Link
                                to="/auth/login"
                                className="inline-flex items-center px-6 py-3 bg-gradient-to-r from-purple-600 to-pink-500 text-white rounded-lg shadow-lg shadow-purple-500/20 hover:shadow-purple-500/40 transition-shadow"
                            >
                                Login to Share Links
                            </Link>
                            <Link
                                to="/auth/signup"
                                className="inline-flex items-center px-6 py-3 bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 rounded-lg hover:bg-purple-200 dark:hover:bg-purple-800/40 transition-colors"
                            >
                                Sign Up
                            </Link>
                        </div>
                    )}
                </motion.div>
            </div>

            {/* Search Section */}
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mb-8">
                <div className="relative">
                    <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 text-gray-400" size={20} />
                    <input
                        type="text"
                        placeholder="Search links..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full pl-12 pr-4 py-3 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-purple-500 dark:focus:ring-purple-400 focus:border-transparent"
                    />
                </div>
            </div>

            {/* Links Grid */}
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pb-16">
                {loading ? (
                    <div className="flex justify-center items-center py-12">
                        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500"></div>
                    </div>
                ) : error ? (
                    <div className="text-center text-red-500 py-12">{error}</div>
                ) : filteredLinks.length === 0 ? (
                    <div className="text-center text-gray-500 dark:text-gray-400 py-12">
                        No links found. {isAuthenticated && "Be the first to share one!"}
                    </div>
                ) : (
                    <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                        {filteredLinks.map((link) => (
                            <motion.div
                                key={link.id}
                                initial={{ opacity: 0, y: 20 }}
                                animate={{ opacity: 1, y: 0 }}
                                className="bg-white dark:bg-gray-800 rounded-xl shadow-md hover:shadow-lg transition-shadow p-6 border border-purple-100 dark:border-purple-900/30"
                            >
                                <div className="flex items-start justify-between mb-4">
                                    <div className="flex items-center">
                                        {link.favicon_url ? (
                                            <img src={link.favicon_url} alt="" className="w-6 h-6 rounded mr-2" />
                                        ) : (
                                            <LinkIcon size={20} className="text-purple-500 mr-2" />
                                        )}
                                        <h3 className="font-semibold text-gray-900 dark:text-white truncate">
                                            {link.title}
                                        </h3>
                                    </div>
                                </div>
                                <p className="text-gray-600 dark:text-gray-400 text-sm mb-4 line-clamp-2">
                                    {link.description}
                                </p>
                                <div className="flex items-center justify-between text-sm">
                                    <span className="text-gray-500 dark:text-gray-400">
                                        By {link.uploader_name}
                                    </span>
                                    <span className="text-gray-500 dark:text-gray-400">
                                        {link.click_count} clicks
                                    </span>
                                </div>
                                <div className="mt-4">
                                    <a
                                        href={link.url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        onClick={() => handleIncrementClick(link.id)}
                                        className="inline-flex items-center text-purple-600 dark:text-purple-400 hover:text-purple-700 dark:hover:text-purple-300"
                                    >
                                        Visit Link
                                        <ExternalLink size={14} className="ml-1" />
                                    </a>
                                </div>
                            </motion.div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    )
}

export default HomePage
