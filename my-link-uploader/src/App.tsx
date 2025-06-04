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
import UserDashboard from "./components/UserDashboard"

interface PrivateRouteProps {
  children: React.ReactNode;
  isAuthenticated: boolean;
  userRole?: string;
  requiredRole?: string;
}

const PrivateRoute = ({ children, isAuthenticated, userRole, requiredRole }: PrivateRouteProps) => {
  if (!isAuthenticated) {
    return <Navigate to="/auth/login" replace />;
  }
  
  if (requiredRole && userRole !== requiredRole) {
    return <Navigate to={userRole === 'admin' ? '/admin' : '/dashboard'} replace />;
  }
  
  return <>{children}</>;
};

export default function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false)
  const [userRole, setUserRole] = useState<string>('')

  useEffect(() => {
    const checkAuth = () => {
      const token = localStorage.getItem("token")
      const storedRole = localStorage.getItem("userRole")
      setIsAuthenticated(!!token)
      setUserRole(storedRole || '')
    }

    checkAuth()
    window.addEventListener("storage", checkAuth)
    return () => window.removeEventListener("storage", checkAuth)
  }, [])

  const handleLogin = (role: string) => {
    setIsAuthenticated(true)
    setUserRole(role)
    localStorage.setItem("userRole", role)
  }

  const handleLogout = () => {
    localStorage.removeItem("token")
    localStorage.removeItem("userRole")
    setIsAuthenticated(false)
    setUserRole('')
  }

  return (
    <BrowserRouter>
      <ThemeProvider>
        <AnimatePresence mode="wait">
          <Layout isAuthenticated={isAuthenticated} onLogout={handleLogout} userRole={userRole}>
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route 
                path="/admin" 
                element={
                  <PrivateRoute isAuthenticated={isAuthenticated} userRole={userRole} requiredRole="admin">
                    <AdminDashboard />
                  </PrivateRoute>
                } 
              />
              <Route 
                path="/dashboard" 
                element={
                  <PrivateRoute isAuthenticated={isAuthenticated} userRole={userRole} requiredRole="user">
                    <UserDashboard />
                  </PrivateRoute>
                } 
              />
              <Route 
                path="/upload" 
                element={
                  <PrivateRoute isAuthenticated={isAuthenticated} userRole={userRole}>
                    <UploadForm />
                  </PrivateRoute>
                } 
              />
              <Route 
                path="/auth/login" 
                element={
                  isAuthenticated ? 
                    <Navigate to={userRole === 'admin' ? '/admin' : '/dashboard'} replace /> : 
                    <Login onLoginSuccess={handleLogin} />
                } 
              />
              <Route 
                path="/auth/signup" 
                element={
                  isAuthenticated ? 
                    <Navigate to={userRole === 'admin' ? '/admin' : '/dashboard'} replace /> : 
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
