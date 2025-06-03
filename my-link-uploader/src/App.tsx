"use client"

import { useState, useEffect } from "react"
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom"
import { ThemeProvider } from "./components/ThemeProvider"
import { AnimatePresence } from "framer-motion"
import Layout from "./components/Layout"
import HomePage from "./components/HomePage"
import AdminDashboard from "./components/AdminDashboard"
import UploadForm from "./components/UploadForm"
import Login from "./pages/auth/Login"
import Signup from "./pages/auth/Signup"

interface PrivateRouteProps {
  children: React.ReactNode;
  isAuthenticated: boolean;
}

const PrivateRoute = ({ children, isAuthenticated }: PrivateRouteProps) => {
  if (!isAuthenticated) {
    return <Navigate to="/auth/login" replace />;
  }
  return <>{children}</>;
};

export default function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false)

  useEffect(() => {
    const checkAuth = () => {
      const token = localStorage.getItem("token")
      setIsAuthenticated(!!token)
    }

    checkAuth()
    window.addEventListener("storage", checkAuth)
    return () => window.removeEventListener("storage", checkAuth)
  }, [])

  const handleLogin = () => {
    setIsAuthenticated(true)
  }

  const handleLogout = () => {
    localStorage.removeItem("token")
    setIsAuthenticated(false)
  }

  return (
    <BrowserRouter>
      <ThemeProvider>
        <AnimatePresence mode="wait">
          <Layout isAuthenticated={isAuthenticated} onLogout={handleLogout}>
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route 
                path="/admin" 
                element={
                  <PrivateRoute isAuthenticated={isAuthenticated}>
                    <AdminDashboard />
                  </PrivateRoute>
                } 
              />
              <Route 
                path="/upload" 
                element={
                  <PrivateRoute isAuthenticated={isAuthenticated}>
                    <UploadForm />
                  </PrivateRoute>
                } 
              />
              <Route 
                path="/auth/login" 
                element={
                  isAuthenticated ? 
                    <Navigate to="/admin" replace /> : 
                    <Login onLoginSuccess={handleLogin} />
                } 
              />
              <Route 
                path="/auth/signup" 
                element={
                  isAuthenticated ? 
                    <Navigate to="/admin" replace /> : 
                    <Signup />
                } 
              />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </Layout>
        </AnimatePresence>
      </ThemeProvider>
    </BrowserRouter>
  )
}
