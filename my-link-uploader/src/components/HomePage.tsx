"use client"

import React, { useState, useEffect, useRef } from "react"
import { Link } from "react-router-dom"
import { motion } from "framer-motion"
import { Search, ArrowRight, LinkIcon } from "lucide-react"

// Define the Link type based on backend struct
interface Link {
    id: number;
    user_id: string; // UUID from backend
    url: string;
    title: string;
    description: string;
    click_count: number;
    favicon_url?: string;
    created_at: string;
    updated_at: string;
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
    const [query, setQuery] = useState<string>("")
    const [isSearchFocused, setIsSearchFocused] = useState(false)
    const searchInputRef = useRef<HTMLInputElement>(null)
    const [links, setLinks] = useState<Link[]>([])
    const [filteredLinks, setFilteredLinks] = useState<Link[]>([])
    const [loading, setLoading] = useState<boolean>(true)
    const [error, setError] = useState<string | null>(null)

    const fetchLinks = async () => {
        setLoading(true)
        setError(null)
        try {
            const response = await fetch('/api/links')
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`)
            }
            const data: Link[] = await response.json()
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
        if (searchInputRef.current) {
            searchInputRef.current.focus()
        }
        fetchLinks()
    }, [])

    useEffect(() => {
        if (query) {
            const lowerCaseQuery = query.toLowerCase()
            const filtered = links.filter(
                (link) =>
                    link.title.toLowerCase().includes(lowerCaseQuery) ||
                    link.description.toLowerCase().includes(lowerCaseQuery) ||
                    link.url.toLowerCase().includes(lowerCaseQuery)
            )
            setFilteredLinks(filtered)
        } else {
            setFilteredLinks(links)
        }
    }, [query, links])

    const handleLinkClick = async (id: number) => {
        try {
            await fetch(`/api/links/${id}/click`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            })
            setLinks(prevLinks =>
                prevLinks.map(link =>
                    link.id === id ? { ...link, click_count: link.click_count + 1 } : link
                )
            )
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
        <div className="max-w-4xl mx-auto">
            <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5 }}
                className="text-center mb-12"
            >
                <h1 className="text-4xl font-bold bg-gradient-to-r from-purple-600 to-pink-500 bg-clip-text text-transparent mb-4">
                    Welcome to LinkSphere
                </h1>
                <p className="text-lg text-gray-600 dark:text-gray-400">
                    Your personal link management system
                </p>
            </motion.div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <motion.div
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: 0.2 }}
                    className="p-6 rounded-2xl bg-white dark:bg-gray-800 shadow-xl"
            >
                    <h2 className="text-2xl font-semibold mb-4 text-gray-800 dark:text-white">
                        Share Links
                </h2>
                    <p className="text-gray-600 dark:text-gray-400 mb-4">
                        Upload and share your favorite links with the community.
                        </p>
                        <Link
                            to="/upload"
                        className="inline-block bg-purple-600 text-white px-6 py-2 rounded-lg hover:bg-purple-700 transition-colors"
                        >
                        Start Sharing
                        </Link>
                    </motion.div>

                            <motion.div
                    initial={{ opacity: 0, x: 20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: 0.4 }}
                    className="p-6 rounded-2xl bg-white dark:bg-gray-800 shadow-xl"
                            >
                    <h2 className="text-2xl font-semibold mb-4 text-gray-800 dark:text-white">
                        Manage Links
                    </h2>
                    <p className="text-gray-600 dark:text-gray-400 mb-4">
                        Access your dashboard to manage all your shared links.
                                        </p>
                    <Link
                        to="/admin"
                        className="inline-block bg-purple-600 text-white px-6 py-2 rounded-lg hover:bg-purple-700 transition-colors"
                    >
                        Go to Dashboard
                    </Link>
                            </motion.div>
                    </div>
        </div>
    )
}

export default HomePage
