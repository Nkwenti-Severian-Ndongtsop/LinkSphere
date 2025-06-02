import React, { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { 
    Box, 
    Button, 
    TextField, 
    Typography, 
    Paper, 
    Container, 
    Alert,
    InputAdornment,
    IconButton,
    CircularProgress,
    useTheme
} from '@mui/material';
import { Visibility, VisibilityOff, Email, Lock } from '@mui/icons-material';
import { useAuth } from '../context/AuthContext';

interface LoginFormData {
    email: string;
    password: string;
}

interface ValidationErrors {
    email?: string;
    password?: string;
}

const Login: React.FC = () => {
    const navigate = useNavigate();
    const { login } = useAuth();
    const theme = useTheme();
    
    const [formData, setFormData] = useState<LoginFormData>({
        email: '',
        password: ''
    });
    const [showPassword, setShowPassword] = useState(false);
    const [errors, setErrors] = useState<ValidationErrors>({});
    const [apiError, setApiError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);

    const validateForm = (): boolean => {
        const newErrors: ValidationErrors = {};
        
        // Email validation
        if (!formData.email) {
            newErrors.email = 'Email is required';
        } else if (!/\S+@\S+\.\S+/.test(formData.email)) {
            newErrors.email = 'Please enter a valid email address';
        }

        // Password validation
        if (!formData.password) {
            newErrors.password = 'Password is required';
        } else if (formData.password.length < 8) {
            newErrors.password = 'Password must be at least 8 characters';
        }

        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const { name, value } = e.target;
        setFormData(prev => ({
            ...prev,
            [name]: value
        }));
        // Clear error when user starts typing
        if (errors[name as keyof ValidationErrors]) {
            setErrors(prev => ({
                ...prev,
                [name]: undefined
            }));
        }
        setApiError(null);
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        
        if (!validateForm()) {
            return;
        }

        setLoading(true);
        setApiError(null);

        try {
            const response = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    email: formData.email,
                    password: formData.password
                }),
                credentials: 'include' // Important for cookie handling
            });

            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.message || 'Login failed');
            }

            await login(data.token);
            navigate('/dashboard');
        } catch (err) {
            setApiError(err instanceof Error ? err.message : 'An error occurred during login');
        } finally {
            setLoading(false);
        }
    };

    return (
        <Container maxWidth="sm">
            <Box
                sx={{
                    mt: 8,
                    mb: 4,
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center'
                }}
            >
                <Paper 
                    elevation={3} 
                    sx={{
                        p: 4,
                        width: '100%',
                        borderRadius: 2,
                        background: theme.palette.background.paper,
                        boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)'
                    }}
                >
                    <Typography
                        component="h1"
                        variant="h4"
                        align="center"
                        gutterBottom
                        sx={{
                            color: theme.palette.primary.main,
                            fontWeight: 600,
                            mb: 4
                        }}
                    >
                        Welcome Back
                    </Typography>

                    {apiError && (
                        <Alert 
                            severity="error" 
                            sx={{ 
                                mb: 3,
                                borderRadius: 1
                            }}
                        >
                            {apiError}
                        </Alert>
                    )}

                    <form onSubmit={handleSubmit}>
                        <TextField
                            fullWidth
                            label="Email"
                            name="email"
                            type="email"
                            value={formData.email}
                            onChange={handleChange}
                            error={!!errors.email}
                            helperText={errors.email}
                            margin="normal"
                            required
                            InputProps={{
                                startAdornment: (
                                    <InputAdornment position="start">
                                        <Email color="action" />
                                    </InputAdornment>
                                ),
                            }}
                            sx={{
                                mb: 2,
                                '& .MuiOutlinedInput-root': {
                                    borderRadius: 1
                                }
                            }}
                        />

                        <TextField
                            fullWidth
                            label="Password"
                            name="password"
                            type={showPassword ? 'text' : 'password'}
                            value={formData.password}
                            onChange={handleChange}
                            error={!!errors.password}
                            helperText={errors.password}
                            margin="normal"
                            required
                            InputProps={{
                                startAdornment: (
                                    <InputAdornment position="start">
                                        <Lock color="action" />
                                    </InputAdornment>
                                ),
                                endAdornment: (
                                    <InputAdornment position="end">
                                        <IconButton
                                            onClick={() => setShowPassword(!showPassword)}
                                            edge="end"
                                        >
                                            {showPassword ? <VisibilityOff /> : <Visibility />}
                                        </IconButton>
                                    </InputAdornment>
                                )
                            }}
                            sx={{
                                mb: 3,
                                '& .MuiOutlinedInput-root': {
                                    borderRadius: 1
                                }
                            }}
                        />

                        <Button
                            fullWidth
                            type="submit"
                            variant="contained"
                            disabled={loading}
                            sx={{
                                mt: 2,
                                mb: 3,
                                py: 1.5,
                                borderRadius: 1,
                                textTransform: 'none',
                                fontSize: '1rem',
                                fontWeight: 600,
                                boxShadow: '0 4px 12px rgba(0, 0, 0, 0.1)',
                                '&:hover': {
                                    boxShadow: '0 6px 16px rgba(0, 0, 0, 0.2)'
                                }
                            }}
                        >
                            {loading ? (
                                <CircularProgress size={24} color="inherit" />
                            ) : (
                                'Sign In'
                            )}
                        </Button>

                        <Box sx={{ textAlign: 'center' }}>
                            <Typography variant="body2" color="textSecondary">
                                Don't have an account?{' '}
                                <Link 
                                    to="/register"
                                    style={{ 
                                        color: theme.palette.primary.main,
                                        textDecoration: 'none',
                                        fontWeight: 600
                                    }}
                                >
                                    Sign up
                                </Link>
                            </Typography>
                        </Box>
                    </form>
                </Paper>
            </Box>
        </Container>
    );
};

export default Login; 